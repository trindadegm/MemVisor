use serde::{Deserialize, Serialize};

use crate::data::types::DebugPointer;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Breakpoint {
    pub id: Option<usize>,
    /// If true, then the breakpoint was set, otherwise it is pending, and may or may not be set by
    /// the debugger later. In which case, the debugger might send an update message setting
    /// verified to true.
    pub verified: bool,
    /// This is information about the breakpoint that may be shown to the user. Might explain why a
    /// breakpoint could not be set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Source code file where the breakpoint was set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Line in the source code where the breakpoint was set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Start position of the source range covered by the breakpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// End of the line of the actual range covered by the breakpoint.
    #[serde(rename = "endLine")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    /// End of position of the source range covered by the breakpoint.
    #[serde(rename = "endColumn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
    /// A memory reference to where the breakpoint is set.
    #[serde(rename = "instructionReference")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reference: Option<DebugPointer>,
    /// The offset from the instruction reference. Can be negative.
    #[serde(rename = "offset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<isize>,
    /// An explanation of why a breakpoint could not be verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<BreakpointUnverifiedReason>,
}

/// An explanation of why a breakpoint could not be verified.
#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum BreakpointUnverifiedReason {
    /// The breakpoint might be verified in the future, but the adapter cannot verify it in the
    /// current state.
    #[serde(rename = "pending")]
    Pending,
    /// The breakpoint was not able to be verified, and the adapter does not believe it can be
    /// verified without intervention.
    #[serde(rename = "failed")]
    Failed,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub checksum: String,
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
pub enum PresentationHint {
    #[serde(rename = "arguments")]
    Arguments,
    #[serde(rename = "locals")]
    Locals,
    #[serde(rename = "registers")]
    Registers,
    #[serde(rename = "returnValue")]
    ReturnValue,
    #[serde(other)]
    Unknown,
}

/// A [Scope] is a named container for variables.
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Scope {
    /// Eg: 'Arguments', 'Locals' or 'Registers'. Should be shown in the UI as is.
    name: String,
    /// A hint of how to present this on the UI
    #[serde(rename = "presentationHint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    presentation_hint: Option<PresentationHint>,
    /// A reference to be able to retrieve variables with the variables request.
    #[serde(rename = "variablesReference")]
    variables_reference: u64,
    /// The number of named variables in this scope. The client can use this number for
    /// paging.
    #[serde(rename = "namedVariables")]
    #[serde(skip_serializing_if = "Option::is_none")]
    named_variables: Option<u64>,
    /// The number of indexed variables in this scope. The client can use this number for
    /// paging.
    #[serde(rename = "indexedVariables")]
    #[serde(skip_serializing_if = "Option::is_none")]
    indexed_variables: Option<u32>,
    /// If true, the number of variables in this scope is large or expensive to retrieve.
    expensive: bool,
    /// The source for this scope,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
    /// The start line of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u64>,
    /// Start position of the range covered by the scope (measured in UTF-16, depends on the
    /// client capability config).
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<u64>,
    /// The end line of the range covered by this scope
    #[serde(rename = "endLine")]
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<u64>,
    /// End position of the range covered by the scope (measured in UTF-16, depends on the
    /// client capability config).
    #[serde(rename = "endColumn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<u64>,
}

/// A [Source] is a descriptor for source code
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
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

// TODO: Document
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum SourcePresentationHint {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "emphasize")]
    Emphasize,
    #[serde(rename = "deemphasize")]
    Deemphasize,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum SteppingGranularity {
    #[serde(rename = "statement")]
    Statement,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "instruction")]
    Instruction,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
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

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Variable {
    /// The variable's name. Yay.
    name: String,
    /// The variables's value. This is a bit tricky. It can be multi-line, can be empty.
    /// Is intended to be used when showing the value on the UI.
    value: String,
    /// The type of the variable's value. Typically shown in the UI when hovering over the value.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    var_type: Option<String>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct VariablePresentationHint {
    /// The kind of the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<VariablePresentationHintKind>,
    /// Set of attributes represented as an array.
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<Vec<VariablePresentationHintAttribute>>,
    /// Visibility of the variable.
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<VariablePresentationHintVisibility>,
    /// If true, clients can present teh variable with a UI that supports a specific gesture to
    /// trigger its evaluation. An example is a property based on a getter function, which might
    /// be expensive or have side effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    lazy: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum VariablePresentationHintAttribute {
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "constant")]
    Constant,
    #[serde(rename = "readOnly")]
    ReadOnly,
    #[serde(rename = "rawString")]
    RawString,
    #[serde(rename = "hasObjectId")]
    HasObjectId,
    #[serde(rename = "canHaveObjectId")]
    CanHaveObjectId,
    #[serde(rename = "hasSideEffects")]
    HasSideEffects,
    #[serde(rename = "hasDataBreakpoint")]
    HasDataBreakpoint,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum VariablePresentationHintKind {
    #[serde(rename = "property")]
    Property,
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "data")]
    Data,
    #[serde(rename = "event")]
    Event,
    #[serde(rename = "baseClass")]
    BaseClass,
    #[serde(rename = "innerClass")]
    InnerClass,
    #[serde(rename = "interface")]
    Interface,
    #[serde(rename = "mostDerivedClass")]
    MostDerivedClass,
    #[serde(rename = "virtual")]
    Virtual,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum VariablePresentationHintVisibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "internal")]
    Internal,
    #[serde(rename = "final")]
    Final,
    #[serde(other)]
    Unknown,
}
