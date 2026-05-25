use crate::dap::message::{
    BreakpointEvent, BreakpointEventReason, ContinueArguments, DapEvent, NextArguments,
    OutputEvent, ProtocolMessage, RequestMessage, ResponseMessage, SetBreakpointsArguments,
    StackTraceArguments,
};
use crate::dap::message_types::{
    self, OutputEventCategory, SteppingGranularity, StoppedEventReason,
};
use crate::dap::requests::RequestId;
use crate::dap::{DapError, DapInstance};
use crate::data::breakpoints::{Breakpoint, BreakpointStore, CodeBreakpoint};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

type ProtectedOption<T> = Arc<RwLock<Option<T>>>;

pub enum ContinueMode {
    AllThreads,
    SingleThread(u64),
}

#[derive(Clone, Default, Debug)]
pub enum DebugState {
    #[default]
    NotInitialized,
    Ready,
    Running,
    Paused,
    Stopped {
        thread_id: Option<u64>,
        breakpoint: Option<Breakpoint>,
        file: Option<PathBuf>,
        lineno: Option<usize>,
        /// The seq id of the request for "stackFrames" as a result of a "Step" breakpoint
        /// This can be used to update breakpoint information by the stackFrame data
        step_stackframe_request_seq: Option<u64>,
    },
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

