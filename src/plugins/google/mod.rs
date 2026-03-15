pub mod auth;
pub mod calendar;
pub mod tasks;

pub use calendar::GoogleCalendarPlugin;
pub use tasks::GoogleTasksPlugin;

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use toml::Value;

use crate::core::message::Message;
use crate::core::plugin::{AuthType, ConfigField, Plugin, PluginMetadata};
use crate::error::Result;
use crate::plugins::google::auth::{oauth_token_schema, run_oauth_flow};

pub struct GooglePlugin;

impl GooglePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GooglePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GooglePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "google",
            name: "Google",
            description:
                "Shared Google OAuth credentials for Calendar, Tasks, and future Google plugins",
            icon: "🔑",
        }
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth2
    }

    fn is_configured(&self) -> bool {
        auth::check_if_credentials_present()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        oauth_token_schema()
    }

    fn configure_from_values(
        &mut self,
        _values: &HashMap<String, Value>,
        _output_path: &PathBuf,
    ) -> Result<()> {
        Ok(())
    }

    async fn run_auth_flow(&self) -> Result<()> {
        run_oauth_flow().await
    }

    async fn is_authenticated(&self) -> bool {
        use crate::models::config::WorkOsConfig;
        WorkOsConfig::load()
            .ok()
            .and_then(|c| c.get_plugin("google").map(|p| p.values.contains_key("access_token")))
            .unwrap_or(false)
    }

    async fn test_connection(&self) -> Result<bool> {
        Ok(self.is_authenticated().await)
    }

    async fn fetch_messages(&self) -> Result<Vec<Message>> {
        // No messages as this plugin exists only for auth/config purpose
        Ok(Vec::new())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
