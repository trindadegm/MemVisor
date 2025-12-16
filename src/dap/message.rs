use serde::{Deserialize, Serialize};

use super::message_types::*;

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
        arguments: InitializeArguments,
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
    /// This launch request is sent from the client to the debug adapter to start
    /// the debuggee with or without debugging (if noDebug is true).
    ///
    /// This is unspecified, depending on the debugger
    #[serde(rename = "launch")]
    Launch {
        seq: u64,
        arguments: serde_json::Value,
    },
    #[serde(rename = "next")]
    Next {
        seq: u64,
        arguments: NextArguments,
    },
    Scopes {
        seq: u64,
        arguments: ScopesArguments,
    },
    /// Sets multiple breakpoints for a single source and clears all previous breakpoints in
    /// that source.
    ///
    /// To clear all breakpoint for a source, specify an empty array.
    #[serde(rename = "setBreakpoints")]
    SetBreakpoints {
        seq: u64,
        arguments: SetBreakpointsArguments,
    },
    Variables {
        seq: u64,
        arguments: VariablesArguments,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<Capabilities>,
    },
    #[serde(rename = "notStopped")]
    NotStopped,
    #[serde(rename = "scopes")]
    Scopes {
        seq: u64,
        request_req: u64,
        success: bool,
        body: ScopesResponseBody,
    },
    #[serde(rename = "variables")]
    Variables {
        seq: u64,
        request_req: u64,
        success: bool,
        body: VariablesResponseBody,
    },
    #[serde(rename="setBreakpoints")]
    SetBreakpoints {
        seq: u64,
        request_seq: u64,
        success: bool,
        body: SetBreakpointsResponseBody,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelArguments {
    #[serde(rename = "requestId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
    #[serde(rename = "progressId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_id: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InitializeArguments {
    #[serde(rename = "clientID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(rename = "clientName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    #[serde(rename = "adapterID")]
    pub adapter_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct NextArguments {
    /// Specifies the thread to resume execution for one step
    #[serde(rename = "threadId")]
    pub thread_id: u64,
    /// If this flag is true, all other suspended threads are not resumed
    #[serde(rename = "singleThread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// Stepping granularity. If none is specified, a default of [SteppingGranularity::Statement]
    /// is assumed.
    #[serde(rename = "steppingGranularity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stepping_granularity: Option<SteppingGranularity>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ScopesArguments {
    /// Id of the stack frame to retrieve scope.
    #[serde(rename = "frameId")]
    pub frame_id: u64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ScopesResponseBody {
    pub scopes: Vec<Scope>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SetBreakpointsArguments {
    /// The source location of the breakpoint. Either `path` or `source_reference` must be
    /// specified.
    pub source: Source,
    /// The code locations of the breakpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
    /// Indicates that the underlying source code has been modified.
    #[serde(rename = "sourceModified")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_modified: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SetBreakpointsResponseBody {
    pub breakpoints: Vec<Breakpoint>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct VariablesArguments {
    /// The variable for which to retrieve it's children
    #[serde(rename = "variablesReference")]
    pub variables_reference: u64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct VariablesResponseBody {
    pub variables: Vec<Variable>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event")]
pub enum DapEvent {
    #[serde(rename = "breakpoint")]
    Breakpoint {
        seq: u64,
        body: BreakpointEvent,
    },
    #[serde(rename = "output")]
    Output {
        seq: u64,
        body: OutputEvent,
    },
    #[serde(rename = "stopped")]
    Stopped {
        seq: u64,
        body: StoppedEvent,
    },
    #[serde(rename = "terminated")]
    Terminated {
        seq: u64,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BreakpointEvent {
    pub reason: BreakpointEventReason,
    pub breakpoint: Breakpoint,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakpointEventReason {
    #[serde(rename = "changed")]
    Changed,
    #[serde(rename = "new")]
    New,
    #[serde(rename = "removed")]
    Removed,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OutputEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<OutputEventCategory>,
    pub output: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StoppedEvent {
    /// The reason for the stoppage
    pub reason: StoppedEventReason,
    /// A description of the stoppage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The thread which was stopped
    #[serde(rename = "threadId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<u64>,
    /// Hint that the client should not change focus
    #[serde(rename = "preserveFocusHint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_focus_hint: Option<bool>,
    /// Additional information. Ex: If the reason is exception, contains the exception name,
    /// to show it on the UI.
    #[serde(rename = "text")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// If true, then all threads have been stopped
    #[serde(rename = "allThreadsStopped")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_stopped: Option<bool>,
    /// A list of the breakpoints that triggered the event
    #[serde(rename = "hitBreakpointIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_breakpoint_ids: Option<Vec<u64>>,
}

#[cfg(test)]
mod tests {
    use crate::dap::message::{CancelArguments, ProtocolMessage, RequestMessage};

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
        assert!(arguments.get("progressId").is_none());
    }

    #[test]
    fn test_serialize_response() {
    }
}
