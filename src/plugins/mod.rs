pub mod github;
pub mod granola;
pub mod jira;
pub mod slack;

use crate::core::plugin::Plugin;
use crate::core::registry::PluginRegistry;
use crate::error::Result;
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubPlugin;
use crate::plugins::granola::GranolaPlugin;
use crate::plugins::jira::JiraPlugin;
use crate::plugins::slack::SlackPlugin;

pub fn create_registry(config: &WorkOsConfig) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();
    let output_path = config.output.base_path.join(&config.output.markdown_path);

    let mut github_plugin = GithubPlugin::new();
    if let Some(ref github_plugin_config) = config.get_plugin("github") {
        if github_plugin_config.enabled {
            if let Err(e) = github_plugin.configure_from_values(&github_plugin_config.values, &output_path) {
                eprintln!("Warning: Failed to configure GitHub: {}", e);
            }
        }
    }
    registry.register(Box::new(github_plugin));

    let mut slack_plugin = SlackPlugin::new();
    if let Some(slack_plugin_config) = config.get_plugin("slack") {
        if slack_plugin_config.enabled {
            if let Err(e) = slack_plugin.configure_from_values(&slack_plugin_config.values, &output_path) {
                eprintln!("Warning: Failed to configure Slack: {}", e);
            }
        }
    }
    registry.register(Box::new(slack_plugin));

    let mut jira_plugin = JiraPlugin::new();
    if let Some(jira_config) = config.get_plugin("jira") {
        if jira_config.enabled {
            if let Err(e) = jira_plugin.configure_from_values(&jira_config.values, &output_path) {
                eprintln!("Warning: Failed to configure Jira: {}", e);
            }
        }
    }
    registry.register(Box::new(jira_plugin));

    let mut granola_plugin = GranolaPlugin::new();
    if let Some(granola_config) = config.get_plugin("granola") {
        if granola_config.enabled {
            if let Err(e) = granola_plugin.configure_from_values(&granola_config.values, &output_path) {
                eprintln!("Warning: Failed to configure Granola: {}", e);
            }
        }
    }
    registry.register(Box::new(granola_plugin));

    Ok(registry)
}

pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(GithubPlugin::new()),
        Box::new(SlackPlugin::new()),
        Box::new(JiraPlugin::new()),
        Box::new(GranolaPlugin::new()),
    ]
}
