use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkOsError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GitHub API error: {0}")]
    GitHub(String),
}

pub type Result<T> = std::result::Result<T, WorkOsError>;