use std::collections::HashMap;

use crate::core::task::Task;
use crate::error::Result;
use async_trait::async_trait;
use toml::Value;

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}

#[derive(Debug, Clone)]
pub enum ConfigFieldType {
    String,
    Secret,
    StringList,
    // Integer,
    // Boolean,
}

impl ConfigFieldType {
    pub fn format_value(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                items.join(", ")
            }
            _ => value.to_string(),
        }
    }

    pub fn parse_value(input: &str, field_type: &ConfigFieldType) -> Value {
        match field_type {
            ConfigFieldType::String | ConfigFieldType::Secret => Value::String(input.to_string()),
            ConfigFieldType::StringList => {
                let items: Vec<Value> = input
                    .split(',')
                    .map(|s| Value::String(s.trim().to_string()))
                    .filter(|v| !v.as_str().unwrap().is_empty())
                    .collect();
                Value::Array(items)
            }
            // ConfigFieldType::Integer => input
            //     .parse::<i64>()
            //     .map(Value::Integer)
            //     .unwrap_or_else(|_| Value::String(input.to_string())),
            // ConfigFieldType::Boolean => {
            //     let lower = input.to_lowercase();
            //     Value::Boolean(lower == "y" || lower == "yes" || lower == "true" || lower == "1")
            // }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigField {
    pub name: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub field_type: ConfigFieldType,
    pub required: bool,
    pub default: Option<&'static str>,
}

impl ConfigField {
    pub fn is_secret(&self) -> bool {
        matches!(self.field_type, ConfigFieldType::Secret)
    }

    pub fn extract_string_list(values: &HashMap<String, Value>, key: &str) -> Vec<String> {
        values
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn is_configured(&self) -> bool;
    fn config_schema(&self) -> Vec<ConfigField>;
    fn configure_from_values(&mut self, values: &HashMap<String, Value>) -> Result<()>;
    async fn test_connection(&self) -> Result<bool>;
    async fn fetch_tasks(&self) -> Result<Vec<Task>>;
}
