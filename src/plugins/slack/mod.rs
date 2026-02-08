mod client;
mod model;

use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
pub use client::SlackClient;
use toml::Value;

use crate::core::plugin::{ConfigField, ConfigFieldType, Plugin, PluginMetadata};
use crate::core::task::Task;
use crate::error::{Result, WorkOsError};
use crate::plugins::slack::model::SlackConfig;

pub struct SlackPlugin {
    client: Option<SlackClient>,
    config: Option<SlackConfig>,
}

impl SlackPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(&mut self, config: SlackConfig) -> Result<()> {
        let client = SlackClient::new(&config)?;
        self.config = Some(config);
        self.client = Some(client);
        Ok(())
    }
}

impl Default for SlackPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SlackPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "slack",
            name: "Slack",
            description: "Fetch messages and mentions from Slack",
            icon: "💬",
        }
    }

    fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                name: "token",
                label: "User Token",
                help: "Slack user token (xoxp-...)\n\
                           Create at: https://api.slack.com/apps\n\
                           Required for DMs, search, private channels",
                field_type: ConfigFieldType::Secret,
                required: true,
                default: None,
            },
            ConfigField {
                name: "keywords",
                label: "Keywords to track",
                help: "Messages containing these words may be action items",
                field_type: ConfigFieldType::StringList,
                required: false,
                default: None,
            },
            ConfigField {
                name: "channels",
                label: "Channels to monitor",
                help: "Comma-separated channel names (leave empty for all)",
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
            .ok_or_else(|| WorkOsError::Config("Slack user token is required".into()))?
            .to_string();

        let keywords = ConfigField::extract_string_list(values, "keywords");
        let channels = ConfigField::extract_string_list(values, "channels");

        let config = SlackConfig {
            token,
            keywords,
            channels,
        };

        self.configure(config)
    }

    async fn test_connection(&self) -> Result<bool> {
        match &self.client {
            Some(client) => client.test_connection().await,
            None => Ok(false),
        }
    }

    async fn fetch_tasks(&self) -> Result<Vec<Task>> {
        match &self.client {
            Some(_client) => {
                let mut client_clone = SlackClient::new(self.config.as_ref().unwrap())?;
                client_clone.get_all_tasks().await
            }
            None => Ok(Vec::new()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
