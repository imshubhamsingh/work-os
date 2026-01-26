pub mod github;
pub mod slack;

use crate::core::plugin::Plugin;
use crate::core::registry::PluginRegistry;
use crate::error::Result;
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubPlugin;
use crate::plugins::slack::SlackPlugin;

pub fn create_registry(config: &WorkOsConfig) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    let mut github_plugin = GithubPlugin::new();
    if let Some(ref github_plugin_config) = config.get_plugin("github") {
        if github_plugin_config.enabled {
            if let Err(e) = github_plugin.configure_from_values(&github_plugin_config.values) {
                eprintln!("Warning: Failed to configure GitHub: {}", e);
            }
        }
    }
    registry.register(Box::new(github_plugin));

    let mut slack_plugin = SlackPlugin::new();
    if let Some(slack_plugin_config) = config.get_plugin("slack") {
        if slack_plugin_config.enabled {
            if let Err(e) = slack_plugin.configure_from_values(&slack_plugin_config.values) {
                eprintln!("Warning: Failed to configure Slack: {}", e);
            }
        }
    }
    registry.register(Box::new(slack_plugin));

    // TODO: Register Jira plugin

    Ok(registry)
}

pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![Box::new(GithubPlugin::new()), Box::new(SlackPlugin::new())]
}
