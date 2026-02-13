use std::collections::HashMap;

use crate::core::plugin::Plugin;
use crate::error::{Result, WorkOsError};
use crate::models::config::{PluginConfig, WorkOsConfig};
use crate::plugins::get_all_plugins;
use colored::*;
use toml::Value;

pub async fn test_all_plugin_auth(plugin_filter: Option<String>) -> Result<()> {
    let plugins = get_all_plugins();
    let config = WorkOsConfig::load()?;
    for plugin in plugins {
        let plugin_id = plugin.metadata().id;
        if let Some(ref filter) = plugin_filter {
            if filter != plugin_id {
                continue;
            }
        }
        test_plugin_auth(plugin_id, config.get_plugin(plugin_id)).await?;
    }

    Ok(())
}

pub async fn test_plugin_auth(plugin_id: &str, plugin_config: Option<&PluginConfig>) -> Result<()> {
    let strict = plugin_config.is_some();

    let owned_values;
    let plugin_config_values: &HashMap<String, Value> = match plugin_config {
        Some(p) => &p.values,
        None => {
            let config = WorkOsConfig::load()?;
            let plugin = match config.get_plugin(plugin_id) {
                Some(p) => p,
                None => {
                    println!("{} Plugin '{}' not configured", "⚠".yellow(), plugin_id);
                    return Ok(());
                }
            };

            owned_values = plugin.values.clone();
            &owned_values
        }
    };

    let config = WorkOsConfig::load()?;
    let output_path = config.output.base_path.join(&config.output.markdown_path);

    let mut test_plugin = create_test_plugin_by_id(plugin_id)?;
    test_plugin.configure_from_values(plugin_config_values, &output_path)?;

    match test_plugin.test_connection().await {
        Ok(true) => {
            println!("{} Connection successful!", "✔".green());
            Ok(())
        }

        Ok(false) => {
            let msg = format!(
                "{} Connection test returned false",
                test_plugin.metadata().name
            );
            if strict {
                Err(WorkOsError::Config(msg.into()))
            } else {
                println!("{} {}", "✗".red(), msg);
                Ok(())
            }
        }

        Err(e) => {
            if strict {
                Err(WorkOsError::Config(format!("Connection failed: {}", e)))
            } else {
                println!(
                    "{} {} Connection failed: {}",
                    "✗".red(),
                    test_plugin.metadata().name,
                    e
                );
                Ok(())
            }
        }
    }
}

pub fn create_test_plugin_by_id(id: &str) -> Result<Box<dyn Plugin>> {
    let plugins = get_all_plugins();
    plugins
        .into_iter()
        .find(|plugin| plugin.metadata().id == id)
        .ok_or_else(|| WorkOsError::Config(format!("Unknown plugin: {}", id)))
}
