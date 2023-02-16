#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![no_main]
mod app;
mod font;
mod ui;

use app::App;
use eframe::{egui, Result};

#[no_mangle]
fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        resizable: false,
        default_theme: eframe::Theme::Light,
        drag_and_drop_support: false,
        ..Default::default()
    };
    eframe::run_native(
        "爆破",
        options,
        Box::new(|cc| {
            let app = App::new(cc);
            Box::new(app)
        }),
    )
}
#[derive(PartialEq)]
pub enum Mode {
    PasswordDictionary,
    Generation,
    Custom,
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}
