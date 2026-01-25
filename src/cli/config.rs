use crate::models::config::*;
use crate::plugins::github::GithubPlugin;
use crate::plugins::slack::SlackPlugin;
use crate::{core::plugin::Plugin, error::Result};
use colored::*;
use std::io::{self, Write};

fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(GithubPlugin::new()),
        Box::new(SlackPlugin::new()),
        // @todo add more plugin here
    ]
}

pub async fn init(plugin_filters: Option<String>) -> Result<()> {
    let mut config = WorkOsConfig::load().unwrap_or_default();
    let plugins = get_all_plugins();

    println!("Work OS Configuration Setup\n");

    for plugin in plugins {
        let meta = plugin.metadata();
        if let Some(ref filter) = plugin_filters {
            if meta.id != filter {
                continue;
            }
        }

        println!("{} {} Configuration", meta.icon, meta.name);
        println!("{}", "-".repeat(40));

        let setup = prompt(&format!("Configure {}? (y/n): ", meta.name))?;
        if setup.to_lowercase() != "y" {
            println!();
            continue;
        }

        match configure_plugin_interactive(&plugin, &mut config).await {
            Ok(()) => {
                println!("{} {} configured successfully!\n", "✔".green(), meta.name);
            }
            Err(e) => {
                println!("{} Failed to configure {}: {}\n", "✖".red(), meta.name, e);
            }
        }
    }
    // let mut config = WorkOsConfig {
    //     github: None,
    //     slack: None,
    //     output: OutputConfig::default(),
    //     sync: SyncConfig::default(),
    // };

    // println!("Github Configuration:");
    // let setup_github = prompt("Configure Github? (y/n): ")?;
    // if setup_github.to_lowercase() == "y" {
    //     let token = prompt("  GitHub Personal Access Token: ")?;
    //     let username = prompt("  GitHub Username: ")?;

    //     config.github = Some(GitHubConfig {
    //         token,
    //         username,
    //         include_repos: Vec::new(),
    //         bots: Vec::new(),
    //     });
    // }

    // println!("Slack Configuration:");
    // let setup_slack = prompt("Configure Slack? (y/n): ")?;
    // if setup_slack.to_lowercase() == "y" {
    //     let token = prompt("  Slack User Token (xoxp-...): ")?;
    //     config.slack = Some(SlackConfig {
    //         token,
    //         keywords: vec![
    //             "todo".to_string(),
    //             "action item".to_string(),
    //             "can you".to_string(),
    //             "please".to_string(),
    //             "urgent".to_string(),
    //             "asap".to_string(),
    //         ],
    //         channels: Vec::new(),
    //         user_groups: Vec::new(),
    //         max_messages_per_channel: 50,
    //     });
    // }

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

    if let Some(ref mut slack) = display_config.slack {
        slack.token = "xoxp-*******************************************".to_string()
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
