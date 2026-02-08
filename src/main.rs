mod cli;
mod core;
mod error;
mod generators;
mod models;
mod plugins;

use clap::Parser;
use cli::{Cli, Commands};
use colored::*;

use crate::cli::ConfigAction;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Hello { name } => greet_user(name),
        Commands::Config { action } => match action {
            ConfigAction::Init { plugin } => cli::config::init(plugin).await,
            ConfigAction::Show { plugin } => cli::config::show(plugin).await,
            ConfigAction::Set { plugin, key, value } => {
                cli::config::set(&plugin, &key, &value).await
            }
            ConfigAction::List => cli::config::list().await,
        },
        Commands::Auth { plugin } => cli::auth::test_all_plugin_auth(plugin).await,
        Commands::Sync {
            json,
            plugins,
            markdown,
            mode,
            from,
            to,
        } => cli::sync::run(json, markdown, plugins, mode, from, to).await,
        Commands::Stats {
            r#type,
            mode,
            from,
            to,
        } => cli::stats::run(r#type, mode, from, to).await,
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red().bold(), e);
        std::process::exit(1);
    }
}

fn greet_user(name: String) -> error::Result<()> {
    if name.is_empty() {
        return Err(error::WorkOsError::Config("Name is required".to_string()));
    }

    println!("Hello, {}! 👋", name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::Value;
    #[tokio::test]
    async fn test_http_request() {
        let client = Client::new();
        let response = client
            .get("https://api.github.com/zen")
            .header("User-Agent", "work-os/0.1.0")
            .send()
            .await
            .unwrap();

        let text: String = response.text().await.unwrap();
        println!("GitHub Zen: {}", text);
        assert!(!text.is_empty());
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let client = Client::new();
        let response = client
            .get("https://api.github.com/users/github")
            .header("User-Agent", "work-os/0.1.0")
            .send()
            .await
            .unwrap();

        let user: Value = response.json().await.unwrap();

        println!("GitHub User: {}", user["name"]);
        assert_eq!(user["login"].as_str().unwrap(), "github");
    }
}
