use serde::{Deserialize, Serialize};

use crate::core::task::{Priority, TaskStatus};

#[derive(serde::Deserialize)]
pub struct JiraSearchResponse {
    pub issues: Vec<JiraIssue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub fields: JiraFields,
    #[serde(rename = "self")]
    pub self_url: String,
    pub changelog: Option<JiraChangelog>,
}

impl JiraIssue {
    pub fn url(&self, domain: &str) -> String {
        format!("https://{}/browse/{}", domain, self.key)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraFields {
    pub summary: String,
    pub status: JiraStatus,
    pub priority: Option<JiraPriority>,
    pub assignee: Option<JiraUser>,
    pub reporter: Option<JiraUser>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "duedate")]
    pub due_date: Option<String>,
    #[serde(rename = "issuetype")]
    pub issue_type: JiraIssueType,
    pub project: JiraProject,
    pub description: Option<serde_json::Value>, // this is in atlassian document format
    #[serde(default)]
    pub resolution: Option<JiraResolution>,
    pub comment: Option<JiraComments>,
    #[serde(flatten)]
    pub custom_fields: Option<serde_json::Value>,
    pub parent: Option<JiraParentIssue>,
}

impl JiraFields {
    pub fn extract_sprint(&self) -> Option<JiraSprint> {
        let custom = self.custom_fields.as_ref()?;

        // Try common sprint field names
        for field_name in ["customfield_10020", "customfield_10007", "sprint"] {
            if let Some(sprints) = custom.get(field_name) {
                // Sprint is usually an array, get the active/first one
                if let Some(arr) = sprints.as_array() {
                    for sprint_val in arr {
                        if let Ok(sprint) = serde_json::from_value::<JiraSprint>(sprint_val.clone())
                        {
                            if sprint.state == "active" {
                                return Some(sprint);
                            }
                        }
                    }
                    // If no active, return first
                    if let Some(first) = arr.first() {
                        return serde_json::from_value(first.clone()).ok();
                    }
                }
            }
        }
        None
    }

    /// Extract epic from parent or custom field
    pub fn extract_epic(&self) -> Option<(String, String)> {
        // Next-gen projects: epic is the parent
        if let Some(ref parent) = self.parent {
            if let Some(ref fields) = parent.fields {
                if let Some(ref issue_type) = fields.issue_type {
                    if issue_type.name.to_lowercase() == "epic" {
                        let name = fields.summary.clone().unwrap_or_default();
                        return Some((parent.key.clone(), name));
                    }
                }
            }
        }

        // Classic projects: epic link is a custom field
        if let Some(ref custom) = self.custom_fields {
            for field_name in ["customfield_10014", "customfield_10008", "epicLink"] {
                if let Some(epic_key) = custom.get(field_name).and_then(|v| v.as_str()) {
                    return Some((epic_key.to_string(), String::new()));
                }
            }
        }

        None
    }

    pub fn extract_parent_task(&self) -> Option<(String, String)> {
        if let Some(ref parent) = self.parent {
            if let Some(ref fields) = parent.fields {
                if let Some(ref issue_type) = fields.issue_type {
                    if issue_type.name.to_lowercase() == "task" {
                        let name = fields.summary.clone().unwrap_or_default();
                        return Some((parent.key.clone(), name));
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraStatus {
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub status_category: Option<JiraStatusCategory>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraStatusCategory {
    pub key: String, // "new", "indeterminate", "done"
    pub name: String,
}

impl JiraStatusCategory {
    pub fn map_status_category(status_category: Option<&JiraStatusCategory>) -> TaskStatus {
        match status_category.map(|c| c.key.as_str()) {
            Some("new") => TaskStatus::Open,                 // To Do
            Some("indeterminate") => TaskStatus::InProgress, // In Progress
            Some("done") => TaskStatus::Done,                // Done
            _ => TaskStatus::Open,                           // Fallback
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraPriority {
    pub id: String,
    pub name: String,
}

impl JiraPriority {
    pub fn map_priority(jira_priority: Option<&str>) -> Priority {
        match jira_priority.map(|p| p.to_lowercase()).as_deref() {
            Some("highest") | Some("blocker") => Priority::Critical,
            Some("high") => Priority::High,
            Some("medium") | Some("normal") => Priority::Medium,
            Some("low") | Some("lowest") => Priority::Low,
            _ => Priority::Medium,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraUser {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
}

impl JiraUser {
    pub fn unknown(account_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            display_name: "Unknown User".to_string(),
            email_address: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraIssueType {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub subtask: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraProject {
    pub id: String,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraResolution {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraMyselfResponse {
    // #[serde(rename = "accountId")]
    // pub account_id: String,
    // #[serde(rename = "displayName")]
    // pub display_name: String,
    // #[serde(rename = "emailAddress")]
    // pub email_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraSprint {
    pub id: u64,
    pub name: String,
    pub state: String, // "active", "closed", "future"
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

impl JiraSprint {
    pub fn display(&self) -> String {
        match self.state.as_str() {
            "active" => format!("{} (Active)", self.name),
            "future" => format!("{} (Future)", self.name),
            _ => self.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraComments {
    pub comments: Vec<JiraComment>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraComment {
    pub id: String,
    pub author: JiraUser,
    pub created: String,
    pub updated: String,
    pub body: Option<serde_json::Value>, // ADF
}

impl JiraComment {
    pub fn body_text(&self) -> String {
        self.body
            .as_ref()
            .map(|adf| extract_text_from_adf(adf))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraParentIssue {
    pub id: String,
    pub key: String,
    pub fields: Option<JiraParentFields>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraParentFields {
    pub summary: Option<String>,
    #[serde(rename = "issuetype")]
    pub issue_type: Option<JiraIssueType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraChangelog {
    pub histories: Vec<JiraChangelogEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraChangelogEntry {
    pub id: String,
    pub author: JiraUser,
    pub created: String, // this is an iso timestamp
    pub items: Vec<JiraChangeItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraChangeItem {
    pub field: String, // like "status", "assignee", "priority"
    #[serde(rename = "fieldtype")]
    pub field_type: String, //like  "jira"
    #[serde(rename = "fromString")]
    pub from_string: Option<String>,
    #[serde(rename = "toString")]
    pub to_string: Option<String>,
}

impl JiraChangelogEntry {
    pub fn summary(&self) -> String {
        let changes: Vec<String> = self
            .items
            .iter()
            .map(
                |item| match (item.from_string.as_ref(), item.to_string.as_ref()) {
                    (Some(from), Some(to)) => format!("{}: {} → {}", item.field, from, to),
                    (None, Some(to)) => format!("{}: set to {}", item.field, to),
                    (Some(from), None) => format!("{}: cleared (was {})", item.field, from),
                    (None, None) => format!("{}: changed", item.field),
                },
            )
            .collect();
        changes.join(", ")
    }
}

// ============================
// Utils
// ============================

fn extract_text_from_adf(adf: &serde_json::Value) -> String {
    let mut text = String::new();
    extract_text_recursive(adf, &mut text);
    text.trim().to_string()
}

fn extract_text_recursive(node: &serde_json::Value, output: &mut String) {
    match node {
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::String(t)) = obj.get("text") {
                output.push_str(t);
            }
            if let Some(serde_json::Value::Array(content)) = obj.get("content") {
                for child in content {
                    extract_text_recursive(child, output);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_text_recursive(item, output);
            }
        }
        _ => {}
    }
}
