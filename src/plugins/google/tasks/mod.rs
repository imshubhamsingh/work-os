mod client;
mod model;

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use toml::Value;

use crate::core::message::Message;
use crate::core::plugin::{AuthType, ConfigField, Plugin, PluginMetadata};
use crate::error::{Result, WorkOsError};
use crate::plugins::google::auth::GoogleOAuthConfig;

use client::GoogleTasksClient;

pub struct GoogleTasksPlugin {
    client: Option<GoogleTasksClient>,
}

impl GoogleTasksPlugin {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn configure(&mut self, config: GoogleOAuthConfig) -> Result<()> {
        self.client = Some(GoogleTasksClient::new(&config));
        Ok(())
    }
}

impl Default for GoogleTasksPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GoogleTasksPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "google_tasks",
            name: "Google Tasks",
            description: "Fetch todo items from Google Tasks",
            icon: "📋",
        }
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth2
    }

    fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![]
    }

    fn configure_from_values(
        &mut self,
        values: &HashMap<String, Value>,
        _output_path: &PathBuf,
    ) -> Result<()> {
        let access_token = values
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                WorkOsError::Google("No OAuth token found. Run: work-os auth google".into())
            })?
            .to_string();

        let refresh_token = values
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(String::from);

        let expires_at = values.get("expires_at").and_then(|v| v.as_integer());

        self.configure(GoogleOAuthConfig {
            access_token,
            refresh_token,
            expires_at,
        })
    }

    async fn is_authenticated(&self) -> bool {
        match &self.client {
            Some(client) => client.test_connection().await.unwrap_or(false),
            None => false,
        }
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}
