use std::fs;
use std::path::PathBuf;
use egui::{CentralPanel, Id, Response, TopBottomPanel, Ui, Widget};
use egui::panel::TopBottomSide;

pub struct FileEntry {
    path: PathBuf,
    is_dir: bool,
}

pub struct FilePicker {
    cwd: PathBuf,
    dir_listing: Vec<FileEntry>,
}

impl FilePicker {
    pub fn new() -> Self {
        let mut new = Self {
            cwd: std::env::current_dir().unwrap_or(PathBuf::new()),
            dir_listing: Vec::new(),
        };
        let _ = new.refresh_directory().ok();

        new
    }

    pub fn set_cwd(&mut self, path: impl Into<PathBuf>) -> Result<(), std::io::Error> {
        self.cwd = path.into();
        self.refresh_directory()
    }

    pub fn refresh_directory(&mut self) -> Result<(), std::io::Error> {
        self.dir_listing.clear();
        let mut dir_data = fs::read_dir(&self.cwd)?;
        while let Some(Ok(entry)) = dir_data.next() {
            self.dir_listing.push(FileEntry {
                path: entry.path(),
                is_dir: entry.metadata().map(|m| m.is_dir()).unwrap_or(false),
            })
        }
        Ok(())
    }
}
impl Widget for &mut FilePicker {
    fn ui(self, ui: &mut Ui) -> Response {
        for entry in &self.dir_listing {
            ui.label(entry.path.to_str().unwrap_or("<invalid_filename>"));
        }

        ui.response()
    }
}
