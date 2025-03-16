use eframe::egui;
use std::fs;
use crate::weather_api::{fetch_weather, WeatherResponse};

pub struct WeatherApp {
    city: String,
    country: String,
    temp_country : String,
    weather_info: Option<WeatherResponse>,
    error_message: String,
    weather_icon: Option<egui::TextureHandle>,
    description : String,
}

impl Default for WeatherApp {
    fn default() -> Self {
        Self {
            city: String::new(),
            country: String::new(),
            temp_country : String::new(),
            weather_info: None,
            error_message: String::new(),
            weather_icon: None,
            description : String::new(),
        }
    }
}

impl WeatherApp {
    fn load_weather_icon(&mut self, ctx: &egui::Context, icon_code: &str) {
        let image_path = format!("img/{}.png", icon_code);
        if let Ok(image_data) = fs::read(&image_path) {
            match image::load_from_memory(&image_data) {
                Ok(mut image) => {
                    image = image.resize_exact(100, 100, image::imageops::FilterType::Nearest);
                    let size = [image.width() as _, image.height() as _];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        image.as_rgba8().unwrap().as_ref(),
                    );
                    let texture_handle = ctx.load_texture(icon_code, color_image, Default::default());
                    self.weather_icon = Some(texture_handle);
                }
                Err(e) => eprintln!("Error loading image : {}", e),
            }
        } else {
            eprintln!("Could not read image from : {}", image_path);
        }
    }
}

impl eframe::App for WeatherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let style = ui.style_mut();
            style.spacing.item_spacing = egui::vec2(10.0, 12.0);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 42, 54);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(68, 71, 90);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 84, 105);

            ui.add_space(30.0);

            ui.group(|ui| {
                ui.horizontal(|ui| {

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("City")
                                .color(egui::Color32::from_rgb(189, 147, 249)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.city)
                                .hint_text("Ex: Madrid")
                                .desired_width(200.0)
                                .text_color(egui::Color32::WHITE),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Country")
                                .color(egui::Color32::from_rgb(189, 147, 249)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.temp_country)
                                .hint_text("Ex: ES")
                                .desired_width(80.0)
                                .text_color(egui::Color32::WHITE),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.add_space(20.0);
                        if ui
                            .add(
                                egui::Button::new("🔍 Search")
                                    .fill(egui::Color32::from_rgb(80, 200, 120))
                                    .min_size(egui::vec2(15.0, 8.0))
                            )
                            .clicked()
                        {
                            self.country = self.temp_country.clone();
                            match fetch_weather(&self.city, &self.country) {
                                Ok(response) => {
                                    self.load_weather_icon(&ctx,&response.weather[0].icon);
                                    self.description = response.weather[0].description.to_string();
                                    self.weather_info = Some(response);
                                    self.error_message.clear();
                                }
                                Err(e) => {
                                    self.weather_info = None;
                                    self.error_message = format!("{}", e);
                                }
                            }

                        }
                    });
                });
            });

            // Mensaje de error
            if !self.error_message.is_empty() {
                egui::Window::new("¡Error!")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::RED, "❗");
                            ui.label(
                                egui::RichText::new(&self.error_message)
                                    .color(egui::Color32::LIGHT_RED)
                            );
                        });

                        ui.vertical_centered(|ui| {
                            if ui.button("Accept").clicked() {
                                self.error_message.clear();
                                self.city.clear();
                                self.country.clear();
                            }
                        });
                    });
            }

            if let Some(weather) = &self.weather_info {
                ui.add_space(20.0);
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(40, 42, 54))
                    .inner_margin(20.0)
                    .show(ui, |ui| {

                        ui.vertical_centered(|ui| {

                            if let Some(image) = &self.weather_icon {
                                ui.image((image.id(), image.size_vec2()));
                            }

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


                            ui.add_space(20.0);
                            egui::Grid::new("weather_details")
                                .spacing(egui::vec2(30.0, 10.0))
                                .min_col_width(100.0)
                                .show(ui, |ui| {
                                    weather_detail(ui, "💧 Humedity", &format!("{}%", weather.main.humidity));
                                    weather_detail(ui, "📉 Pressure", &format!("{} hPa", weather.main.pressure));
                                    weather_detail(ui, "🌀 Wind velocity", &format!("{} m/s", weather.wind.speed));
                                    weather_detail(ui , "✨ Description" , &format!("{}" , &self.description));
                                    ui.end_row();
                                });
                        });
                    });
            }
        });
    }
}

fn weather_detail(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
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
