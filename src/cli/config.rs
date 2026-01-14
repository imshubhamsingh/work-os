use crate::error::Result;
use crate::models::config::*;
use std::io::{self, Write};

pub async fn init() -> Result<()> {
    println!("Work OS Configuration Setup\n");
    let mut config = WorkOsConfig {
        github: None,
        slack: None,
        output: OutputConfig::default(),
        sync: SyncConfig::default(),
    };

    println!("Github Configuration:");
    let setup_github = prompt("Configure Github? (y/n): ")?;
    if setup_github.to_lowercase() == "y" {
        let token = prompt("  GitHub Personal Access Token: ")?;
        let username = prompt("  GitHub Username: ")?;

        config.github = Some(GitHubConfig {
            token,
            username,
            include_orgs: Vec::new(),
            include_repos: Vec::new(),
        });
    }

    println!("Slack Configuration:");
    let setup_slack = prompt("Configure Slack? (y/n): ")?;
    if setup_slack.to_lowercase() == "y" {
        let token = prompt("  Slack User Token (xoxp-...): ")?;
        config.slack = Some(SlackConfig {
            token,
            keywords: vec![
                "todo".to_string(),
                "action item".to_string(),
                "can you".to_string(),
                "please".to_string(),
                "urgent".to_string(),
                "asap".to_string(),
            ],
            channels: Vec::new(),
            max_messages_per_channel: 50,
        });
    }

    config.save()?;

    println!(
        "\n✓ Configuration saved to: {:?}",
        WorkOsConfig::config_path()
    );
    Ok(())
}

pub async fn show() -> Result<()> {
    let config = WorkOsConfig::load()?;
    let mut display_config = config.clone();

    if let Some(ref mut github) = display_config.github {
        github.token = "****hidden****".to_string();
    }

    let toml = toml::to_string_pretty(&display_config).unwrap();
    println!("{}", toml);

    Ok(())
}

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
