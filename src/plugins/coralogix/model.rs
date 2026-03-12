use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoralogixLog {
    pub logid: String,
    pub timestamp: String,
    pub application_name: String,
    pub severity: Severity,
    pub body: String,
    pub error: String,
    pub service: String,
    pub trace_id: String,
    pub span_id: String,
    pub environment: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Debug,
    Verbose,
    Info,
    Warning,
    Error,
    Critical,
}

impl Severity {
    pub fn from_num(n: u8) -> Self {
        match n {
            1 => Severity::Debug,
            2 => Severity::Verbose,
            3 => Severity::Info,
            4 => Severity::Warning,
            5 => Severity::Error,
            6 => Severity::Critical,
            _ => Severity::Error,
        }
    }
}
