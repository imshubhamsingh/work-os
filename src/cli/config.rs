use crate::models::config::*;
use crate::error::Result;
use std::io::{self, Write};

pub async fn init() -> Result<()> {
    println!("Work OS Configuration Setup\n");
    let mut config = WorkOsConfig {
        github: None,
        output: OutputConfig::default(),
        sync: SyncConfig::default(),
    };

    println!("Github Configuration:");
    let token = prompt("Github Personal Access Token: ")?;
    let username = prompt("Github Username: ")?;

    config.github = Some(GitHubConfig { token, username });

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