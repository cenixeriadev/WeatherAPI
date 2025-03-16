use std::env;
use dotenvy::dotenv;
use redis::Commands;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Deserialize,Serialize,Debug)]
pub struct WeatherResponse {
    pub main: Main,
    pub weather: Vec<Weather>,
    pub wind: Wind,
    pub name: String,
}

#[derive(Deserialize,Serialize,Debug)]
pub struct Main {
    pub temp: f64,
    pub humidity: u8,
    pub pressure: u16,
}

#[derive(Deserialize,Serialize , Debug)]
pub struct Weather {
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize,Serialize ,Debug)]
pub struct Wind {
    pub speed: f64,
}
lazy_static!{//para compilar el regex solo una vez
    static ref RE_CITY: Regex = Regex::new(r"^[\p{L}\s,.'-]+$").unwrap();
    static ref RE_COUNTRY_CODE: Regex = Regex::new(r"^[A-Z]{2}$").unwrap();
}
const REQUESTS_PER_MINUTE: u64 = 15;
const RATE_LIMIT_WINDOW: i64 = 60;

fn validate_city(city: &str) -> bool {
    let city = city.trim();
    !city.is_empty() && city.len() >= 2 && RE_CITY.is_match(city)
}

fn validate_country_code(code: &str) -> bool {
    RE_COUNTRY_CODE.is_match(code)
}


pub fn fetch_weather(city: &str, country_code: &str) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    if city.is_empty() || country_code.is_empty() {
        return Err("You must fill out all required fields".into());
    }
    if !validate_city(city) || !validate_country_code(country_code){
        return Err("You must enter valid values".into());
    }
    dotenv().ok();

    let api_key = env::var("API_KEY").expect("Could not find API key in .env");
    let redis_url = env::var("REDIS_URL").expect("Could not find REDIS URL in .env");
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;
    let cache_key = format!("weather:{}:{}", city, country_code);
    let rate_limit_key = format!("ratelimit:{}:{}", city, country_code);
    let count: u64 = con.incr(&rate_limit_key, 1)?; // Incrementa el contador

    // Si es la primera solicitud, establece expiración
    if count == 1 {
        con.expire(&rate_limit_key, RATE_LIMIT_WINDOW)?;
    }

    if count > REQUESTS_PER_MINUTE {
        return Err("Rate limit exceeded".into());
    }
    if let Ok(Some(cached_data)) = con.get::<_, Option<String>>(&cache_key) {
        let weather_response: WeatherResponse = serde_json::from_str(&cached_data)?;
        return Ok(weather_response);
    }

    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={},{}&units=metric&appid={}",
        city, country_code, api_key
    );
    let rt = Builder::new_current_thread().enable_all().build()?;
    let mut response: WeatherResponse = rt.block_on(async {
        reqwest::get(&url).await?.json::<WeatherResponse>().await
    })?;
    if response.main.temp< 0.0 {
        response.weather[0].icon = "snow".parse().unwrap();
    }else if  response.main.temp>=0.0 && response.main.temp<=10.0 {
        response.weather[0].icon = "mist".parse().unwrap();
    }else if response.main.temp>=10.0 && response.main.temp<20.0 {
        response.weather[0].icon = "rain".parse().unwrap();
    }else if response.main.temp>=20.0 && response.main.temp<25.0 {
        response.weather[0].icon = "cloud".parse().unwrap();
    }else{
        response.weather[0].icon = "clear".parse().unwrap();
    }
    let _: () = con.set_ex(&cache_key, serde_json::to_string(&response)?, 3600)?;
    let _: () = con.set_ex(&rate_limit_key, count, 60)?;

    Ok(response)
}
