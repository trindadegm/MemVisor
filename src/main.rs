use crate::app::MemVisorApp;

mod app;
mod dap;
pub mod widget;
pub mod data;

fn main() {
    env_logger::init();

    log::info!("Log enabled");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MemVisor",
        native_options,
        Box::new(|cc| Ok(Box::new(MemVisorApp::new(cc)))),
    ).unwrap()
}
