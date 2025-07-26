use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry: String,
    pub permissions: Vec<String>,
    pub dependencies: HashMap<String, String>,
    #[serde(skip)]
    pub path: Option<String>,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Plugin ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }
        if self.version.is_empty() {
            return Err("Plugin version cannot be empty".to_string());
        }
        if self.entry.is_empty() {
            return Err("Plugin entry file cannot be empty".to_string());
        }
        Ok(())
    }
}