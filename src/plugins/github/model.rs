use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrComment {
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub author: String,
    pub body: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewCounts {
    pub approved: u32,
    pub changes_requested: u32,
    pub commented: u32,
}

impl ReviewCounts {
    pub fn from_reviews(reviews: &[PrReview]) -> Self {
        let mut counts = ReviewCounts::default();
        for review in reviews {
            match review.state {
                ReviewState::Approved => counts.approved += 1,
                ReviewState::ChangesRequested => counts.changes_requested += 1,
                ReviewState::Commented => counts.commented += 1,
                _ => {}
            }
        }
        counts
    }
}
