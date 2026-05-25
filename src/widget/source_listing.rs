use crate::dap::dap_interface::{DapInterface, DebugState};
use crate::data::breakpoints::Breakpoint;
use egui::{Response, ScrollArea, Ui, UiBuilder, Widget};
use epaint::FontId;
use epaint::text::LayoutJob;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SourceCode {
    path: PathBuf,
    #[allow(unused)]
    content: String,
}

const DEFAULT_LINE_HEIGHT_PX: f32 = 12.0;

pub struct SourceListing {
    dap_interface: Arc<DapInterface>,
    source_code: SourceCode,
    list_breakpoints: Vec<Breakpoint>,
    lines: Vec<String>,
    line_height_px: f32,

    scroll_event_target: Option<usize>,
    last_debug_highlighted_line: usize,
}

impl SourceListing {
    pub fn load(
        dap_interface: Arc<DapInterface>,
        path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        Self::_load(dap_interface, path.as_ref())
    }

    fn _load(dap_interface: Arc<DapInterface>, path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let lines = content.lines().map(String::from).collect();
        Ok(Self {
            dap_interface,
            source_code: SourceCode {
                path: path.into(),
                content,
            },
            list_breakpoints: Vec::new(),
            lines,
            line_height_px: DEFAULT_LINE_HEIGHT_PX,

            scroll_event_target: None,
            last_debug_highlighted_line: 0,
        })
    }

    pub fn filename(&self) -> &str {
        self.source_code
            .path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("<unknown>")
    }
}
impl Widget for &mut SourceListing {
    fn ui(self, ui: &mut Ui) -> Response {
        let _span = tracy_client::span!("ui_update_source_listing");

        let debug_state = self.dap_interface.get_debug_state();
        let mut stopped_at_line = None;

        match debug_state {
            DebugState::Stopped {
                file: Some(file),
                lineno,
                ..
            } if file == self.source_code.path => {
                stopped_at_line = lineno;
            }
            DebugState::Paused => {}
            _ => {}
        }

        self.dap_interface
            .get_file_breakpoints(&self.source_code.path, &mut self.list_breakpoints);

        ui.set_width(ui.available_width());

        let fresh_scroll_event = if let Some(lineno) = stopped_at_line
            && lineno != self.last_debug_highlighted_line
        {
            self.last_debug_highlighted_line = lineno;
            self.scroll_event_target = Some(lineno - 1);
            true
        } else {
            false
        };

        ScrollArea::both().show_rows(ui, self.line_height_px, self.lines.len(), |ui, range| {
            ui.set_width(ui.available_width());

            let lines_in_range = &self.lines[range.clone()];

            let scroll_target_index = match self.scroll_event_target {
                Some(scroll_final_target) if range.contains(&scroll_final_target) => {
                    self.scroll_event_target = None;
                    // If the line is in range THE MOMENT the scroll event happens, we don't trigger scrolling
                    // Otherwise we will keep scrolling all the way to it, not only until it gets "in range".
                    // This is because an item can be in range and not exactly be showing. This is not
                    // perfect but it is good enough
                    if fresh_scroll_event {
                        None
                    } else {
                        Some(scroll_final_target)
                    }
                }
                Some(scroll_final_target) => {
                    if scroll_final_target < range.start {
                        Some(range.start)
                    } else {
                        Some(range.end - 1)
                    }
                }
                None => None,
            };

            for (i, line) in lines_in_range.iter().enumerate() {
                let line_index = i + range.start;
                let lineno = line_index + 1;

                let line_breakpoint = self
                    .list_breakpoints
                    .iter()
                    .map(|b| match b {
                        Breakpoint::Source(b) => b,
                    })
                    .find(|b| b.lineno == lineno);

                let has_breakpoint = line_breakpoint.is_some();

                let job = LayoutJob::simple_singleline(
                    line.clone(),
                    FontId::monospace(self.line_height_px),
                    ui.style().visuals.widgets.active.fg_stroke.color,
                );

                egui::Frame::new()
                    .fill(
                        if stopped_at_line
                            .map(|stopped_lineno| stopped_lineno == lineno)
                            .unwrap_or(false)
                        {
                            egui::Color32::from_rgb(24, 32, 72)
                        } else {
                            ui.style().visuals.window_fill
                        },
                    )
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_width(ui.available_width());
                            let set_bp_res = ui.add_sized(
                                [self.line_height_px, self.line_height_px],
                                egui::Button::selectable(has_breakpoint, "O"),
                            );
                            if set_bp_res.clicked() {
                                let dap_result = if let Some(bp) = line_breakpoint {
                                    self.dap_interface
                                        .remove_breakpoint(&Breakpoint::Source(bp.clone()))
                                } else {
                                    let path = self.source_code.path.clone();
                                    self.dap_interface
                                        .put_breakpoint(Breakpoint::on_source(path, line_index + 1))
                                };

                                if let Err(e) = dap_result {
                                    log::error!("{e}");
                                }
                            }
                            ui.horizontal(|ui| {
                                ui.set_width(30.0);
                                ui.with_layout(
                                    egui::Layout::right_to_left(Default::default()),
                                    |ui| {
                                        ui.label(lineno.to_string());
                                    },
                                );
                            });
                            let job_res = ui.label(job);
                            if let Some(scroll_target_index) = scroll_target_index
                                && line_index == scroll_target_index
                            {
                                log::debug!("SCROLL TO {line_index} {}", line);
                                job_res.scroll_to_me_animation(
                                    Some(egui::Align::Center),
                                    egui::style::ScrollAnimation::none(),
                                );
                            }
                        });
                    });
            }
        });

        ui.response()
    }
}
