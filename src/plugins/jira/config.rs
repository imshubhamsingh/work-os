use crate::error::WorkOsError;
use crate::{core::message::Priority, error::Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub domain: String,
    pub email: String,
    pub token: String,
    #[serde(default)]
    pub filters: Vec<JqlFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JqlFilter {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub jql: String,
    #[serde(default = "default_medium")]
    pub priority: String,
}

fn default_true() -> bool {
    true
}

fn default_medium() -> String {
    "medium".to_string()
}

impl JiraConfig {
    pub fn parse_filter(value: &toml::Value) -> Result<JqlFilter> {
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("Filter missing 'name'".into()))?
            .to_string();

        let jql = value
            .get("jql")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("Filter missing 'jql'".into()))?
            .to_string();

        let enabled = value
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let priority = value
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium")
            .to_string();

        Ok(JqlFilter {
            name,
            enabled,
            jql,
            priority,
        })
    }
}

impl JqlFilter {
    pub fn priority_enum(&self) -> Priority {
        match self.priority.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "low" => Priority::Low,
            _ => Priority::Medium,
        }
    }
}
