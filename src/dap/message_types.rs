use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum OutputEventCategory {
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
pub enum StoppedEventReason {
    #[serde(rename = "step")]
    Step,
    #[serde(rename = "breakpoint")]
    Breakpoint,
    #[serde(rename = "exception")]
    Exception,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "entry")]
    Entry,
    #[serde(rename = "goto")]
    Goto,
    #[serde(rename = "function breakpoint")]
    FunctionBreakpoint,
    #[serde(rename = "data breakpoint")]
    DataBreakpoint,
    #[serde(rename = "instruction breakpoint")]
    InstructionBreakpoint,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ChecksumAlgorithm {
    #[serde(rename = "MD5")]
    Md5,
    #[serde(rename = "SHA1")]
    Sha1,
    #[serde(rename = "SHA256")]
    Sha256,
    #[serde(rename = "timestamp")]
    Timestamp,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub checksum: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SourcePresentationHint {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "emphasize")]
    Emphasize,
    #[serde(rename = "deemphasize")]
    Deemphasize,
}

/// A [Source] is a descriptor for source code
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Source {
    /// Short name of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The path of the source to be shown in the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// If this value is > 0 then the contents of the source must be retrieved through
    /// the 'source' request even when the path is specified.
    /// This is only valid for one session.
    #[serde(rename = "sourceReference")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<usize>,
    /// A hint for how to present the source in the UI
    /// [SourcePresentationHint::Deemphasize] can be used to indicate that the source is not
    /// available or that it is skipped on stepping.
    #[serde(rename = "presentationHint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<SourcePresentationHint>,
    /// The origin of this source
    /// Ex: 'internal module', 'inline content from source map', maybe some other things idk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// A list of sources that are related to this source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<Source>>,
    /// Additional data that the DAP might want to loop through the client. Leave it intact.
    #[serde(rename = "adapterData")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_data: Option<serde_json::Value>,
    /// The checksums associated with this file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Checksum>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct SourceBreakpoint {
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "hitCondition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    #[serde(rename = "logMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

// We'll do it little by little
/// Set of DAP capabilities. Not all are defined here. Too many
#[derive(Clone, Copy, Deserialize, Serialize, Default, Debug)]
pub struct Capabilities {
    #[serde(rename = "supportsConfigurationDoneRequest")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_configuration_done_request: Option<bool>,

    #[serde(rename = "supportsSingleThreadExecutionRequests")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_single_thread_execution_requests: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SteppingGranularity {
    #[serde(rename = "statement")]
    Statement,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "instruction")]
    Instruction,
}