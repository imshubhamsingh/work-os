pub mod github;

use crate::error::Result;
use crate::core::registry::PluginRegistry;
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubPlugin;

pub fn create_registry(config: &WorkOsConfig) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    // Register github plugin
    let mut github_plugin = GithubPlugin::new();

    if let Some(ref github_config) = config.github {
        github_plugin.configure(github_config.clone())?;
    }
    registry.register(Box::new(github_plugin));

    // TODO: Register Slack plugin
    // TODO: Register Jira plugin

    Ok(registry)

}