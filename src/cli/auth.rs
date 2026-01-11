use crate::models::config::WorkOsConfig;
use crate::error::{Result, WorkOsError};
use crate::plugins::github::GithubClient;


pub async fn test_github() -> Result<()> {
    let config = WorkOsConfig::load()?;
    let github_config = config.github.ok_or(WorkOsError::Config("GitHub not configured".to_string()))?;
    println!("Testing Github connection...");
    let github_client = GithubClient::new(&github_config)?;
    let success = github_client.test_connection().await?;
    if success {
        println!("✓ GitHub authentication successful for user: {}", github_config.username);
    } else {
        println!("✗ GitHub connection failed - please check your credentials or username and try again");
    }

    Ok(())
}