use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::dap::{DapError, DapInstance};
use crate::data::breakpoints::{Breakpoint, BreakpointStore};

type ProtectedOption<T> = Arc<RwLock<Option<T>>>;

pub struct DapInterface {
    instance: ProtectedOption<DapInstance>,
    breakpoints: BreakpointStore,
}

impl DapInterface {
    pub fn new() -> Self {
        Self {
            instance: Default::default(),
            breakpoints: BreakpointStore::new(),
        }
    }
    
    pub fn load_target(&self, filepath: impl AsRef<Path>) -> Result<(), DapError> {
        let instance = DapInstance::instance(filepath)?;
        let mut w_dap = self.instance.write().unwrap();
        *w_dap = Some(instance);
        Ok(())
    }
    
    pub fn launch(&self, launch_json: impl AsRef<str>) -> Result<(), DapError> {
        let mut w_dap = self.instance.write().unwrap();
        if let Some(w_dap) = &mut *w_dap {
            w_dap.launch(launch_json.as_ref())?;
            Ok(())
        } else {
            Err(DapError::NoLoadedTarget)
        }
    }
    
    pub fn put_breakpoint(&self, breakpoint: Breakpoint) {
        self.breakpoints.add(breakpoint.clone());
    }
    
    pub fn remove_breakpoint(&self, breakpoint: &Breakpoint) {
        self.breakpoints.remove(&breakpoint);
    }
    
    pub fn get_files_with_breakpoints(&self, out: &mut Vec<PathBuf>) {
        self.breakpoints.get_files(out);
    }
    
    pub fn get_file_breakpoints(&self, file: impl AsRef<Path>, out: &mut Vec<Breakpoint>) {
        self.breakpoints.get_file_breakpoints(file, out);
    }
}
//if let Some(dap_interface) = dap_instance {
// while let Some(msg) = dap_interface.poll_message() {
// log::debug!("Received message: {msg:?}");
// match msg {
// ProtocolMessage::Response(ResponseMessage::Initialize { success, .. }) => {
// if success {
// let seq = dap_interface.next_seq();
// dap_interface.send_message(
// &ProtocolMessage::Request(RequestMessage::ConfigurationDone {
// seq,
// arguments: None,
// }),
// ).unwrap()
// } else {
// log::error!("Failed to initialize DAP");
// }
// }
// ProtocolMessage::Event(DapEvent::Terminated { .. }) => {
// self.debugging = false;
// }
// _ => {}
// }
// }
// }
