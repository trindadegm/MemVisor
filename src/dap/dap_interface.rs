use crate::dap::message::{
    DapEvent, NextArguments, ProtocolMessage, RequestMessage, ResponseMessage,
    SetBreakpointsArguments,
    OutputEvent,
};
use crate::dap::{self, message_types};
use crate::dap::{DapError, DapInstance};
use crate::data::breakpoints::{Breakpoint, BreakpointStore};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use crate::dap::message_types::{OutputEventCategory, SteppingGranularity};
use crate::dap::requests::RequestId;

type ProtectedOption<T> = Arc<RwLock<Option<T>>>;

#[derive(Clone, Debug)]
pub enum DebugState {
    NotInitialized,
    Ready,
    Running,
    Paused,
    StoppedAtBreakpoint {
        breakpoint: Breakpoint,
    }
}

pub struct DapInterface {
    instance: ProtectedOption<DapInstance>,
    breakpoints: BreakpointStore,
    debug_state: Mutex<DebugState>,
}

impl DapInterface {
    pub fn new() -> Self {
        Self {
            instance: Default::default(),
            breakpoints: BreakpointStore::new(),
            debug_state: Mutex::new(DebugState::NotInitialized),
        }
    }

    pub fn start_dap<TArgs, TArgStr>(&self, filepath: impl AsRef<Path>, options: TArgs) -> Result<(), DapError>
    where
        TArgs: IntoIterator<Item = TArgStr>,
        TArgStr: AsRef<str>,
    {
        let instance = DapInstance::instance(filepath, options)?;
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

            let mut debug_state = self.debug_state.lock().unwrap();
            *debug_state = DebugState::NotInitialized;
        } else {
            return Err(DapError::NoLoadedTarget);
        }
        Ok(())
    }

    pub fn process_dap_events(&self) -> Result<(), DapError> {
        let mut configuration_done = false;

        {
            let mut instance_w = self.instance.write().unwrap();
            tracy_client::Client::start().message("process_dap_events_instance_w", 0);

            if let Some(dap_instance) = &mut *instance_w {
                while let Some(msg) = dap_instance.poll_message() {
                    log::trace!("Received message: {msg:?}");
                    match msg {
                        ProtocolMessage::Response(ResponseMessage::Initialize {
                            success,
                            body,
                            ..
                        }) => {
                            if success {
                                configuration_done = true;
                                if let Some(cap) = &body {
                                    dap_instance.set_capabilities(*cap);
                                }

                                let mut debug_state = self.debug_state.lock().unwrap();
                                *debug_state = DebugState::Ready;
                            } else {
                                log::error!("Failed to initialize DAP");
                            }
                        }
                        ProtocolMessage::Event(DapEvent::Output { body: OutputEvent {
                            category: Some(category),
                            output,
                        } , .. }) => {
                            match category {
                                OutputEventCategory::Stdout => {
                                    print!("{output}");
                                }
                                OutputEventCategory::Stderr => {
                                    eprint!("{output}");
                                }
                                _ => {
                                    log::info!("OutputEvent ({category:?}) says: {output}");
                                }
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
                    if let Err(e) = dap_interface.flush_pending_launch_requests() {
                        log::error!("Error while flushing pending launch request: {e}");
                    }

                    let seq = dap_interface.next_seq();

                    dap_interface.send_message(&ProtocolMessage::Request(
                        RequestMessage::ConfigurationDone {
                            seq,
                            arguments: Some(serde_json::json!({})),
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
            let breakpoints = list
                .iter()
                .map(|bp| {
                    match bp {
                        Breakpoint::Source(b) => b,
                    }
                })
                .map(|bp| message_types::SourceBreakpoint {
                    line: bp.lineno,
                    ..Default::default()
                })
                .collect();

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
        match breakpoint {
            Breakpoint::Source(code_bp) => {
                self.update_breakpoints_for_file(code_bp.file.as_ref())
            }
        }
    }

    pub fn remove_breakpoint(&self, breakpoint: &Breakpoint) -> Result<(), DapError> {
        self.breakpoints.remove(breakpoint);
        match breakpoint {
            Breakpoint::Source(code_bp) => {
                self.update_breakpoints_for_file(code_bp.file.as_ref())
            }
        }
    }

    pub fn request_next(&self) -> Result<(), DapError> {
        let mut instance_w = self.instance.write().unwrap();
        if let Some(instance) = instance_w.as_mut() {
            let seq = instance.next_seq();

            // If step single thread is supported, we'll use it
            let single_thread = instance
                .get_capabilities()
                .supports_single_thread_execution_requests;

            instance.send_message(&ProtocolMessage::Request(RequestMessage::Next {
                seq,
                arguments: NextArguments {
                    single_thread,
                    stepping_granularity: Some(SteppingGranularity::Statement),
                    ..Default::default()
                },
            }))
        } else {
            Err(DapError::NoDapInstance)
        }
    }

    pub fn request_variables(&self) -> Result<RequestId, DapError> {
        let mut instance_w = self.instance.write().unwrap();
        if let Some(instance) = instance_w.as_mut() {
            let seq = instance.next_seq();

            // If step single thread is supported, we'll use it
            let single_thread = instance
                .get_capabilities()
                .supports_single_thread_execution_requests;

            instance.send_message(&ProtocolMessage::Request(RequestMessage::Next {
                seq,
                arguments: NextArguments {
                    single_thread,
                    stepping_granularity: Some(SteppingGranularity::Statement),
                    ..Default::default()
                },
            }))?;

            Ok(RequestId::new(seq))
        } else {
            Err(DapError::NoDapInstance)
        }
    }
    
    pub fn get_debug_state(&self) -> DebugState {
        self.debug_state.lock().unwrap().clone()
    }
}
impl Default for DapInterface {
    fn default() -> Self {
        Self::new()
    }
}
unsafe impl Sync for DapInterface {}
unsafe impl Send for DapInterface {}
