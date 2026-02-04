use core::str;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub source: String,
    pub task_type: TaskType,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub priority: Priority,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub people: Vec<Person>,
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
pub enum TaskType {
    PullRequest,
    Issue,
    Review,
    Message,
    Ticket,
    Statistics,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
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

impl Task {
    pub fn new(source: &str, task_type: TaskType, id: &str, title: String, url: String) -> Self {
        let now = Utc::now();
        Self {
            id: format!("{}:{}:{}", source, task_type.short_name(), id),
            source: source.to_string(),
            task_type,
            title,
            description: None,
            url,
            priority: Priority::Unknown,
            status: TaskStatus::Open,
            created_at: now,
            updated_at: now,
            due_date: None,
            people: Vec::new(),
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

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }
}

impl TaskType {
    pub fn short_name(&self) -> &str {
        match self {
            TaskType::PullRequest => "pr",
            TaskType::Issue => "issue",
            TaskType::Review => "review",
            TaskType::Message => "message",
            TaskType::Ticket => "ticket",
            TaskType::Statistics => "statistics",
            TaskType::Other(name) => name,
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

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Open => write!(f, "Open"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Blocked => write!(f, "Blocked"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Other(s) => write!(f, "{}", s),
        }
    }
}
