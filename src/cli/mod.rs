use clap::{Parser, Subcommand};
pub mod config;

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
    }
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Init,
    Show,
}