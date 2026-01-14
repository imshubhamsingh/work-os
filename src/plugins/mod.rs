pub mod github;
pub mod slack;

use crate::error::Result;
use crate::core::registry::PluginRegistry;
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubPlugin;
use crate::plugins::slack::SlackPlugin;

pub fn create_registry(config: &WorkOsConfig) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    // Register github plugin
    let mut github_plugin = GithubPlugin::new();

    if let Some(ref github_config) = config.github {
        github_plugin.configure(github_config.clone())?;
    }
    registry.register(Box::new(github_plugin));

    // Register slack plugin
    let mut slack_plugin = SlackPlugin::new();

    if let Some(ref slack_config) = config.slack {
        slack_plugin.configure(slack_config.clone())?;
    }
    registry.register(Box::new(slack_plugin));

    // TODO: Register Jira plugin

    Ok(registry)

}