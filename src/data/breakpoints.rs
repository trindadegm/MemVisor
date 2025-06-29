use egui::ahash::HashMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub enum Breakpoint {
    Source(CodeBreakpoint)
}

impl Breakpoint {
    pub fn on_source(file: impl Into<PathBuf>, lineno: usize) -> Self {
        Self::Source(CodeBreakpoint {
            file: Arc::new(file.into()),
            lineno,
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct CodeBreakpoint {
    // The Arc is so that we can clone this crap. It should never change anyway
    pub file: Arc<PathBuf>,
    pub lineno: usize,
}

/// For each line of the file (usize), we can have a breakpoint
type FileBreakpoints = BTreeMap<usize, CodeBreakpoint>;
/// We protect them to be able to access them from multiple threads
type ProtectedFileBreakpoints = RwLock<FileBreakpoints>;
/// A project has breakpoints of several files
type ProjectBreakpoints = HashMap<PathBuf, ProtectedFileBreakpoints>;

#[derive(Default)]
pub struct BreakpointStore {
    /// And all the breakpoints are also protected
    points: RwLock<ProjectBreakpoints>,
}

impl BreakpointStore {
    pub fn new() -> Self {
        Self {
            points: RwLock::new(HashMap::default()),
        }
    }

    pub fn add(&self, breakpoint: Breakpoint) {
        match breakpoint {
            Breakpoint::Source(code_bp) => {
                let project_breakpoints = self.points.read().unwrap();
                if let Some(file_breakpoints) = project_breakpoints.get(code_bp.file.as_ref()) {
                    let mut w_file_breakpoints = file_breakpoints.write().unwrap();
                    w_file_breakpoints.insert(code_bp.lineno, code_bp);
                } else {
                    drop(project_breakpoints);

                    let file = code_bp.file.as_ref().clone();
                    let mut file_breakpoints: FileBreakpoints = BTreeMap::default();
                    file_breakpoints.insert(code_bp.lineno, code_bp);

                    let mut w_project_breakpoints = self.points.write().unwrap();
                    w_project_breakpoints.insert(file, RwLock::new(file_breakpoints));
                }
            }
        }
    }
    
    pub fn remove(&self, breakpoint: &Breakpoint) -> bool {
        match breakpoint {
            Breakpoint::Source(code_bp) => {
                let project_breakpoints = self.points.read().unwrap();
                if let Some(file_breakpoints) = project_breakpoints.get(code_bp.file.as_ref()) {
                    let mut w_file_breakpoints = file_breakpoints.write().unwrap();
                    w_file_breakpoints.remove(&code_bp.lineno);
                    if w_file_breakpoints.is_empty() {
                        drop(w_file_breakpoints);
                        drop(project_breakpoints);

                        let mut w_project_breakpoints = self.points.write().unwrap();
                        w_project_breakpoints.remove(code_bp.file.as_ref());
                    }
                    return true;
                }
            }
        }

        false
    }
    
    pub fn get_file_breakpoints(&self, file: impl AsRef<Path>, out: &mut Vec<Breakpoint>) {
        self._get_file_breakpoints(file.as_ref(), out);
    }
    
    fn _get_file_breakpoints(&self, file: &Path, out: &mut Vec<Breakpoint>) {
        out.clear();
        
        let project_breakpoints = self.points.read().unwrap();
        if let Some(file_breakpoints) = project_breakpoints.get(file) {
            let file_breakpoints = file_breakpoints.read().unwrap();
            for (_lineno, breakpoint) in file_breakpoints.iter() {
                out.push(Breakpoint::Source(breakpoint.clone()));
            }
        }
    }
    
    pub fn get_files(&self, out: &mut Vec<PathBuf>) {
        out.clear();

        let project_breakpoints = self.points.read().unwrap();
        for path in project_breakpoints.keys() {
            out.push(path.clone());
        }
    }
}
