mod dap_messenger;
pub mod message;
pub mod dap_interface;
pub mod message_types;
pub mod requests;

use crate::dap::dap_messenger::DapMessenger;
use crate::dap::message::{InitializeArguments, ProtocolMessage, RequestMessage};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Stdio};
use std::str::Utf8Error;
use std::sync::mpsc::{Receiver, TryRecvError};
use crate::dap::message_types::Capabilities;

#[derive(thiserror::Error, Debug)]
pub enum DapError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("DAP instance not connected")]
    NoDapInstance,
    #[error("There is no loaded target")]
    NoLoadedTarget,
    #[error("Failed to get DAP process stdin")]
    NoStdin,
    #[error("Failed to get DAP process stdout")]
    NoStdout,
    #[error("JSON encoding error: {0}")]
    JsonEncodingError(#[from] serde_json::Error),
    #[error("Could not decode message header: {0}")]
    BadMessageHeader(String),
    #[error("Could not parse content length: {0}")]
    InvalidContentLength(String),
    #[error("Failed to decode string because of invalid UTF-8")]
    BadCharacterEncoding(#[from] Utf8Error),
}

pub struct DapInstance {
    #[allow(unused)]
    exec_path: PathBuf,
    #[allow(unused)]
    process: Child,

    last_seq: u64,
    dap_messenger: DapMessenger<ChildStdin>,
    receiver: Receiver<ProtocolMessage>,
    capabilities: Capabilities,

    pending_launch_req: Option<serde_json::Value>,
}

impl DapInstance {
    pub fn instance<TArgs, TArgStr>(path: impl AsRef<Path>, options: TArgs) -> Result<Self, DapError>
    where
        TArgs: IntoIterator<Item = TArgStr>,
        TArgStr: AsRef<str>,
    {
        let args: Vec<String> = options.into_iter().map(|s| s.as_ref().into()).collect();
        Self::_instance(path.as_ref(), &args)
    }

    fn _instance(path: &Path, args: &[String]) -> Result<Self, DapError> {
        log::info!("Launching debugger {path:?} with arguments {args:?}");
        let mut process = std::process::Command::new(path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().ok_or(DapError::NoStdin)?;
        let stdout = process.stdout.take().ok_or(DapError::NoStdout)?;

        let (tx, rx) = std::sync::mpsc::sync_channel(10);
        let dap_messenger = DapMessenger::new(BufReader::new(stdout), stdin, tx);

        Ok(Self {
            exec_path: path.into(),
            process,
            last_seq: 0,
            dap_messenger,
            receiver: rx,
            capabilities: Capabilities::default(),
            pending_launch_req: None,
        })
    }

    pub fn next_seq(&mut self) -> u64 {
        self.last_seq += 1;
        self.last_seq
    }

    pub fn launch(&mut self, backend_args_json: &str) -> Result<(), DapError> {
        let seq = self.next_seq();
        let message = ProtocolMessage::Request(RequestMessage::Initialize {
            seq,
            arguments: InitializeArguments {
                client_id: Some("memvisor".into()),
                client_name: Some("MemVisor".into()),
                adapter_id: "rust-gdb".into(),
                ..Default::default()
            },
        });

        log::debug!("Initialize message: {message:?}");
        self.send_message(&message)?;

        let arguments = serde_json::from_str(backend_args_json)?;

        self.pending_launch_req = Some(arguments);

        Ok(())
    }

    pub fn send_message(&mut self, msg: &ProtocolMessage) -> Result<(), DapError> {
        self.send_message_json(&serde_json::to_string(msg)?)
    }

    pub fn flush_pending_launch_requests(&mut self) -> Result<(), DapError> {
        if let Some(launch_req) = self.pending_launch_req.take() {
            let seq = self.next_seq();
            let message = ProtocolMessage::Request(RequestMessage::Launch { seq, arguments: launch_req });

            return self.send_message(&message);
        }

        Ok(())
    }

    pub fn poll_message(&mut self) -> Option<ProtocolMessage> {
        match self.receiver.try_recv() {
            Ok(v) => Some(v),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::error!("Can't poll message: messenger disconnected");
                None
            }
        }
    }

    fn send_message_json(&mut self, msg: &str) -> Result<(), DapError> {
        self.dap_messenger.send_message(msg)
    }

    pub fn get_capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    pub fn set_capabilities(&mut self, cap: Capabilities) {
        self.capabilities = cap;
    }
}
