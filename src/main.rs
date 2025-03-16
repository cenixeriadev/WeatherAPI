mod weather_app_ui;
mod weather_api;
use eframe::NativeOptions;
use weather_app_ui::WeatherApp;

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions::default();
    eframe::run_native("Weather App", options, Box::new(|_cc| Box::new(WeatherApp::default())))
}