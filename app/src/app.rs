use eframe::egui;

#[derive()]
pub struct App {
    pub can_exit: bool,
    pub is_exiting: bool,
}
impl eframe::App for App {
    fn on_exit_event(&mut self) -> bool {
        self.is_exiting = true;
        self.can_exit
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // custom_window_frame(tx, ctx, frame, "egui with custom frame", |ui| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("hello egui");
            });
        });

        if self.is_exiting {
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Not yet").clicked() {
                            self.is_exiting = false;
                        }
                        if ui.button("Yes!").clicked() {
                            self.can_exit = true;
                            frame.quit()
                        }
                    })
                });
        }
    }
}
impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            can_exit: false,
            is_exiting: false,
        }
    }
}
