use std::env;
use dotenvy::dotenv;
use redis::Commands;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

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
const REQUESTS_PER_MINUTE: u64 = 12;
const RATE_LIMIT_WINDOW: i64 = 60;
pub fn fetch_weather(city: &str, country: &str) -> Result<WeatherResponse, Box<dyn std::error::Error>> {

    dotenv().ok();

    let api_key = env::var("API_KEY").expect("Could not find API key in .env");

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    let cache_key = format!("weather:{}:{}", city, country);
    let rate_limit_key = format!("ratelimit:{}:{}", city, country);
    let count: u64 = con.incr(&rate_limit_key, 1)?; // Incrementa el contador

    // Si es la primera solicitud, establece expiración
    if count == 1 {
        con.expire(&rate_limit_key, RATE_LIMIT_WINDOW)?;
    }

    // Bloquear después de 20 solicitudes
    if count > REQUESTS_PER_MINUTE {
        return Err("Rate limit exceeded".into());
    }
    // Verificar si hay datos en caché
    if let Ok(Some(cached_data)) = con.get::<_, Option<String>>(&cache_key) {
        let weather_response: WeatherResponse = serde_json::from_str(&cached_data)?;
        return Ok(weather_response);
    }

    // Verificar el límite de solicitudes


    // Crear la URL para la API
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={},{}&units=metric&appid={}",
        city, country, api_key
    );

    // Crear un Runtime para ejecutar la solicitud asincrónica
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
    // Almacenar en caché el resultado
    let _: () = con.set_ex(&cache_key, serde_json::to_string(&response)?, 3600)?;
    let _: () = con.set_ex(&rate_limit_key, count, 60)?;

    Ok(response)
}
