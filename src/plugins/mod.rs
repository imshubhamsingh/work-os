pub mod coralogix;
pub mod github;
pub mod google;
pub mod granola;
pub mod jira;
pub mod slack;

use crate::core::plugin::Plugin;
use crate::core::registry::PluginRegistry;
use crate::error::Result;
use crate::models::config::WorkOsConfig;
use crate::plugins::coralogix::CoralogixPlugin;
use crate::plugins::github::GithubPlugin;
use crate::plugins::google::{GoogleCalendarPlugin, GooglePlugin, GoogleTasksPlugin};
use crate::plugins::granola::GranolaPlugin;
use crate::plugins::jira::JiraPlugin;
use crate::plugins::slack::SlackPlugin;

pub fn create_registry(config: &WorkOsConfig) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();
    let output_path = config.output.base_path.join(&config.output.markdown_path);

    let mut github_plugin = GithubPlugin::new();
    if let Some(ref github_plugin_config) = config.get_plugin("github") {
        if github_plugin_config.enabled {
            if let Err(e) =
                github_plugin.configure_from_values(&github_plugin_config.values, &output_path)
            {
                eprintln!("Warning: Failed to configure GitHub: {}", e);
            }
        }
    }
    registry.register(Box::new(github_plugin));

    let mut slack_plugin = SlackPlugin::new();
    if let Some(slack_plugin_config) = config.get_plugin("slack") {
        if slack_plugin_config.enabled {
            if let Err(e) =
                slack_plugin.configure_from_values(&slack_plugin_config.values, &output_path)
            {
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
            if let Err(e) =
                granola_plugin.configure_from_values(&granola_config.values, &output_path)
            {
                eprintln!("Warning: Failed to configure Granola: {}", e);
            }
        }
    }
    registry.register(Box::new(granola_plugin));

    let mut coralogix_plugin = CoralogixPlugin::new();
    if let Some(coralogix_config) = config.get_plugin("coralogix") {
        if coralogix_config.enabled {
            if let Err(e) =
                coralogix_plugin.configure_from_values(&coralogix_config.values, &output_path)
            {
                eprintln!("Warning: Failed to configure Coralogix: {}", e);
            }
        }
    }
    registry.register(Box::new(coralogix_plugin));

    registry.register(Box::new(GooglePlugin::new()));

    // Read the shared token once for calendar and tasks both use it
    let google_values = config
        .get_plugin("google")
        .map(|p| p.values.clone())
        .unwrap_or_default();
    let google_enabled = config.get_plugin("google").map_or(false, |p| p.enabled);

    let mut google_calendar_plugin = GoogleCalendarPlugin::new();
    if google_enabled
        && config
            .get_plugin("google_calendar")
            .map_or(false, |p| p.enabled)
    {
        let mut calendar_values = google_values.clone();
        if let Some(cal_config) = config.get_plugin("google_calendar") {
            calendar_values.extend(cal_config.values.clone());
        }
        if let Err(e) = google_calendar_plugin.configure_from_values(&calendar_values, &output_path)
        {
            eprintln!("Warning: Failed to configure Google Calendar: {}", e);
        }
    }
    registry.register(Box::new(google_calendar_plugin));

    let mut google_tasks_plugin = GoogleTasksPlugin::new();
    if google_enabled
        && config
            .get_plugin("google_tasks")
            .map_or(false, |p| p.enabled)
    {
        if let Err(e) = google_tasks_plugin.configure_from_values(&google_values, &output_path) {
            eprintln!("Warning: Failed to configure Google Tasks: {}", e);
        }
    }
    registry.register(Box::new(google_tasks_plugin));

    Ok(registry)
}

pub fn get_all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(GithubPlugin::new()),
        Box::new(SlackPlugin::new()),
        Box::new(JiraPlugin::new()),
        Box::new(GranolaPlugin::new()),
        Box::new(CoralogixPlugin::new()),
        Box::new(GooglePlugin::new()),
        Box::new(GoogleCalendarPlugin::new()),
        Box::new(GoogleTasksPlugin::new()),
    ]
}
