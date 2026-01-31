use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkOsError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("State error: {0}")]
    State(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("Slack API error: {0}")]
    Slack(String),
}

pub type Result<T> = std::result::Result<T, WorkOsError>;
