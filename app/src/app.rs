use eframe::egui::{self};

use crate::Mode;

#[derive()]
pub struct App {
    
    pub file_path: Option<String>,
    pub dictionary_path: Option<String>,
    mode: Mode,
    pub selected_charset: [bool; 4],
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                crate::ui::file_selector(self, ui);

                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.mode, Mode::PasswordDictionary, "password");
                    ui.radio_value(&mut self.mode, Mode::Generation, "letter");
                    ui.radio_value(&mut self.mode, Mode::Custom, "custom");
                });
                ui.end_row();

                match self.mode {
                    Mode::PasswordDictionary => {
                        crate::ui::dictionary_selector(self, ui);
                    }
                    Mode::Generation => {
                        ui.horizontal(|ui| {
                            ui.toggle_value(&mut self.selected_charset[0], "digits");
                            ui.toggle_value(&mut self.selected_charset[1], "lower");
                            ui.toggle_value(&mut self.selected_charset[2], "upper");
                            ui.toggle_value(&mut self.selected_charset[3], "special");
                        });
                    }
                    Mode::Custom => {
                        ui.horizontal(|ui| {
                            ui.label("custom charset: ");
                            ui.text_edit_singleline(&mut String::new());
                        });
                    }
                }
                ui.end_row();

                if ui.button("start").clicked() {
                    match self.mode {
                        Mode::PasswordDictionary => {
                            egui::Window::new("test dictionnary")
                                .collapsible(false)
                                .resizable(false)
                                .show(ctx, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("Yes!").clicked() {
                                            todo!()
                                        }
                                    })
                                });
                        }
                        Mode::Generation => {}
                        Mode::Custom => {}
                    }
                }

                let progressbar = eframe::egui::ProgressBar::new(1.0);
                ui.add(progressbar);
            });
        });
    }
}
impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            mode: Mode::PasswordDictionary,
            dictionary_path: None,
            file_path: None,
            selected_charset: [true, false, false, false],
        }
    }
}
