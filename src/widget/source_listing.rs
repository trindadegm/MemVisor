use crate::dap::dap_interface::DapInterface;
use crate::data::breakpoints::Breakpoint;
use egui::{Response, ScrollArea, Ui, Widget};
use epaint::FontId;
use epaint::text::LayoutJob;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SourceCode {
    path: PathBuf,
    content: String,
}

const DEFAULT_LINE_HEIGHT_PX: f32 = 12.0;

pub struct SourceListing {
    dap_interface: Arc<DapInterface>,
    source_code: SourceCode,
    list_breakpoints: Vec<Breakpoint>,
    lines: Vec<String>,
    line_height_px: f32,
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

        self.dap_interface
            .get_file_breakpoints(&self.source_code.path, &mut self.list_breakpoints);

        ui.set_width(ui.available_width());

        ScrollArea::both().show_rows(ui, self.line_height_px, self.lines.len(), |ui, range| {
            ui.set_width(ui.available_width());
            
            let lines_in_range = &self.lines[range.clone()];
            
            for (i, line) in lines_in_range.iter().enumerate() {
                let line_index = i + range.start;
                
                let line_breakpoint = self.list_breakpoints
                    .iter()
                    .find(|b| b.lineno == line_index + 1);
                
                let has_breakpoint = line_breakpoint.is_some();
                
                let job = LayoutJob::simple_singleline(
                    line.clone(),
                    FontId::monospace(self.line_height_px),
                    ui.style().visuals.widgets.active.fg_stroke.color,
                );
                ui.horizontal(|ui| {
                    let set_bp_res = ui.add_sized(
                        [self.line_height_px, self.line_height_px],
                        egui::SelectableLabel::new(has_breakpoint, "O"),
                    );
                    if set_bp_res.clicked() {
                        if let Some(bp) = line_breakpoint {
                            self.dap_interface.remove_breakpoint(&bp);
                        } else {
                            let path = self.source_code.path.clone();
                            self.dap_interface.put_breakpoint(Breakpoint {
                                file: path,
                                lineno: line_index + 1,
                            });
                        }
                    } 
                    ui.label(job);
                });
            }
        });

        ui.response()
    }
}
