use crate::error::{Result, WorkOsError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkOsConfig {
    pub output: OutputConfig,
    pub plugins: HashMap<String, PluginConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(flatten)]
    pub values: HashMap<String, Value>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub base_path: PathBuf,
    pub markdown_path: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            base_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".work-os"),
            markdown_path: "raw".to_string(),
        }
    }
}

impl WorkOsConfig {
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| WorkOsError::Config("Could not determine home directory".into()))?;
        Ok(home.join(".work-os").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Err(WorkOsError::Config(
                "Config file not found. Run: work-os config init".to_string(),
            ));
        }

        let contents = std::fs::read_to_string(config_path)?;
        let config: WorkOsConfig = toml::from_str(&contents)
            .map_err(|e| WorkOsError::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(config_dir) = config_path.parent() {
            std::fs::create_dir_all(config_dir)?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|e| WorkOsError::Config(e.to_string()))?;

        std::fs::write(&config_path, contents)?;

        Ok(())
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginConfig> {
        self.plugins.get(plugin_id)
    }

    pub fn get_plugin_mut(&mut self, plugin_id: &str) -> &mut PluginConfig {
        self.plugins
            .entry(plugin_id.to_string())
            .or_insert_with(PluginConfig::default)
    }

    pub fn set_plugin_value(&mut self, plugin_id: &str, key: &str, value: Value) {
        let plugin_config = self
            .plugins
            .entry(plugin_id.to_string())
            .or_insert_with(PluginConfig::default);
        plugin_config.values.insert(key.to_string(), value);
    }

    pub fn is_plugin_configured(&self, plugin_id: &str) -> bool {
        self.plugins
            .get(plugin_id)
            .map_or(false, |p| p.enabled && !p.values.is_empty())
    }
}
