mod client;
mod config;
mod model;

use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
pub use client::CoralogixClient;
use toml::Value;

use crate::core::message::Message;
use crate::core::plugin::{ConfigField, ConfigFieldType, Plugin, PluginMetadata};
use crate::error::{Result, WorkOsError};
use crate::plugins::coralogix::config::CoralogixConfig;

pub struct CoralogixPlugin {
    client: Option<CoralogixClient>,
    config: Option<CoralogixConfig>,
}

impl CoralogixPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(&mut self, config: CoralogixConfig) -> Result<()> {
        let client = CoralogixClient::new(&config)?;
        self.config = Some(config);
        self.client = Some(client);
        Ok(())
    }
}

impl Default for CoralogixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CoralogixPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "coralogix",
            name: "Coralogix",
            description: "Fetch production error logs from Coralogix",
            icon: "🚨",
        }
    }

    fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                name: "api_key",
                label: "API Key",
                help: "Coralogix Logs Query API key",
                field_type: ConfigFieldType::Secret,
                required: true,
                default: None,
            },
            ConfigField {
                name: "domain",
                label: "Domain",
                help: "Your Coralogix dashboard URL (e.g. https://yourteam.coralogix.com)",
                field_type: ConfigFieldType::String,
                required: true,
                default: None,
            },
            ConfigField {
                name: "application_names",
                label: "Application names",
                help: "Comma-separated list of Coralogix application names to query",
                field_type: ConfigFieldType::StringList,
                required: true,
                default: None,
            },
        ]
    }

    fn configure_from_values(
        &mut self,
        values: &HashMap<String, Value>,
        output_path: &PathBuf,
    ) -> Result<()> {
        let api_key = values
            .get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Coralogix("Coralogix API key is required".into()))?
            .to_string();

        let domain = values
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Coralogix("Coralogix domain is required".into()))?
            .to_string();

        let application_names = ConfigField::extract_string_list(values, "application_names");
        if application_names.is_empty() {
            return Err(WorkOsError::Coralogix(
                "At least one application name is required".into(),
            ));
        }

        let config = CoralogixConfig {
            api_key,
            domain,
            application_names,
            output_path: output_path.clone(),
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
            Some(_) => {
                let client = CoralogixClient::new(self.config.as_ref().unwrap())?;
                client.get_all_messages().await
            }
            None => Ok(Vec::new()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
