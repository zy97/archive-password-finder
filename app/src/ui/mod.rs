use eframe::{
    egui::{Layout, Ui},
    emath::Align,
};

use crate::app::App;

pub fn file_selector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "file path: {}",
            app.file_path.clone().unwrap_or(String::new())
        ));
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("selelct").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.file_path = Some(path.display().to_string());
                }
            }
        });
    });
    ui.end_row();
}
pub fn dictionary_selector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "dictionary path: {}",
            app.dictionary_path.clone().unwrap_or(String::new())
        ));
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("selelct").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.dictionary_path = Some(path.display().to_string());
                }
            }
        });
    });
    ui.end_row();
}
