mod app;
mod font;
mod ui;

use app::App;
use eframe::Result;

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
        Box::new(|_cc| {
            let app = App::new(_cc);
            Box::new(app)
        }),
    )
}
#[derive(PartialEq)]
enum Mode {
    PasswordDictionary,
    Generation,
    Custom,
}
#[derive(PartialEq)]
enum Charset {
    Lower,
    Upper,
    Digital,
    Special,
}
