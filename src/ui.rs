use crate::dap::dap_interface::DapInterface;
use crate::widget::SourceListing;
use egui::panel::TopBottomSide;
use egui::{Button, Context, Id, PopupCloseBehavior, Widget, popup_below_widget};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum AppTab {
    Source(SourceListing),
}

impl AppTab {
    pub fn title(&self) -> String {
        match self {
            AppTab::Source(source) => source.filename().into(),
        }
    }

    pub fn widget(&mut self) -> impl Widget {
        match self {
            AppTab::Source(source) => source,
        }
    }
}

const RENDER_TIME_NUM_SAMPLES: u32 = 10;

pub struct MemVisorUi {
    debugging: bool,
    selected_tab: usize,
    tabs: Vec<AppTab>,

    last_render_t: Instant,
    render_time_acc: Duration,
    render_time_avg: Duration,
    num_render_time_samples: u32,
}

impl MemVisorUi {
    pub fn new() -> Self {
        Self {
            debugging: false,
            selected_tab: 0,
            tabs: Vec::new(),

            last_render_t: Instant::now(),
            render_time_acc: Duration::new(0, 0),
            render_time_avg: Duration::new(0, 0),
            num_render_time_samples: 0,
        }
    }

    pub fn update(&mut self, ctx: &Context, dap_interface: Arc<DapInterface>) {
        let _span = tracy_client::span!("ui_update");

        egui::TopBottomPanel::new(TopBottomSide::Top, Id::new("main-header")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let file_res = ui.button("File");
                let popup_id = Id::new("main-file-popup");

                if file_res.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }

                if ui.button("Start").clicked() {
                    let res = dap_interface.load_target("test_backends/adapter/codelldb.exe");
                    if res.is_ok() {
                        if let Err(e) = dap_interface.launch(json!({
                        "name": "launch",
                        "type": "lldb",
                        "request": "launch",
                        "program": "C:/Users/Vanderley/Codigos_Gustavo/rose-engine-2/target/debug/game.exe",
                        "cwd": "C:/Users/Vanderley/Codigos_Gustavo/rose-engine-2",
                    }).to_string()) {
                            log::error!("Error: {e}");
                        } else {
                            self.debugging = true;
                        }
                    }
                }
                
                if ui.button("Step").clicked() {
                    dap_interface.request_next().expect("TODO remove this panic");
                }

                popup_below_widget(
                    ui,
                    popup_id,
                    &file_res,
                    PopupCloseBehavior::CloseOnClick,
                    |ui| {
                        ui.set_min_width(120.0);
                        if ui.add(Button::new("Open").frame(false)).clicked() {
                            let file = rfd::FileDialog::new()
                                .set_directory(std::env::current_dir().unwrap_or(PathBuf::new()))
                                .pick_file();
                            if let Some(file) = file {
                                if let Ok(listing) =
                                    SourceListing::load(Arc::clone(&dap_interface), &file)
                                {
                                    self.tabs.push(AppTab::Source(listing));
                                }
                            }
                        }
                    },
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(self.selected_tab == i, tab.title()).clicked() {
                        self.selected_tab = i;
                    }
                }
            });

            let first_listing = self.tabs.get_mut(self.selected_tab);
            if let Some(tab) = first_listing {
                ui.add(tab.widget());
            }
        });

        self.render_time_acc += self.last_render_t.elapsed();
        self.num_render_time_samples += 1;
        if self.num_render_time_samples >= RENDER_TIME_NUM_SAMPLES {
            self.render_time_avg = self.render_time_acc / self.num_render_time_samples;
            self.num_render_time_samples = 0;
            self.render_time_acc = Duration::new(0, 0);
        }
        self.last_render_t = Instant::now();
        egui::TopBottomPanel::new(TopBottomSide::Bottom, Id::new("main-footer")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.render_time_avg.as_millis() < 10000 {
                    ui.label(format!(
                        "Average frame time: {}ms",
                        self.render_time_avg.as_millis()
                    ));
                } else {
                    ui.label("Average frame time: :D");
                }
            });
        });
    }
}
