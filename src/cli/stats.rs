use crate::cli::sync::setup_workspace;
use crate::error::{Result, WorkOsError};
use crate::plugins::github::GithubPlugin;
use colored::*;

pub async fn run(
    stat_type: String,
    run_mode: String,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<()> {
    let (_, _, registry) = setup_workspace(run_mode, from_date, to_date)?;

    let stats_task = match stat_type.as_str() {
        "ai-code" => {
            println!("{}", "Fetching AI code usage statistics...".dimmed());

            let plugin = registry.get_client("github")?;
            let github_plugin = plugin
                .as_any()
                .downcast_ref::<GithubPlugin>()
                .ok_or_else(|| WorkOsError::Config("Failed to get GitHub plugin".to_string()))?;

            let github_client = github_plugin
                .client()
                .ok_or_else(|| WorkOsError::Config("GitHub client not configured".to_string()))?;

            github_client.get_ai_stats().await?
        }
        _ => {
            return Err(WorkOsError::Config(format!(
                "Unknown stat type: '{}'. Available types: ai-code, productivity",
                stat_type
            )));
        }
    };

    println!();

    if let Some(description) = &stats_task.description {
        println!("{}", description);
    }

    Ok(())
}
