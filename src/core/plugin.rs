use crate::core::task::Task;
use crate::error::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub icon: &'static str,
}

#[derive(Debug, Clone)]
pub struct ConfigField {
    pub name: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub required: bool,
    pub secret: bool,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn is_configured(&self) -> bool;
    fn config_schema(&self) -> Vec<ConfigField>;
    async fn test_connection(&self) -> Result<bool>;
    async fn fetch_tasks(&self) -> Result<Vec<Task>>;
}
