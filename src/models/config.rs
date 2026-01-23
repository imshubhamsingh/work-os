use crate::error::{Result, WorkOsError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOsConfig {
    pub github: Option<GitHubConfig>,
    pub slack: Option<SlackConfig>,
    pub output: OutputConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: String,
    pub username: String,
    pub include_orgs: Vec<String>,
    pub include_repos: Vec<String>,
    pub bots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub token: String,
    pub keywords: Vec<String>,
    pub channels: Vec<String>,
    pub user_groups: Vec<String>,
    pub max_messages_per_channel: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub base_path: PathBuf,
    pub dashboard_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub lookback_days: u32,
    pub max_items_per_source: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            base_path: dirs::home_dir()
                .unwrap()
                .join("Projects/obsidian/work/work-os"),
            dashboard_path: "dashboards".to_string(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            lookback_days: 7,
            max_items_per_source: 50,
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
}
