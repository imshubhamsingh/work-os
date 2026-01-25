use crate::error::{Result, WorkOsError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkOsConfig {
    // pub github: Option<GitHubConfig>,
    // pub slack: Option<SlackConfig>,
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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GitHubConfig {
//     pub token: String,
//     pub username: String,
//     pub include_orgs: Vec<String>,
//     pub include_repos: Vec<String>,
//     pub bots: Vec<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SlackConfig {
//     pub token: String,
//     pub keywords: Vec<String>,
//     pub channels: Vec<String>,
//     pub user_groups: Vec<String>,
//     pub max_messages_per_channel: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub base_path: PathBuf,
    pub markdown_path: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            base_path: dirs::home_dir().unwrap().join(".work-os"),
            markdown_path: "markdown".to_string(),
        }
    }
}

impl WorkOsConfig {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap()
            .join(".work-os")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
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
        let config_path = Self::config_path();
        let config_dir = config_path.parent().unwrap();

        std::fs::create_dir_all(config_dir)?;

        let contents =
            toml::to_string_pretty(self).map_err(|e| WorkOsError::Config(e.to_string()))?;

        std::fs::write(&config_path, contents)?;

        Ok(())
    }

    // pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginConfig> {
    //     self.plugins.get(plugin_id)
    // }

    // fn get_plugin_mut(&mut self, plugin_id: &str) -> &mut PluginConfig {
    //     self.plugins
    //         .entry(plugin_id.to_string())
    //         .or_insert_with(PluginConfig::default)
    // }

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

impl PluginConfig {
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.values
            .get(key)
            .and_then(|v| v.as_str().map(String::from))
    }

    pub fn get_string_list(&self, key: &str) -> Vec<String> {
        self.values
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| v.as_integer())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }
}
