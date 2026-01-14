use clap::{Parser, Subcommand};
pub mod config;
pub mod auth;
pub mod sync;

#[derive(Parser)]
#[command(name = "work-os")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    Hello {
        #[arg(short, long)]
        name: String,
    },

    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    Sync {
        #[arg(long)]
        json: bool,

        #[arg(long, value_delimiter = ',')]
        plugins: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Init,
    Show,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    Github,
    Slack,
}