    pub fn start_dap<TArgs, TArgStr>(
        &self,
        filepath: impl AsRef<Path>,
        options: TArgs,
    ) -> Result<(), DapError>
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
                        ProtocolMessage::Response(ResponseMessage::SetBreakpoints {
                            success,
                            body,
                            ..
                        }) => {
                            if success {
                                for breakpoint in body.breakpoints.iter().cloned() {
                                    log::debug!("Confirming addition of breakpoint {breakpoint:?}");
                                    self.breakpoints.add_breakpoint_data(breakpoint);
                                }
                            } else {
                                log::error!("Failed to set breakpoints to DAP")
                            }
                        }
                        ProtocolMessage::Response(ResponseMessage::StackTrace {
                            success,
                            body,
                            request_seq,
                            ..
                        }) => {
                            if success {
                                if let Some(frame) = body.stack_frames.first() {
                                    let mut state = self.debug_state.lock().unwrap();

                                    // We check to see if we were stopped and if we had a pending
                                    // request for the stack frame in order to complete the stop
                                    // state information. In that case, we use the top frame in
                                    // order to feed file and line number information about the
                                    // stopped status.
                                    if let DebugState::Stopped {
                                        thread_id,
                                        breakpoint,
                                        step_stackframe_request_seq,
                                        ..
                                    } = &*state
                                        && step_stackframe_request_seq
                                            .map(|seq| seq == request_seq)
                                            .unwrap_or(false)
                                    {
                                        *state = DebugState::Stopped {
                                            thread_id: *thread_id,
                                            breakpoint: breakpoint.clone(),
                                            file: frame
                                                .source
                                                .as_ref()
                                                .and_then(|source| source.path.as_ref())
                                                .map(PathBuf::from),
                                            lineno: if frame.line > 0 {
                                                Some(frame.line)
                                            } else {
                                                None
                                            },
                                            step_stackframe_request_seq: None,
                                        };
                                    }
                                }

                                for frame in body.stack_frames.iter() {
                                    log::debug!("Received stack frame: {frame:?}");
                                }
                            } else {
                                log::error!("Failed to query stack frames from DAP");
                            }
                        }
                        ProtocolMessage::Event(DapEvent::Output {
                            body:
                                OutputEvent {
                                    category: Some(category),
                                    output,
                                },
                            ..
                        }) => match category {
                            OutputEventCategory::Stdout => {
                                print!("{output}");
                            }
                            OutputEventCategory::Stderr => {
                                eprint!("{output}");
                            }
                            _ => {
                                log::info!("OutputEvent ({category:?}) says: {output}");
                            }
                        },
                        ProtocolMessage::Event(DapEvent::Breakpoint {
                            body:
                                BreakpointEvent {
                                    reason: BreakpointEventReason::New,
                                    breakpoint,
                                },
                            ..
                        }) => {
                            log::debug!("Confirming addition of breakpoint {breakpoint:?}");
                            self.breakpoints.add_breakpoint_data(breakpoint);
                        }
                        ProtocolMessage::Event(DapEvent::Breakpoint {
                            body:
                                BreakpointEvent {
                                    reason: BreakpointEventReason::Changed,
                                    breakpoint,
                                },
                            ..
                        }) => {
                            log::debug!("Breakpoint updated {breakpoint:?}");
                            self.breakpoints.update_breakpoint_data(breakpoint);
                        }
                        ProtocolMessage::Event(DapEvent::Breakpoint {
                            body:
                                BreakpointEvent {
                                    reason: BreakpointEventReason::Removed,
                                    breakpoint: message_types::Breakpoint { id: Some(id), .. },
                                },
                            ..
                        }) => {
                            log::debug!("Breakpoint of id {id} removed");
                            self.breakpoints.delete_breakpoint_data(id);
                        }
                        ProtocolMessage::Event(DapEvent::Stopped { body, .. }) => {
                            let stack_trace_req_seq_id = if let Some(thread_id) = body.thread_id {
                                let seq_id = dap_instance.next_seq();
                                let msg = ProtocolMessage::Request(RequestMessage::StackTrace {
                                    seq: seq_id,
                                    arguments: StackTraceArguments {
                                        thread_id,
                                        // start_frame: Some(0),
                                        // TODO: this should be configured somewhere by the
                                        // user
                                        levels: Some(2),
                                        ..Default::default()
                                    },
                                });

                                dap_instance.send_message(&msg)?;

                                Some(seq_id)
                            } else {
                                log::warn!("Stopped at some unknown thread");

                                None
                            };

                            match body.reason {
                                StoppedEventReason::Breakpoint
                                | StoppedEventReason::FunctionBreakpoint => {
                                    let hit_breakpoint = body
                                        .hit_breakpoint_ids
                                        .and_then(|list| list.first().copied());
                                    let breakpoint = hit_breakpoint.and_then(|b| {
                                        self.breakpoints.get_breakpoint_for_dap_id(b)
                                    });

                                    let mut debug_state = self.debug_state.lock().unwrap();
                                    match &breakpoint {
                                        Some(Breakpoint::Source(CodeBreakpoint {
                                            file,
                                            lineno,
                                            ..
                                        })) => {
                                            let file = file.as_ref().clone();
                                            let lineno = *lineno;

                                            *debug_state = DebugState::Stopped {
                                                thread_id: body.thread_id,
                                                breakpoint,
                                                file: Some(file),
                                                lineno: Some(lineno),
                                                step_stackframe_request_seq: None,
                                            };
                                        }
                                        None => {}
                                    }
                                }
                                StoppedEventReason::Step => {
                                    let mut debug_state = self.debug_state.lock().unwrap();
                                    let state = &*debug_state;
                                    match state {
                                        // If it was stopped before, let's not try to change
                                        // anything besides the thread id, wait for the stack
                                        // frames to arrive to then make further updates
                                        DebugState::Stopped {
                                            breakpoint,
                                            file,
                                            lineno,
                                            ..
                                        } => {
                                            *debug_state = DebugState::Stopped {
                                                thread_id: body.thread_id,
                                                breakpoint: breakpoint.clone(),
                                                file: file.clone(),
                                                lineno: *lineno,
                                                step_stackframe_request_seq: stack_trace_req_seq_id,
                                            };
                                        }
                                        // If the state is not Stopped, then we need to set it to
                                        // stopped anyway, even if we don't know what caused it
                                        _ => {
                                            *debug_state = DebugState::Stopped {
                                                thread_id: body.thread_id,
                                                breakpoint: None,
                                                file: None,
                                                lineno: None,
                                                step_stackframe_request_seq: stack_trace_req_seq_id,
                                            };
                                        }
                                    }
                                }
                                _ => {
                                    log::warn!("Stopped for some unknown reason");
                                    let mut debug_state = self.debug_state.lock().unwrap();
                                    let state = &*debug_state;
                                    match state {
                                        // If it was stopped before, let's not try to change
                                        // anything besides the thread id
                                        DebugState::Stopped {
                                            breakpoint,
                                            file,
                                            lineno,
                                            ..
                                        } => {
                                            *debug_state = DebugState::Stopped {
                                                thread_id: body.thread_id,
                                                breakpoint: breakpoint.clone(),
                                                file: file.clone(),
                                                lineno: *lineno,
                                                step_stackframe_request_seq: None,
                                            };
                                        }
                                        // If the state is not Stopped, then we need to set it to
                                        // stopped anyway, even if we don't know what caused it
                                        _ => {
                                            *debug_state = DebugState::Stopped {
                                                thread_id: body.thread_id,
                                                breakpoint: None,
                                                file: None,
                                                lineno: None,
                                                step_stackframe_request_seq: None,
                                            };
                                        }
                                    }
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
                .map(|bp| match bp {
                    Breakpoint::Source(b) => b,
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
            Breakpoint::Source(code_bp) => self.update_breakpoints_for_file(code_bp.file.as_ref()),
        }
    }

    pub fn remove_breakpoint(&self, breakpoint: &Breakpoint) -> Result<(), DapError> {
        self.breakpoints.remove(breakpoint);
        match breakpoint {
            Breakpoint::Source(code_bp) => self.update_breakpoints_for_file(code_bp.file.as_ref()),
        }
    }

    pub fn request_next(&self) -> Result<(), DapError> {
        let thread_id = {
            let debug_state = self.get_debug_state();
            if let DebugState::Stopped { thread_id, .. } = debug_state {
                thread_id.unwrap_or(0)
            } else {
                0
            }
        };

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
                    thread_id,
                    single_thread,
                    stepping_granularity: Some(SteppingGranularity::Line),
                },
            }))
        } else {
            Err(DapError::NoDapInstance)
        }
    }

    pub fn request_continue(&self, mode: ContinueMode) -> Result<(), DapError> {
        let (thread_id, single_thread) = match mode {
            ContinueMode::AllThreads => (0, false),
            ContinueMode::SingleThread(thread_id) => (thread_id, true),
        };

        let mut instance_w = self.instance.write().unwrap();
        if let Some(instance) = instance_w.as_mut() {
            let seq = instance.next_seq();

            instance.send_message(&ProtocolMessage::Request(RequestMessage::Continue {
                seq,
                arguments: ContinueArguments {
                    thread_id,
                    single_thread: Some(single_thread),
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
