use crate::dap::message::{DapEvent, ProtocolMessage, RequestMessage, ResponseMessage, SetBreakpointsArguments};
use crate::dap::{DapError, DapInstance};
use crate::dap::message_types;
use crate::data::breakpoints::{Breakpoint, BreakpointStore};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

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
        tracy_client::Client::start().message("load_target_instance_w", 0);

        *w_dap = Some(instance);
        Ok(())
    }

    pub fn launch(&self, launch_json: impl AsRef<str>) -> Result<(), DapError> {
        let mut w_dap = self.instance.write().unwrap();
        tracy_client::Client::start().message("launch_instance_w", 0);
        if let Some(w_dap) = &mut *w_dap {
            w_dap.launch(launch_json.as_ref())?;
        } else {
            return Err(DapError::NoLoadedTarget)
        }
        Ok(())
    }

    pub fn process_dap_events(&self) -> Result<(), DapError> {
        let mut configuration_done = false;
        
        {
            let mut instance_w = self.instance.write().unwrap();
            tracy_client::Client::start().message("process_dap_events_instance_w", 0);

            if let Some(dap_interface) = &mut *instance_w {
                while let Some(msg) = dap_interface.poll_message() {
                    log::debug!("Received message: {msg:?}");
                    match msg {
                        ProtocolMessage::Response(ResponseMessage::Initialize { success, body, .. }) => {
                            if success {
                                configuration_done = true;
                                if let Some(cap) = &body {
                                    dap_interface.set_capabilities(*cap);
                                }
                            } else {
                                log::error!("Failed to initialize DAP");
                            }
                        }
                        ProtocolMessage::Event(DapEvent::Terminated { .. }) => {}
                        _ => {}
                    }
                }
            }
        }
        
        if configuration_done {
            self.update_all_breakpoints()?;

            {
                let mut instance_w = self.instance.write().unwrap();
                if let Some(dap_interface) = instance_w.as_mut() {
                    let seq = dap_interface.next_seq();
                    dap_interface
                        .send_message(&ProtocolMessage::Request(
                            RequestMessage::ConfigurationDone {
                                seq,
                                arguments: None,
                            },
                        ))?;
                }
            }
        }
        
        Ok(())
    }

    pub fn update_all_breakpoints(&self) -> Result<(), DapError> {
        let mut files = Vec::new();
        self.breakpoints.get_files(&mut files);

        for file in &files {
            self.update_breakpoints_for_file(file)?
        }

        Ok(())
    }

    fn update_breakpoints_for_file(&self, file: &Path) -> Result<(), DapError> {
        let mut instance_w = self.instance.write().unwrap();
        tracy_client::Client::start().message("update_breakpoints_for_file_instance_w", 0);
        if let Some(instance) = instance_w.as_mut() {
            let mut list = Vec::new();
            self.breakpoints.get_file_breakpoints(file, &mut list);
            let source = message_types::Source {
                path: Some(file.to_string_lossy().into()),
                ..Default::default()
            };
            let breakpoints = list.iter().map(|bp| {
                message_types::SourceBreakpoint {
                    line: bp.lineno,
                    ..Default::default()
                }
            }).collect();

            let seq = instance.next_seq();
            let msg = ProtocolMessage::Request(RequestMessage::SetBreakpoints {
                seq,
                arguments: SetBreakpointsArguments {
                    source,
                    breakpoints: Some(breakpoints),
                    ..Default::default()
                },
            });

            instance.send_message(&msg)?;
        }

        Ok(())
    }

    pub fn get_files_with_breakpoints(&self, out: &mut Vec<PathBuf>) {
        self.breakpoints.get_files(out);
    }

    pub fn get_file_breakpoints(&self, file: impl AsRef<Path>, out: &mut Vec<Breakpoint>) {
        self.breakpoints.get_file_breakpoints(file, out);
    }

    pub fn put_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), DapError> {
        self.breakpoints.add(breakpoint.clone());
        self.update_breakpoints_for_file(&breakpoint.file)
    }

    pub fn remove_breakpoint(&self, breakpoint: &Breakpoint) -> Result<(), DapError> {
        self.breakpoints.remove(&breakpoint);
        self.update_breakpoints_for_file(&breakpoint.file)
    }
    
    pub fn request_next(&self) {
        let mut instance_w = self.instance.write().unwrap();
        if let Some(instance) = instance_w.as_mut() {
            //instance.send_message(ProtocolMessage::Request())
        }
    }
}
