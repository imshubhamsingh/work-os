use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct GitHubConfig {
    pub token: String,
    pub username: String,
    pub include_orgs: Vec<String>,
    pub include_repos: Vec<String>,
    pub bots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrDetails {
    pub body: Option<String>,
    pub reviews: Vec<PrReview>,
    pub comments: Vec<PrComment>,
    pub review_comments: Vec<ReviewComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub author: String,
    pub state: ReviewState,
    pub submitted_at: DateTime<Utc>,
    pub body: Option<String>,
}

impl PrReview {
    pub fn truncated_body(&self, max_len: usize) -> Option<String> {
        self.body.as_ref().map(|body| truncate_text(body, max_len))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Pending,
    Dismissed,
}

impl ReviewState {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "APPROVED" => ReviewState::Approved,
            "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
            "COMMENTED" => ReviewState::Commented,
            "PENDING" => ReviewState::Pending,
            "DISMISSED" => ReviewState::Dismissed,
            _ => ReviewState::Commented,
        }
    }
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Approved => write!(f, "Approved"),
            ReviewState::ChangesRequested => write!(f, "Changes Requested"),
            ReviewState::Commented => write!(f, "Commented"),
            ReviewState::Pending => write!(f, "Pending"),
            ReviewState::Dismissed => write!(f, "Dismissed"),
        }
    }
}

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct ReviewCounts {
//     pub approved: u32,
//     pub changes_requested: u32,
//     pub commented: u32,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrComment {
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl PrComment {
    pub fn truncated_body(&self, max_len: usize) -> String {
        truncate_text(&self.body, max_len)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub author: String,
    pub body: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

impl ReviewComment {
    pub fn truncated_body(&self, max_len: usize) -> String {
        truncate_text(&self.body, max_len)
    }
}

/*
 * Utility function for truncating text
 */
fn truncate_text(text: &str, max_len: usize) -> String {
    let text = text.trim();
    let chars = text.chars();
    if chars.clone().count() <= max_len {
        return text.to_string();
    }
    let truncated: String = chars.take(max_len).collect();
    format!("{}...", truncated)
}

#[derive(Debug, Clone)]
pub struct PrCommit {
    pub sha: String,
    pub message: String,
    pub date: DateTime<Utc>,
    pub additions: u64,
    pub deletions: u64,
}

pub enum SearchType {
    Involved,
    Author,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Involved => write!(f, "involves"),
            SearchType::Author => write!(f, "author"),
        }
    }
}
