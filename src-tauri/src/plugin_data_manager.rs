use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct PluginDataManager {
    data_dir: PathBuf,
}

impl PluginDataManager {
    pub fn new<R: Runtime>(app_handle: &AppHandle<R>) -> Self {
        let data_dir = app_handle.path().app_data_dir().unwrap().join("plugin_data");
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).expect("Failed to create plugin data directory");
        }
        Self { data_dir }
    }

    fn get_plugin_data_path(&self, plugin_id: &str) -> PathBuf {
        self.data_dir.join(format!("{}.json", plugin_id))
    }

    pub fn get_config(&self, plugin_id: &str, key: &str) -> Result<Value, String> {
        let path = self.get_plugin_data_path(plugin_id);
        if !path.exists() {
            return Ok(Value::Null);
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let data: HashMap<String, Value> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(data.get(key).cloned().unwrap_or(Value::Null))
    }

    pub fn set_config(&self, plugin_id: &str, key: &str, value: Value) -> Result<(), String> {
        let path = self.get_plugin_data_path(plugin_id);
        let mut data: HashMap<String, Value> = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&content).map_err(|e| e.to_string())?
        } else {
            HashMap::new()
        };
        data.insert(key.to_string(), value);
        let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}