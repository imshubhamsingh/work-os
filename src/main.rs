mod cli;
mod error;
mod models;

use cli::{Cli, Commands, ConfigCommands};
use clap::{Parser};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    match cli.command {
        Commands::Hello { name } => {
            let message = greet_user(name)?;
            println!("{}", message);
            Ok(())
        }
        Commands::Config { command } => match command {
            ConfigCommands::Init => cli::config::init().await,
            ConfigCommands::Show => cli::config::show().await,
        },
    }
}

fn greet_user(name: String) -> error::Result<String> {
    if name.is_empty() {
     return Err(error::WorkOsError::Config("Name is required".to_string()));
    }
    Ok(format!("Hello, {}! 👋", name))
 }

#[cfg(test)]
mod tests {
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_async_basic() {
        println!("Start async task");

        sleep(Duration::from_secs(1)).await;

        println!("Async task completed");
    }

    #[tokio::test]
    async fn test_concurrent_ops() {
        println!("Start two task concurrently");

        let (result1, result2) = tokio::join!(fetch_date("Task 1"), fetch_date("Task 2"));

        println!("Result 1: {} Result 2: {}", result1, result2);

        println!("Concurrent tasks completed");
    }

    async fn fetch_date(task_name: &str) -> String {
        sleep(Duration::from_secs(1)).await;
        format!("{} completed", task_name)
    }
}