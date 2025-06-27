mod dap_messenger;
pub mod message;

use crate::dap::dap_messenger::DapMessenger;
use crate::dap::message::{InitializeRequestArguments, ProtocolMessage, RequestMessage};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Stdio};
use std::str::Utf8Error;
use std::sync::mpsc::{Receiver, TryRecvError};

#[derive(thiserror::Error, Debug)]
pub enum DapError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
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
    exec_path: PathBuf,
    process: Child,
    last_seq: u64,
    dap_messenger: DapMessenger<ChildStdin>,
    receiver: Receiver<ProtocolMessage>,
}

impl DapInstance {
    pub fn instance(path: impl AsRef<Path>) -> Result<Self, DapError> {
        Self::_instance(path.as_ref())
    }

    fn _instance(path: &Path) -> Result<Self, DapError> {
        let mut process = std::process::Command::new(path)
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
        })
    }

    pub fn next_seq(&mut self) -> u64 {
        self.last_seq += 1;
        self.last_seq
    }

    pub fn launch(&mut self, backend_args_json: String) -> Result<(), DapError> {
        let seq = self.next_seq();
        let message = ProtocolMessage::Request(RequestMessage::Initialize {
            seq,
            arguments: InitializeRequestArguments {
                client_id: Some("memvisor".into()),
                client_name: Some("MemVisor".into()),
                adapter_id: "codelldb".into(),
                ..Default::default()
            },
        });
        
        log::debug!("Initialize message: {message:?}");
        self.send_message(&message)?;
        
        let arguments = serde_json::from_str(&backend_args_json)?;

        let seq = self.next_seq();
        let message = ProtocolMessage::Request(RequestMessage::Launch { seq, arguments });

        log::debug!("Launch message: {message:?}");
        self.send_message(&message)?;

        Ok(())
    }
    
    pub fn send_message(&mut self, msg: &ProtocolMessage) -> Result<(), DapError> {
        self.send_message_json(&serde_json::to_string(msg)?)
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
}
