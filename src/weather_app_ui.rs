
use eframe::egui;
use crate::weather_api::{fetch_weather, WeatherResponse};

pub struct WeatherApp {
    city: String,
    country: String,
    weather_info: Option<WeatherResponse>,
    error_message: String,
}

impl Default for WeatherApp {
    fn default() -> Self {
        Self {
            city: String::new(),
            country: String::new(),
            weather_info: None,
            error_message: String::new(),
        }
    }
}

impl eframe::App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Aplicación de Clima");

            // Sección de entrada de ciudad y país
            ui.horizontal(|ui| {
                ui.label("Ciudad:");
                ui.text_edit_singleline(&mut self.city);
                ui.label("Código de país:");
                ui.text_edit_singleline(&mut self.country);
                if ui.button("Buscar").clicked() {
                    match fetch_weather(&self.city, &self.country) {
                        Ok(response) => {
                            self.weather_info = Some(response);
                            self.error_message.clear();
                        }
                        Err(e) => {
                            self.weather_info = None;
                            self.error_message = format!("Error: {}", e);
                        }
                    }
                }
            });

            // Mostrar mensaje de error si existe
            if !self.error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error_message);
            }

            // Mostrar información del clima dentro de un área desplazable
            if let Some(weather) = &self.weather_info {
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(format!("{} ({})", weather.name, self.country.to_uppercase()));
                    ui.image(format!("img/{}.png", weather.weather[0].icon));
                    ui.label(format!("Temperatura: {} °C", weather.main.temp));
                    ui.label(format!("Humedad: {}%", weather.main.humidity));
                    ui.label(format!("Presión: {} hPa", weather.main.pressure));
                    ui.label(format!("Velocidad del viento: {} m/s", weather.wind.speed));
                });
            }
        });
    }
}

