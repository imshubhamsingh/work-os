mod client;
pub mod model;

pub use crate::core::plugin::{ConfigField, Plugin, PluginMetadata};
use crate::core::task::Task;
pub use crate::error::Result;
pub use crate::models::config::GitHubConfig;
use async_trait::async_trait;
pub use client::GithubClient;

pub struct GithubPlugin {
    client: Option<GithubClient>,
    config: Option<GitHubConfig>,
}

impl GithubPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(&mut self, config: GitHubConfig) -> Result<()> {
        let client = GithubClient::new(&config)?;
        self.config = Some(config);
        self.client = Some(client);
        Ok(())
    }
}

impl Default for GithubPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GithubPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "github",
            name: "GitHub",
            description: "Fetch PRs, issues, and reviews from GitHub",
            version: "0.1.0",
            icon: "🐙",
        }
    }

    fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                name: "token",
                label: "GitHub access token",
                help: "Create at: https://github.com/settings/tokens",
                required: true,
                secret: true,
            },
            ConfigField {
                name: "username",
                label: "GitHub username",
                help: "Your GitHub username",
                required: true,
                secret: false,
            },
        ]
    }

    async fn test_connection(&self) -> Result<bool> {
        match &self.client {
            Some(client) => client.test_connection().await,
            None => Ok(false),
        }
    }

    async fn fetch_tasks(&self) -> Result<Vec<Task>> {
        match &self.client {
            Some(client) => client.get_all_tasks().await,
            None => Ok(Vec::new()),
        }
    }
}
