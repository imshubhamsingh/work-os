use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use toml::Value;

use crate::{
    core::{
        plugin::{ConfigField, ConfigFieldType, Plugin, PluginMetadata},
        task::Task,
    },
    error::Result,
    plugins::granola::{client::GranolaClient, config::GranolaConfig},
};

mod cache_reader;
mod client;
mod config;
mod model;
mod mom_writer;

pub struct GranolaPlugin {
    client: Option<GranolaClient>,
    config: Option<GranolaConfig>,
}

impl GranolaPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(&mut self, config: GranolaConfig) -> Result<()> {
        let client = GranolaClient::new(&config)?;
        self.client = Some(client);
        self.config = Some(config);
        Ok(())
    }
}

impl Default for GranolaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GranolaPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "granola",
            name: "Granola",
            description: "Fetch meeting notes from Granola cache",
            icon: "🍥",
        }
    }

    fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
                ConfigField {
                    name: "mom_folder_name",
                    label: "MOM Folder Name",
                    help: "Relative path for MOM folder (created inside work-os output directory).\n\
                           Default: mom\n\
                           Note: Cache file location is fixed at ~/Library/Application Support/Granola/cache-v3.json",
                    field_type: ConfigFieldType::String,
                    required: false,
                    default: Some("mom"),
                },
            ]
    }

    fn configure_from_values(
        &mut self,
        values: &HashMap<String, Value>,
        base_path: &PathBuf,
    ) -> Result<()> {
        let mom_folder_name = values
            .get("mom_folder_name")
            .and_then(|v| v.as_str())
            .unwrap_or("mom")
            .to_string();

        let granola_config = GranolaConfig {
            mom_folder_name,
            output_base: base_path.clone(),
        };

        self.configure(granola_config)
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
                let mut client_clone = GranolaClient::new(self.config.as_ref().unwrap())?;
                client_clone.get_all_tasks().await
            }
            None => Ok(Vec::new()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
