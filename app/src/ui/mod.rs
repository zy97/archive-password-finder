use eframe::{
    egui::{Layout, Ui},
    emath::Align,
};
use password_crack::{get_password_count, Strategy};

use crate::app::App;

pub fn file_selector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "文件路径: {}",
            app.file_path.clone().unwrap_or(String::new())
        ));
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("选择").clicked() {
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
            "字典路径: {}",
            app.dictionary_path.clone().unwrap_or(String::new())
        ));
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("选择").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.dictionary_path = Some(path.display().to_string());
                    // let strategy = Strategy::PasswordFile(path);
                    // let count = get_password_count(&strategy).unwrap();
                    // app.password_count = count;
                }
            }
        });
    });
    ui.end_row();
}

pub fn progress_bar(app: &mut App, ui: &mut Ui) {
    let progressbar = eframe::egui::ProgressBar::new(app.progress).show_percentage();
    ui.add(progressbar);
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.label(format!("{}/{}", app.tested_count, app.password_count));
    });
    if app.find_result.is_some() {
        match app.find_result.as_ref().unwrap() {
            Some(password) => {
                ui.label(format!("已查找密码: {}", password));
            }
            None => {
                ui.label("未查找密码！");
            }
        }
    }
    ui.end_row();
}
