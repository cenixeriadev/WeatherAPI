use eframe::egui;
use std::fs;
use crate::weather_api::{fetch_weather, WeatherResponse};

pub struct WeatherApp {
    city: String,
    country: String,
    weather_info: Option<WeatherResponse>,
    error_message: String,
    weather_icon: Option<egui::TextureHandle>,  // Nueva variable para almacenar la imagen
}

impl Default for WeatherApp {
    fn default() -> Self {
        Self {
            city: String::new(),
            country: String::new(),
            weather_info: None,
            error_message: String::new(),
            weather_icon: None,
        }
    }
}

impl WeatherApp {
    fn load_weather_icon(&mut self, ctx: &egui::Context, icon_code: &str) {
        let image_path = format!("img/{}.png", icon_code);
        if let Ok(image_data) = fs::read(&image_path) {
            // Cargar la imagen como una textura
            match image::load_from_memory(&image_data) {
                Ok(mut image) => {
                    image = image.resize_exact(100, 100, image::imageops::FilterType::Nearest);
                    let size = [image.width() as _, image.height() as _];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        image.as_rgba8().unwrap().as_ref(),
                    );
                    let texture_handle = ctx.load_texture(icon_code, color_image, Default::default());
                    self.weather_icon = Some(texture_handle); // Guardar la textura
                }
                Err(e) => eprintln!("Error cargando imagen: {}", e),
            }
        } else {
            eprintln!("No se pudo leer la imagen desde: {}", image_path);
        }
    }
}

impl eframe::App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Título principal
            let style = ui.style_mut();
            style.spacing.item_spacing = egui::vec2(10.0, 12.0);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 42, 54);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(68, 71, 90);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 84, 105);
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(
                    egui::RichText::new("Weather App")
                        .color(egui::Color32::from_rgb(255, 184, 108))
                        .heading()
                        .size(24.0),
                );
                ui.add_space(10.0);
            });

            // Sección de búsqueda
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Ciudad")
                                .color(egui::Color32::from_rgb(189, 147, 249)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.city)
                                .hint_text("Ej: Madrid")
                                .desired_width(200.0)
                                .text_color(egui::Color32::WHITE),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("País")
                                .color(egui::Color32::from_rgb(189, 147, 249)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.country)
                                .hint_text("Ej: ES")
                                .desired_width(80.0)
                                .text_color(egui::Color32::WHITE),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.add_space(20.0);
                        if ui
                            .add(
                                egui::Button::new("🔍 Buscar")
                                    .fill(egui::Color32::from_rgb(80, 200, 120))
                                    .min_size(egui::vec2(15.0, 8.0))
                            )
                            .clicked()
                        {
                            // Lógica de búsqueda...
                            match fetch_weather(&self.city, &self.country) {
                                Ok(response) => {
                                    self.load_weather_icon(&ctx,&response.weather[0].icon);
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
                });
            });

            // Mensaje de error
            if !self.error_message.is_empty() {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, "❗");
                    ui.colored_label(
                        egui::Color32::RED,
                        egui::RichText::new(&self.error_message).strong().size(20.0),
                    );
                });
            }

            // Sección de resultados
            if let Some(weather) = &self.weather_info {
                ui.add_space(20.0);
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(40, 42, 54))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            // Icono del clima
                            if let Some(image) = &self.weather_icon {
                                //image.show_size(ui, egui::vec2(100.0, 100.0));
                                ui.image((image.id(), image.size_vec2()));
                            }

                            // Información principal
                            ui.heading(
                                egui::RichText::new(format!(
                                    "{} ({})",
                                    weather.name,
                                    self.country.to_uppercase()
                                ))
                                    .color(egui::Color32::from_rgb(139, 233, 253))
                                    .size(20.0),
                            );

                            ui.add_space(15.0);
                            ui.label(
                                egui::RichText::new(format!("{:.1}°C", weather.main.temp))
                                    .color(egui::Color32::from_rgb(255, 121, 198))
                                    .size(40.0),
                            );

                            // Detalles secundarios
                            ui.add_space(20.0);
                            egui::Grid::new("weather_details")
                                .spacing(egui::vec2(30.0, 10.0))
                                .show(ui, |ui| {
                                    weather_detail(ui, "💧 Humedad", &format!("{}%", weather.main.humidity));
                                    weather_detail(ui, "📉 Presión", &format!("{} hPa", weather.main.pressure));
                                    weather_detail(ui, "🌬️ Viento", &format!("{} m/s", weather.wind.speed));
                                    ui.end_row();
                                });
                        });
                    });
            }
        });
    }
}

fn weather_detail(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(egui::Color32::from_rgb(189, 147, 249))
                .size(14.0),
        );
        ui.label(
            egui::RichText::new(value)
                .color(egui::Color32::WHITE)
                .strong()
                .size(16.0),
        );
    });
}
