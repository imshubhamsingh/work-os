use crate::core::plugin::Plugin;
use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;
use crate::models::date_range::DateRange;
use crate::models::state::WorkOsState;
use crate::plugins::github::GithubPlugin;

pub async fn run(stat_type: &str, markdown: bool) -> Result<()> {
    let config = WorkOsConfig::load()?;
    let state = WorkOsState::load()?;

    let range = DateRange::resolve_date_range(None, None, "today", &state)?;
    DateRange::init(range);

    match stat_type {
        "code-ai-usage" => code_ai_usage(&config, markdown).await,
        _ => {
            eprintln!("Unknown stat type: {}", stat_type);
            eprintln!("Available types: code-ai-usage");
            Err(WorkOsError::Config(format!(
                "Unknown stat type: {}",
                stat_type
            )))
        }
    }
}

async fn code_ai_usage(config: &WorkOsConfig, markdown: bool) -> Result<()> {
    // Get GitHub plugin
    let mut github_plugin = GithubPlugin::new();

    if let Some(github_config) = config.plugins.get("github") {
        github_plugin.configure_from_values(&github_config.values)?;
    } else {
        eprintln!("GitHub plugin not configured. Run: work-os config init github");
        return Ok(());
    }

    // Check if configured
    if !github_plugin.is_configured() {
        eprintln!("GitHub plugin not configured. Run: work-os config init github");
        return Ok(());
    }

    println!("Analyzing AI usage from GitHub PRs...\n");

    // Fetch AI stats directly
    match github_plugin.fetch_ai_stats().await {
        Ok(task) => {
            if markdown {
                println!("{}", task.title);
                if let Some(desc) = &task.description {
                    println!("\n{}", desc);
                }
            } else {
                println!("{}", task.title);
                if let Some(desc) = &task.description {
                    println!("\n{}", desc);
                }
            }
        }
        Err(e) => {
            eprintln!("Error generating AI stats: {}", e);
            println!("\nNo AI usage statistics available.");
            println!("Make sure you have PRs in the current date range.");
        }
    }

    Ok(())
}
