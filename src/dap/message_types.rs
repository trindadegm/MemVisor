use serde::{Deserialize, Serialize};

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
    pub name: Option<String>,
    /// The path of the source to be shown in the UI
    pub path: Option<String>,
    /// If this value is > 0 then the contents of the source must be retrieved through
    /// the 'source' request even when the path is specified.
    /// This is only valid for one session.
    #[serde(rename = "sourceReference")]
    pub source_reference: Option<usize>,
    /// A hint for how to present the source in the UI
    /// [SourcePresentationHint::Deemphasize] can be used to indicate that the source is not
    /// available or that it is skipped on stepping.
    #[serde(rename = "presentationHint")]
    pub presentation_hint: Option<SourcePresentationHint>,
    /// The origin of this source
    /// Ex: 'internal module', 'inline content from source map', maybe some other things idk.
    pub origin: Option<String>,
    /// A list of sources that are related to this source.
    pub sources: Option<Vec<Source>>,
    /// Additional data that the DAP might want to loop through the client. Leave it intact.
    #[serde(rename = "adapterData")]
    pub adapter_data: Option<serde_json::Value>,
    /// The checksums associated with this file
    pub checksums: Option<Checksum>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct SourceBreakpoint {
    pub line: usize,
    pub column: Option<usize>,
    pub condition: Option<String>,
    #[serde(rename = "hitCondition")]
    pub hit_condition: Option<String>,
    #[serde(rename = "logMessage")]
    pub log_message: Option<String>,
    pub mode: Option<String>,
}