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
use crate::plugins::google::auth::{check_if_credentials_present, GoogleOAuthConfig};

use client::GoogleCalendarClient;

pub struct GoogleCalendarPlugin {
    client: Option<GoogleCalendarClient>,
    config: Option<GoogleOAuthConfig>,
}

impl GoogleCalendarPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(
        &mut self,
        config: GoogleOAuthConfig,
        color_labels: HashMap<String, String>,
        upcoming_days: i64,
    ) -> Result<()> {
        let client = GoogleCalendarClient::new(&config, color_labels, upcoming_days);
        self.config = Some(config);
        self.client = Some(client);
        Ok(())
    }
}

impl Default for GoogleCalendarPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GoogleCalendarPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "google_calendar",
            name: "Google Calendar",
            description: "Fetch meetings and events from Google Calendar",
            icon: "📅",
        }
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth2
    }

    fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                name: "upcoming_days",
                label: "Upcoming Days",
                help: "How many days ahead to fetch calendar events (default: 3)",
                field_type: crate::core::plugin::ConfigFieldType::String,
                required: false,
                default: Some("3"),
            },
        ]
    }

    fn configure_from_values(
        &mut self,
        values: &HashMap<String, Value>,
        _output_path: &PathBuf,
    ) -> Result<()> {
        if !check_if_credentials_present() {
            return Err(WorkOsError::Google(
                "Google credentials not embedded. Add them to .cargo/config.toml and rebuild."
                    .into(),
            ));
        }

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

        // How many days ahead to fetch events (default: 3)
        let upcoming_days = values
            .get("upcoming_days")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()).or_else(|| v.as_integer()))
            .unwrap_or(3);

        // User-defined color labels from [plugins.google_calendar.colors]
        // e.g. Sage = "Focus time", Tomato = "Interviews"
        let color_labels = values
            .get("colors")
            .and_then(|v| v.as_table())
            .map(|table| {
                table
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        self.configure(
            GoogleOAuthConfig { access_token, refresh_token, expires_at },
            color_labels,
            upcoming_days,
        )
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
