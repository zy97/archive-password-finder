use std::{
    path::Path,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
};

use eframe::egui::{self};
use password_crack::{CharsetChoice, Cracker, Strategy};
use time::OffsetDateTime;

use crate::{font::setup_custom_fonts, ui::progress_bar, Mode};

#[derive()]
pub struct App {
    pub file_path: Option<String>,
    pub dictionary_path: Option<String>,
    pub mode: Mode,
    pub workers_count: usize,
    pub selected_charset: [(CharsetChoice, bool); 4],
    pub password_count: Result<usize, String>,
    pub tested_count: usize,
    pub progress: f32,
    strategy: Option<Strategy>,
    running: bool,
    pub find_result: Option<Option<String>>,
    password_receiver: Option<Receiver<Option<Option<String>>>>,
    pub start_time: Option<OffsetDateTime>,
    pub current_time: Option<OffsetDateTime>,
    pub min_pasword_length: usize, // timer:time
    pub max_pasword_length: usize,
    pub custom_charsets: String,
    pub crack: Option<Cracker>,
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
        self.strategy = None;
        match self.mode {
            Mode::PasswordDictionary => {
                if self.dictionary_path.is_some() {
                    let path = Path::new(self.dictionary_path.as_ref().unwrap());
                    let strategy = Strategy::PasswordFile(path.to_path_buf());

                    self.strategy = Some(strategy);
                }
            }
            Mode::Generation => {
                let charsets = self
                    .selected_charset
                    .iter()
                    .filter_map(|&i| if i.1 { Some(i.0.to_charset()) } else { None })
                    .flatten()
                    .collect::<Vec<_>>();
                if charsets.len() != 0 {
                    let strategy = Strategy::GenPasswords {
                        charsets,
                        min_password_len: self.min_pasword_length,
                        max_password_len: self.max_pasword_length,
                    };

                    self.strategy = Some(strategy);
                }
            }
            Mode::Custom => {
                let charsets = self
                    .custom_charsets
                    .split(',')
                    .filter(|f| f.len() == 1)
                    .map(|f| f.chars().next().unwrap())
                    .collect::<Vec<_>>();
                if charsets.len() != 0 {
                    let strategy = Strategy::GenPasswords {
                        charsets,
                        min_password_len: self.min_pasword_length,
                        max_password_len: self.max_pasword_length,
                    };
                    self.strategy = Some(strategy);
                }
            }
        }

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
                            ui.toggle_value(&mut self.selected_charset[0].1, "数字");
                            ui.toggle_value(&mut self.selected_charset[1].1, "小写字母");
                            ui.toggle_value(&mut self.selected_charset[2].1, "大写字母");
                            ui.toggle_value(&mut self.selected_charset[3].1, "特殊字符");
                        });
                        crate::ui::password_length(self, ui);
                    }
                    Mode::Custom => {
                        ui.horizontal(|ui| {
                            ui.label("自定义字符(以引文逗号为分隔符,): ");
                            ui.text_edit_singleline(&mut self.custom_charsets);
                        });
                        crate::ui::password_length(self, ui);
                    }
                }
                ui.end_row();

                ui.add_enabled_ui(self.strategy.is_some() && !self.running, |ui| {
                    if ui.button("开始").clicked() {
                        self.running = true;
                        let file = self.file_path.clone();
                        let strategy = self.strategy.clone();
                        let (send_password_find, receive_password_find) = channel();
                        self.password_receiver = Some(receive_password_find);

                        let work_count = self.workers_count;
                        self.reset();
                        let crack = Cracker::new(file.unwrap(), work_count, strategy.unwrap());

                        self.password_count = crack
                            .count()
                            .map_err(|e| format!("获取密码总数失败：{}", e));
                        self.crack = Some(crack.clone());
                        let crack1 = Arc::new(crack);
                        thread::spawn(move || match crack1.start() {
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
                        });
                    }
                });

                progress_bar(self, ui);

                if self.password_receiver.is_some() {
                    if let Ok(r) = self.password_receiver.as_ref().unwrap().try_recv() {
                        self.find_result = r;
                        self.running = false;
                    }
                }
                if self.crack.is_some() {
                    if self.running {
                        self.current_time = Some(OffsetDateTime::now_utc());
                    }
                    self.tested_count = self.crack.as_ref().unwrap().tested_count() as usize;

                    self.progress =
                        self.tested_count as f32 / *self.password_count.as_ref().unwrap() as f32;
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
            selected_charset: [
                (CharsetChoice::Number, true),
                (CharsetChoice::Lower, false),
                (CharsetChoice::Upper, false),
                (CharsetChoice::Special, false),
            ],
            password_count: Ok(0),
            progress: 0.0,
            strategy: None,
            running: false,
            find_result: None,
            tested_count: 0,
            password_receiver: None,
            start_time: None,
            current_time: None,
            workers_count: num_cpus::get_physical(),
            min_pasword_length: 1,
            max_pasword_length: 8,
            custom_charsets: String::new(),
            crack: None,
        }
    }
}
