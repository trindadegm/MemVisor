use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    #[serde(rename = "request")]
    Request(RequestMessage),
    #[serde(rename = "response")]
    Response(ResponseMessage),
    #[serde(rename = "event")]
    Event(DapEvent),
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
pub enum RequestMessage {
    #[serde(rename = "cancel")]
    Cancel {
        seq: u64,
        arguments: Option<CancelArguments>,
    },
    /// The initialize request is sent as the first request from the client to the debug
    /// adapter in order to configure it with client capabilities and to retrieve capabilities
    /// from the debug adapter.
    ///
    /// Until the debug adapter has responded with an initialize response, the client must
    /// not send any additional requests or events to the debug adapter.
    ///
    /// In addition the debug adapter is not allowed to send any requests or events to the
    /// client until it has responded with an initialize response.
    ///
    /// The initialize request may only be sent once.
    #[serde(rename = "initialize")]
    Initialize {
        seq: u64,
        arguments: InitializeRequestArguments,
    },
    /// The attach request is sent from the client to the debug adapter to attach
    /// to a debuggee that is already running.
    ///
    /// This is unspecified, depending on the debugger
    #[serde(rename = "attach")]
    Attach {
        seq: u64,
        arguments: serde_json::Value,
    },
    /// This launch request is sent from the client to the debug adapter to start
    /// the debuggee with or without debugging (if noDebug is true).
    ///
    /// This is unspecified, depending on the debugger
    #[serde(rename = "launch")]
    Launch {
        seq: u64,
        arguments: serde_json::Value,
    },
    /// This request indicates that the client has finished initialization of the debug adapter.
    ///
    /// So it is the last request in the sequence of configuration requests (which was
    /// started by the initialized event).
    ///
    /// Clients should only call this request if the corresponding capability
    /// supportsConfigurationDoneRequest is true.
    #[serde(rename = "configurationDone")]
    ConfigurationDone {
        seq: u64,
        /// Just send None for now.
        arguments: Option<serde_json::Value>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command")]
pub enum ResponseMessage {
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "initialize")]
    Initialize {
        seq: u64,
        request_seq: u64,
        success: bool,
    },
    #[serde(rename = "notStopped")]
    NotStopped,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelArguments {
    #[serde(rename = "requestId")]
    pub request_id: Option<u64>,
    #[serde(rename = "progressId")]
    pub progress_id: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InitializeRequestArguments {
    #[serde(rename = "clientID")]
    pub client_id: Option<String>,
    #[serde(rename = "clientName")]
    pub client_name: Option<String>,
    #[serde(rename = "adapterID")]
    pub adapter_id: String,
    pub locale: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event")]
pub enum DapEvent {
    #[serde(rename = "output")]
    Output {
        seq: u64,
        body: DapEventOutput,
    },
    #[serde(rename = "terminated")]
    Terminated {
        seq: u64,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum DapEventOutputCategory {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "important")]
    Important,
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "stderr")]
    Stderr,
    #[serde(rename = "telemetry")]
    Telemetry,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DapEventOutput {
    category: Option<DapEventOutputCategory>,
    output: String,
}

#[cfg(test)]
mod tests {
    use crate::dap::message::{CancelArguments, ProtocolMessage, RequestMessage, ResponseMessage};

    #[test]
    fn test_serialize_request() {
        let msg = ProtocolMessage::Request(RequestMessage::Cancel {
            seq: 100,
            arguments: Some(CancelArguments {
                progress_id: None,
                request_id: Some(4),
            }),
        });

        let encoded = serde_json::to_string(&msg).unwrap();
        eprintln!("{encoded}");

        let val: serde_json::Value = serde_json::from_str(&encoded).unwrap();

        assert_eq!(val.get("seq").unwrap().as_u64().unwrap(), 100);
        assert_eq!(val.get("type").unwrap().as_str().unwrap(), "request");
        assert_eq!(val.get("command").unwrap().as_str().unwrap(), "cancel");

        let arguments = val.get("arguments").unwrap();

        assert_eq!(arguments.get("requestId").unwrap().as_u64().unwrap(), 4);
        assert!(arguments.get("progressId").unwrap().is_null());
    }

    #[test]
    fn test_serialize_response() {
    }
}
