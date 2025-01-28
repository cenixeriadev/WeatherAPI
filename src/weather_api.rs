use std::env;
use std::time::{SystemTime};
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

pub fn fetch_weather(city: &str, country: &str) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    // Cargar las variables del archivo .env
    dotenv().ok();

    // Obtener la clave de la API desde las variables de entorno
    let api_key = env::var("API_KEY").expect("No se pudo encontrar la clave de la API en .env");

    // Conexión con Redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    let cache_key = format!("weather:{}:{}", city, country);
    let rate_limit_key = format!("ratelimit:{}:{}", city, country);

    // Verificar si hay datos en caché
    if let Ok(Some(cached_data)) = con.get::<_, Option<String>>(&cache_key) {
        let weather_response: WeatherResponse = serde_json::from_str(&cached_data)?;
        return Ok(weather_response);
    }

    // Verificar el límite de solicitudes
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    if let Ok(Some(last_request)) = con.get::<_, Option<u64>>(&rate_limit_key) {
        if now - last_request < 60 {
            return Err("Rate limit exceeded".into());
        }
    }

    // Crear la URL para la API
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={},{}&units=metric&appid={}",
        city, country, api_key
    );

    // Crear un Runtime para ejecutar la solicitud asincrónica
    let rt = Builder::new_current_thread().enable_all().build()?;
    let response: WeatherResponse = rt.block_on(async {
        reqwest::get(&url).await?.json::<WeatherResponse>().await
    })?;

    // Almacenar en caché el resultado
    let _: () = con.set_ex(&cache_key, serde_json::to_string(&response)?, 3600)?;
    let _: () = con.set_ex(&rate_limit_key, now, 60)?;

    Ok(response)
}
