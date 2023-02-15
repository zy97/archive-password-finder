use eframe::{
    egui::{Layout, Slider, Ui},
    emath::Align,
};

use crate::{app::App, Mode};

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

pub fn mode_selector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.radio_value(&mut app.mode, Mode::PasswordDictionary, "字典");
        ui.radio_value(&mut app.mode, Mode::Generation, "字符");
        ui.radio_value(&mut app.mode, Mode::Custom, "自定义");
    });
    ui.end_row();
}

pub fn worker_slider(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("工作线程数(默认为CPU物理核心数)：");
        let slider = Slider::new(&mut app.workers_count, 1..=100);
        ui.add(slider);
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

pub fn password_length(app: &mut App, ui: &mut Ui) {
    if app.min_pasword_length >= app.max_pasword_length {
        app.max_pasword_length = app.min_pasword_length;
    }
    ui.horizontal(|ui| {
        ui.label("最短密码长度：");
        let min_max = app.min_pasword_length;
        let min_slider = Slider::new(&mut app.min_pasword_length, 1..=16);
        ui.label("最长密码长度：");
        let max_slider = Slider::new(&mut app.max_pasword_length, min_max.to_owned()..=16);
        ui.add(min_slider);
        ui.add_space(32.0);
        ui.add(max_slider);
    });
    ui.end_row();
}

pub fn progress_bar(app: &mut App, ui: &mut Ui) {
    let progressbar = eframe::egui::ProgressBar::new(app.progress).show_percentage();
    ui.add(progressbar);
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        let msg = match app.password_count.as_ref() {
            Ok(count) => count.to_string(),
            Err(err) => err.to_owned(),
        };
        ui.label(format!("{}/{}", app.tested_count, msg));

        if app.current_time.is_some() {
            let time = app.current_time.unwrap() - app.start_time.unwrap();
            ui.label(format!(
                "{:0>2}:{:0>2}:{:0>2}",
                time.whole_hours(),
                time.whole_minutes(),
                time.whole_seconds() % 60
            ));
        }
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
