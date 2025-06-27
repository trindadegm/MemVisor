use crate::data::breakpoints::{Breakpoint, BreakpointStore};
use egui::{Color32, Response, ScrollArea, Ui, Widget};
use epaint::text::{LayoutJob, TextWrapMode};
use std::ffi::OsStr;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct SourceCode {
    path: PathBuf,
    content: String,
}

pub struct SourceListing {
    source_code: SourceCode,
    breakpoint_store: Rc<BreakpointStore>,
    list_breakpoints: Vec<Breakpoint>,
}

impl SourceListing {
    pub fn load(
        breakpoints: Rc<BreakpointStore>,
        path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        Self::_load(breakpoints, path.as_ref())
    }

    fn _load(breakpoints: Rc<BreakpointStore>, path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            source_code: SourceCode {
                path: path.into(),
                content,
            },
            breakpoint_store: breakpoints,
            list_breakpoints: Vec::new(),
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
        self.breakpoint_store
            .get_file_breakpoints(&self.source_code.path, &mut self.list_breakpoints);
        //let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.set_min_width(300.0);

        //let layout_job = egui_extras::syntax_highlighting::highlight(
        //    ui.ctx(),
        //    ui.style(),
        //    &theme,
        //    &self.source_code.content,
        //    self.source_code
        //        .path
        //        .extension()
        //        .and_then(OsStr::to_str)
        //        .unwrap_or("txt"),
        //);

        ScrollArea::both().show(ui, |ui| {
            for (line_index, line) in self.source_code.content.lines().enumerate() {
                let has_breakpoint = self
                    .list_breakpoints
                    .iter()
                    .find(|b| b.lineno == line_index + 1);
                ui.horizontal(|ui| {
                    if ui.selectable_label(has_breakpoint.is_some(), "O").clicked() {
                        self.breakpoint_store.add(Breakpoint {
                            file: self.source_code.path.clone(),
                            lineno: line_index + 1,
                        });
                    }
                    ui.add(
                        egui::Label::new(egui::RichText::new(line).monospace())
                            .wrap_mode(TextWrapMode::Extend),
                    );
                });
            }
        });

        ui.response()
    }
}
