use std::{
    path::Path,
    sync::mpsc::{channel, Receiver},
    thread,
};

use eframe::egui::{self};
use password_crack::{get_password_count, password_finder, Strategy};
use time::OffsetDateTime;

use crate::{font::setup_custom_fonts, ui::progress_bar, Mode};

#[derive()]
pub struct App {
    pub file_path: Option<String>,
    pub dictionary_path: Option<String>,
    pub mode: Mode,
    pub workers_count: usize,
    pub selected_charset: [bool; 4],
    pub password_count: usize,
    pub tested_count: usize,
    pub progress: f32,
    strategy: Option<Strategy>,
    running: bool,
    pub find_result: Option<Option<String>>,
    progress_receiver: Option<Receiver<u64>>,
    password_receiver: Option<Receiver<Option<Option<String>>>>,
    pub start_time: Option<OffsetDateTime>,
    pub current_time: Option<OffsetDateTime>,
    // timer:time
}
impl App {
    fn reset(self: &mut Self) {
        self.start_time = Some(OffsetDateTime::now_utc());
        self.current_time = None;
        self.tested_count = 0;
        self.find_result = None;
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &self.dictionary_path {
            Some(dict_path) => {
                let path = Path::new(&dict_path);
                let strategy = Strategy::PasswordFile(path.to_path_buf());
                self.strategy = Some(strategy);
            }
            // None => Strategy::GenPasswords {
            //     charsets,
            //     min_password_len,
            //     max_password_len,
            // },
            None => {}
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.ctx().request_repaint();
            ui.vertical_centered_justified(|ui| {
                crate::ui::file_selector(self, ui);
                crate::ui::mode_selector(self, ui);
                crate::ui::worker_slider(self, ui);
                match self.mode {
                    Mode::PasswordDictionary => {
                        crate::ui::dictionary_selector(self, ui);
                    }
                    Mode::Generation => {
                        ui.horizontal(|ui| {
                            ui.toggle_value(&mut self.selected_charset[0], "数字");
                            ui.toggle_value(&mut self.selected_charset[1], "小写字母");
                            ui.toggle_value(&mut self.selected_charset[2], "大写字母");
                            ui.toggle_value(&mut self.selected_charset[3], "特殊字符");
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

                ui.add_enabled_ui(
                    self.strategy.is_some() && self.file_path.is_some() && !self.running,
                    |ui| {
                        if ui.button("开始").clicked() {
                            self.running = true;
                            let file = self.file_path.clone();
                            let strategy = self.strategy.clone();
                            let strategy1 = self.strategy.clone();
                            let (send_progress_info, receive_progress_info) = channel();
                            let (send_password_find, receive_password_find) = channel();
                            self.progress_receiver = Some(receive_progress_info);
                            self.password_receiver = Some(receive_password_find);
                            self.password_count = get_password_count(&strategy1.unwrap()).unwrap();
                            let work_count = self.workers_count;
                            self.reset();
                            thread::spawn(move || {
                                match password_finder(
                                    &file.unwrap(),
                                    work_count,
                                    strategy.unwrap(),
                                    send_progress_info,
                                ) {
                                    Ok(Some(password)) => {
                                        println!("password: {}", password);
                                        send_password_find.send(Some(Some(password))).unwrap();
                                    }
                                    Ok(None) => {
                                        send_password_find.send(Some(None)).unwrap();
                                    }
                                    Err(e) => {
                                        println!("err: {:?}", e);
                                        send_password_find.send(None).unwrap();
                                    }
                                }
                            });
                        }
                    },
                );

                progress_bar(self, ui);
                if self.progress_receiver.is_some() {
                    if let Ok(r) = self.progress_receiver.as_ref().unwrap().try_recv() {
                        self.current_time = Some(OffsetDateTime::now_utc());
                        self.tested_count += r as usize;
                        self.progress = self.tested_count as f32 / self.password_count as f32;
                        println!("progress: {}", self.progress)
                    }
                }
                if self.password_receiver.is_some() {
                    if let Ok(r) = self.password_receiver.as_ref().unwrap().try_recv() {
                        self.find_result = r;
                        self.running = false;
                        self.progress_receiver = None;
                    }
                }
            });
        });
    }
}
impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        Self {
            mode: Mode::PasswordDictionary,
            dictionary_path: None,
            file_path: None,
            selected_charset: [true, false, false, false],
            password_count: 0,
            progress: 0.0,
            strategy: None,
            running: false,
            find_result: None,
            tested_count: 0,
            progress_receiver: None,
            password_receiver: None,
            start_time: None,
            current_time: None,
            workers_count: num_cpus::get_physical(),
        }
    }
}
