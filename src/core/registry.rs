use crate::core::plugin::Plugin;
use crate::core::message::Message;
use crate::error::{Result, WorkOsError};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins
            .insert(plugin.metadata().id.to_string(), Arc::from(plugin));
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(id).cloned()
    }

    pub fn get_all(&self) -> Vec<Arc<dyn Plugin>> {
        self.plugins.values().cloned().collect()
    }

    pub fn list_ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub async fn fetch_messages_from(&self, plugin_ids: &[String]) -> Result<Vec<Message>> {
        let mut messages = Vec::new();
        for plugin_id in plugin_ids {
            if let Some(plugin) = self.get(plugin_id) {
                if !plugin.is_configured() {
                    continue;
                }
                match plugin.test_connection().await {
                    Ok(true) => {}
                    Ok(false) => {
                        eprintln!(
                            "Error fetching from {}: authentication failed — check your credentials",
                            plugin.metadata().name
                        );
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error fetching from {}: {}", plugin.metadata().name, e);
                        continue;
                    }
                }
                match plugin.fetch_messages().await {
                    Ok(msgs) => messages.extend(msgs),
                    Err(e) => {
                        eprintln!("Error fetching from {}: {}", plugin.metadata().name, e);
                    }
                }
            }
        }
        Ok(messages)
    }

    pub fn get_client(&self, plugin_name: &str) -> Result<Arc<dyn Plugin>> {
        self.get(plugin_name)
            .ok_or_else(|| WorkOsError::Config(format!("{} plugin not found", plugin_name)))
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

