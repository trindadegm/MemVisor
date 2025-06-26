use eframe::{CreationContext, Frame};
use egui::Context;

pub struct MemVisorApp {
    test: String,
}

impl MemVisorApp {
    pub fn new(cc: &CreationContext) -> Self {
        Self {
            test: String::new(),
        }
    }
}
impl eframe::App for MemVisorApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MemVisor");

            ui.horizontal(|ui| {
                ui.label("This is a test:");
                ui.text_edit_singleline(&mut self.test);
            });
        });
    }
}