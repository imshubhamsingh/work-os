use crate::cli::auth::{create_test_plugin_by_id, test_plugin_auth};
use crate::core::plugin::ConfigFieldType;
use crate::error::WorkOsError;
use crate::models::config::*;
use crate::models::terminal::*;
use crate::plugins::get_all_plugins;
use crate::{core::plugin::Plugin, error::Result};
use colored::*;

pub async fn init(plugin_filter: Option<String>) -> Result<()> {
    let mut config = WorkOsConfig::load().unwrap_or_default();
    let plugins = get_all_plugins();

    println!("Work OS Configuration Setup\n");

    for plugin in plugins {
        let meta = plugin.metadata();
        if let Some(ref filter) = plugin_filter {
            if meta.id != filter {
                continue;
            }
        }

        println!("{} {} Configuration", meta.icon, meta.name);
        println!("{}", "-".repeat(40));

        let setup = Terminal::prompt(&format!("Configure {}? (y/n): ", meta.name))?;
        if setup.to_lowercase() != "y" {
            println!();
            continue;
        }

        match configure_plugin_interactive(&plugin, &mut config).await {
            Ok(()) => {
                println!("{} {} configured successfully!\n", "✔".green(), meta.name);
            }
            Err(e) => {
                println!("{} Failed to configure {}: {}\n", "✖".red(), meta.name, e);
            }
        }
    }

    config.save()?;

    println!(
        "\n✓ Configuration saved to: {:?}",
        WorkOsConfig::config_path()
    );
    Ok(())
}

async fn configure_plugin_interactive(
    plugin: &Box<dyn Plugin>,
    config: &mut WorkOsConfig,
) -> Result<()> {
    let meta = plugin.metadata();
    let schema = plugin.config_schema();
    let plugin_config = config.get_plugin_mut(meta.id);

    for field in &schema {
        let value = Terminal::prompt_for_field(field, plugin_config.values.get(field.name))?;

        if let Some(v) = value {
            plugin_config.values.insert(field.name.to_string(), v);
        }
    }

    plugin_config.enabled = true;

    println!("\nTesting connection...");

    test_plugin_auth(&meta.id, Some(&plugin_config.clone())).await
}

pub async fn set(plugin_id: &str, key: &str, value: &str) -> Result<()> {
    let mut config = WorkOsConfig::load()?;

    let plugin = create_test_plugin_by_id(plugin_id)?;
    let schema = plugin.config_schema();

    let field = schema.iter().find(|f| f.name == key).ok_or_else(|| {
        WorkOsError::Config(format!(
            "Unknown field '{}' for plugin '{}'",
            key, plugin_id
        ))
    })?;

    let parsed = ConfigFieldType::parse_value(value, &field.field_type);
    config.set_plugin_value(plugin_id, key, parsed);

    config.save()?;
    println!("✓ Set {}.{} = {}", plugin_id, key, value);

    Ok(())
}

pub async fn show(plugin_filter: Option<String>) -> Result<()> {
    let config = WorkOsConfig::load()?;
    let mut display_config = config.clone();
    let plugins = get_all_plugins();

    for plugin in plugins {
        let plugin_id = plugin.metadata().id;
        if let Some(ref filter) = plugin_filter {
            if plugin_id != filter {
                continue;
            }
        }

        let schema = plugin.config_schema();

        for field in schema {
            if !field.is_secret() {
                continue;
            }

            if let Some(plugin_config_values) = display_config
                .plugins
                .get_mut(plugin_id)
                .and_then(|plugin_config| plugin_config.values.get_mut(field.name))
            {
                *plugin_config_values =
                    toml::Value::String("************secret***********".to_string());
            }
        }
    }
    if let Some(filter) = plugin_filter {
        display_config.plugins.retain(|id, _| id == &filter);
    }

    let toml = toml::to_string_pretty(&display_config).unwrap();
    println!("{}", toml);

    Ok(())
}

pub async fn list() -> Result<()> {
    let config = WorkOsConfig::load().unwrap_or_default();

    println!("Available Plugins:\n");

    for plugin in get_all_plugins() {
        let meta = plugin.metadata();
        let status = if config.is_plugin_configured(meta.id) {
            format!("{}", "✔ configured".green())
        } else {
            format!("{}", "○ not configured".dimmed())
        };

        println!(
            "  {} {} - {} {}",
            meta.icon, meta.id, meta.description, status
        );
    }

    Ok(())
}
