use clap::{Parser, Subcommand};
pub mod auth;
pub mod config;
pub mod stats;
pub mod sync;

#[derive(Parser)]
#[command(name = "work-os")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Hello {
        #[arg(short, long)]
        name: String,
    },

    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    Auth {
        plugin: Option<String>,

        /// Force re-authentication even if a valid token exists (for OAuth2 plugins)
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    Sync {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        markdown: bool,

        #[arg(long, value_delimiter = ',')]
        plugins: Option<Vec<String>>,

        #[arg(long, default_value = "today")]
        mode: String,

        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        to: Option<String>,
    },

    Stats {
        #[arg(long, default_value = "ai-code")]
        r#type: String,

        #[arg(long, default_value = "today")]
        mode: String,

        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        to: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Init {
        plugin: Option<String>,
    },

    Show {
        plugin: Option<String>,
    },

    Set {
        plugin: String,
        key: String,
        value: String,
    },

    List,
}
