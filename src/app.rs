use crate::dap::DapInstance;
use crate::dap::message::{DapEvent, ProtocolMessage, RequestMessage, ResponseMessage};
use crate::data::breakpoints::BreakpointStore;
use crate::widget::SourceListing;
use eframe::{CreationContext, Frame};
use egui::panel::TopBottomSide;
use egui::{Button, Context, Id, PopupCloseBehavior, Widget, popup_below_widget};
use serde_json::json;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

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

pub struct MemVisorApp {
    dap_instance: Option<DapInstance>,
    debugging: bool,
    selected_tab: usize,
    tabs: Vec<AppTab>,
    breakpoints: Rc<BreakpointStore>,
    test: String,
}

impl MemVisorApp {
    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            dap_instance: None,
            debugging: false,
            selected_tab: 0,
            tabs: Vec::new(),
            breakpoints: Rc::new(BreakpointStore::new()),
            test: String::new(),
        }
    }
}
impl eframe::App for MemVisorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.debugging {
            // When debugging, avoid spending too long sleeping
            // Rerender with at least 2 FPS
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        egui::TopBottomPanel::new(TopBottomSide::Top, Id::new("main-header")).show(ctx, |ui| {
            let file_res = ui.button("File");
            let popup_id = Id::new("main-file-popup");

            if file_res.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
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
                                SourceListing::load(Rc::clone(&self.breakpoints), &file)
                            {
                                self.tabs.push(AppTab::Source(listing));
                            }
                        }
                    }
                },
            );
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

            ui.horizontal(|ui| {
                ui.label("This is a test:");
                ui.text_edit_singleline(&mut self.test);
            });

            if ui.button("Start").clicked() {
                self.dap_instance =
                    DapInstance::instance("test_backends/adapter/codelldb.exe").ok();
                if let Some(instance) = &mut self.dap_instance {
                    if let Err(e) = instance.launch(json!({
                        "name": "launch",
                        "type": "lldb",
                        "request": "launch",
                        "program": "C:/Users/Vanderley/Codigos_Gustavo/rose-engine-2/target/debug/game.exe",
                        "cwd": "C:/Users/Vanderley/Codigos_Gustavo/rose-engine-2",
                    }).to_string()) {
                        log::error!("Error: {e}");
                    }
                    self.debugging = true;
                }
            }

            if let Some(instance) = &mut self.dap_instance {
                while let Some(msg) = instance.poll_message() {
                    log::debug!("Received message: {msg:?}");
                    match msg {
                        ProtocolMessage::Response(ResponseMessage::Initialize { success, .. }) => {
                            if success {
                                let seq = instance.next_seq();
                                instance.send_message(
                                    &ProtocolMessage::Request(RequestMessage::ConfigurationDone {
                                        seq,
                                        arguments: None,
                                    }),
                                ).unwrap()
                            } else {
                                log::error!("Failed to initialize DAP");
                            }
                        },
                        ProtocolMessage::Event(DapEvent::Terminated { .. }) => {
                            self.debugging = false;
                        },
                        _ => {},
                    }
                }
            }
        });
    }
}
