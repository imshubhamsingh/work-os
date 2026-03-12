use core::str;

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub source: String,
    pub message_type: MessageType,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub priority: Priority,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub people: Vec<Person>,
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageMetadata {
    GitHub(GitHubMetadata),
    // Slack(SlackMetadata),
    // Jira(JiraMetadata),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMetadata {
    pub repo: String,
    pub number: u64,
    pub state: String,
    pub comments: u32,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    PullRequest,
    Issue,
    Review,
    Message,
    Ticket,
    Statistics,
    MOM,
    Canvas,
    Coralogix,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Open,
    InProgress,
    Blocked,
    Done,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub username: String,
    pub role: PersonRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PersonRole {
    Author,
    Assignee,
    Reviewer,
    Mentioned,
}

impl Message {
    pub fn new(source: &str, message_type: MessageType, id: &str, title: String, url: String) -> Self {
        let now = Utc::now();
        Self {
            id: format!("{}:{}:{}", source, message_type.short_name(), id),
            source: source.to_string(),
            message_type,
            title,
            description: None,
            url,
            priority: Priority::Unknown,
            status: MessageStatus::Open,
            created_at: now,
            updated_at: now,
            due_date: None,
            people: Vec::new(),
            metadata: MessageMetadata::None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_date(mut self, created: DateTime<Utc>, updated: DateTime<Utc>) -> Self {
        self.created_at = created;
        self.updated_at = updated;
        self
    }

    pub fn with_person(mut self, person: Person) -> Self {
        self.people.push(person);
        self
    }

    pub fn with_status(mut self, status: MessageStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn format_absolute_time(date: DateTime<Utc>) -> String {
        date.with_timezone(&Local).format("%b %d, %l:%M %p").to_string()
    }
}

impl MessageType {
    pub fn short_name(&self) -> &str {
        match self {
            MessageType::PullRequest => "pr",
            MessageType::Issue => "issue",
            MessageType::Review => "review",
            MessageType::Message => "message",
            MessageType::Ticket => "ticket",
            MessageType::Statistics => "statistics",
            MessageType::MOM => "mom",
            MessageType::Canvas => "canvas",
            MessageType::Coralogix => "coralogix",
            MessageType::Other(name) => name,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Critical => write!(f, "Critical"),
            Priority::High => write!(f, "High"),
            Priority::Medium => write!(f, "Medium"),
            Priority::Low => write!(f, "Low"),
            Priority::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Open => write!(f, "Open"),
            MessageStatus::InProgress => write!(f, "In Progress"),
            MessageStatus::Blocked => write!(f, "Blocked"),
            MessageStatus::Done => write!(f, "Done"),
            MessageStatus::Other(s) => write!(f, "{}", s),
        }
    }
}
