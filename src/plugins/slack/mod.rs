mod client;
mod model;

use async_trait::async_trait;
pub use client::SlackClient;

use crate::core::plugin::{ConfigField, Plugin, PluginMetadata};
use crate::core::task::Task;
use crate::error::Result;
use crate::models::config::SlackConfig;

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
            version: "0.1.0",
            icon: "💬",
        }
    }

    fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![ConfigField {
            name: "token",
            label: "User Token",
            help: "Slack user token (xoxp-...) - Required for DMs, search, private channels",
            required: true,
            secret: true,
        }]
    }

    async fn test_connection(&self) -> Result<bool> {
        match &self.client {
            Some(client) => client.test_connection().await,
            None => Ok(false),
        }
    }

    async fn fetch_tasks(&self) -> Result<Vec<Task>> {
        match &self.client {
            Some(client) => {
                let mut client_clone = SlackClient::new(self.config.as_ref().unwrap())?;
                client_clone.get_all_tasks().await
            }
            None => Ok(Vec::new())
        }
    }
}
