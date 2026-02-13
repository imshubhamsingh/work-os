mod client;
mod config;
mod model;

use crate::core::plugin::{
    ConfigField, ConfigFieldType, NestedFieldSchema, Plugin, PluginMetadata,
};
use crate::core::message::Message;
use crate::error::{Result, WorkOsError};
use async_trait::async_trait;
use client::JiraClient;
use config::JiraConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

pub struct JiraPlugin {
    client: Option<JiraClient>,
    config: Option<JiraConfig>,
}

impl JiraPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }

    pub fn configure(&mut self, config: JiraConfig) -> Result<()> {
        let client = JiraClient::new(&config)?;
        self.client = Some(client);
        self.config = Some(config);
        Ok(())
    }
}

impl Default for JiraPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for JiraPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "jira",
            name: "Jira",
            description: "Fetch issues from Jira/Atlassian",
            icon: "🎫",
        }
    }

    fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    fn config_schema(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                name: "domain",
                label: "Jira Domain",
                help: "Your Atlassian domain (e.g., company.atlassian.net)",
                field_type: ConfigFieldType::String,
                required: true,
                default: None,
            },
            ConfigField {
                name: "email",
                label: "Email",
                help: "Email address for authentication",
                field_type: ConfigFieldType::String,
                required: true,
                default: None,
            },
            ConfigField {
                name: "token",
                label: "API Token",
                help: "API token from https://id.atlassian.com/manage-profile/security/api-tokens",
                field_type: ConfigFieldType::Secret,
                required: true,
                default: None,
            },
            ConfigField {
                name: "filters",
                label: "Jira Filters",
                help: "JQL filters to fetch specific issues from Jira",
                field_type: ConfigFieldType::NestedArray(vec![
                    NestedFieldSchema {
                        name: "name",
                        label: "Filter Name",
                        help: "Display name for this filter (e.g., 'My Active Messages')",
                        field_type: ConfigFieldType::String,
                        required: true,
                        default: None,
                    },
                    NestedFieldSchema {
                        name: "jql",
                        label: "JQL Query",
                        help: "Jira Query Language expression (e.g., 'project = EM AND assignee = currentUser()')",
                        field_type: ConfigFieldType::String,
                        required: true,
                        default: None,
                    },
                    NestedFieldSchema {
                        name: "priority",
                        label: "Priority",
                        help: "Priority level: critical, high, medium, or low",
                        field_type: ConfigFieldType::String,
                        required: false,
                        default: Some("medium"),
                    },
                ]),
                required: false,
                default: None,
            },
        ]
    }

    fn configure_from_values(
        &mut self,
        values: &HashMap<String, Value>,
        _output_path: &PathBuf,
    ) -> Result<()> {
        let domain = values
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("Jira missing 'domain'".into()))?
            .to_string();

        let email = values
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("Jira missing 'email'".into()))?
            .to_string();

        let token = values
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkOsError::Config("Jira missing 'api_token'".into()))?
            .to_string();

        let filters = values
            .get("filters")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| JiraConfig::parse_filter(v).ok())
                    .collect()
            })
            .unwrap_or_default();

        let jira_config = JiraConfig {
            domain,
            email,
            token,
            filters,
        };

        self.configure(jira_config)
    }

    async fn test_connection(&self) -> Result<bool> {
        match &self.client {
            Some(client) => client.test_connection().await,
            None => Ok(false),
        }
    }

    async fn fetch_messages(&self) -> Result<Vec<Message>> {
        match &self.client {
            Some(_client) => {
                let mut client_clone = JiraClient::new(self.config.as_ref().unwrap())?;
                client_clone.get_all_messages().await
            }
            None => Ok(Vec::new()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
