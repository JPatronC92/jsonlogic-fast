use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleEnvelope {
    pub id: String,
    pub assert: Value,
    pub message: String,
    pub path: Option<String>,
    #[serde(default = "default_severity")]
    pub severity: Severity,
    #[serde(default = "default_retryable")]
    pub retryable: bool,
}

fn default_severity() -> Severity {
    Severity::Error
}

fn default_retryable() -> bool {
    true
}
