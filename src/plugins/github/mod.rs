mod client;
pub mod model;
pub mod ai_stats;
pub mod commit_analyzer;

use std::collections::HashMap;
use std::path::PathBuf;

pub use crate::core::plugin::{ConfigField, Plugin, PluginMetadata};
pub use crate::error::Result;
use crate::{core::plugin::ConfigFieldType, plugins::github::model::GitHubConfig};
use crate::{core::message::Message, error::WorkOsError};
use async_trait::async_trait;
pub use client::GithubClient;
use toml::Value;

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

    pub fn client(&self) -> Option<&GithubClient> {
        self.client.as_ref()
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
                label: "Personal Access Token",
                help: "Create at: https://github.com/settings/tokens\n\
                        Required scopes: repo, read:org, read:user",
                field_type: ConfigFieldType::Secret,
                required: true,
                default: None,
            },
            ConfigField {
                name: "username",
                label: "GitHub Username",
                help: "Your GitHub username",
                field_type: ConfigFieldType::String,
                required: true,
                default: None,
            },
            ConfigField {
                name: "include_orgs",
                label: "Organizations to include",
                help: "Comma-separated org names (tracks all repos in these orgs)",
                field_type: ConfigFieldType::StringList,
                required: false,
                default: None,
            },
            ConfigField {
                name: "bots",
                label: "Bot usernames to ignore",
                help: "Comma-separated bot usernames",
                field_type: ConfigFieldType::StringList,
                required: false,
                default: None,
            },
        ]
    }

    fn configure_from_values(&mut self, values: &HashMap<String, Value>, _output_path: &PathBuf) -> Result<()> {
        let token = values
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("GitHub token is required".into()))?
            .to_string();

        let username = values
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("GitHub username is required".into()))?
            .to_string();

        let include_orgs = ConfigField::extract_string_list(values, "include_orgs");
        let bots = ConfigField::extract_string_list(values, "bots");

        let config = GitHubConfig {
            token,
            username,
            include_orgs,
            bots,
        };

        self.configure(config)
    }

    async fn test_connection(&self) -> Result<bool> {
        match &self.client {
            Some(client) => client.test_connection().await,
            None => Ok(false),
        }
    }

    async fn fetch_messages(&self) -> Result<Vec<Message>> {
        match &self.client {
            Some(client) => client.get_all_messages().await,
            None => Ok(Vec::new()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
