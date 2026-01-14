use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubClient;
use crate::plugins::slack::SlackClient;

pub async fn test_github() -> Result<()> {
    let config = WorkOsConfig::load()?;
    let github_config = config
        .github
        .ok_or(WorkOsError::Config("GitHub not configured".to_string()))?;
    println!("Testing Github connection...");
    let github_client = GithubClient::new(&github_config)?;
    let success = github_client.test_connection().await?;
    if success {
        println!(
            "✓ GitHub authentication successful for user: {}",
            github_config.username
        );
    } else {
        println!(
            "✗ GitHub connection failed - please check your credentials or username and try again"
        );
    }

    Ok(())
}

pub async fn test_slack() -> Result<()> {
    let config = WorkOsConfig::load()?;
    let slack_config = config.slack.ok_or_else(|| {
        WorkOsError::Config("Slack not configured. Add [slack] section to config.".to_string())
    })?;
    println!("Testing Slack connection...");
    let slack_client = SlackClient::new(&slack_config)?;
    let success = slack_client.test_connection().await?;
    if success {
        println!("✓ Slack authentication successful");
    } else {
        println!("✗ Slack authentication failed");
    }

    Ok(())
}
