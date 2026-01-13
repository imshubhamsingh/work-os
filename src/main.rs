mod cli;
mod core;
mod error;
mod models;
mod plugins;

use clap::Parser;
use cli::{AuthCommands, Cli, Commands, ConfigCommands};
use colored::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Hello { name } => greet_user(name),
        Commands::Config { command } => match command {
            ConfigCommands::Init => cli::config::init().await,
            ConfigCommands::Show => cli::config::show().await,
        },
        Commands::Auth { command } => match command {
            AuthCommands::Github => cli::auth::test_github().await,
        },
        Commands::Sync { json, plugins } => cli::sync::run(json, plugins).await,
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
