use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct PluginStore(pub Mutex<HashMap<String, LoadedPlugin>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadedPlugin {
    #[serde(flatten)]
    pub manifest: PluginManifest,
    pub dir_name: String,
}

#[tauri::command]
pub fn load_plugins(app: tauri::AppHandle, store: State<PluginStore>) -> Result<Vec<LoadedPlugin>, String> {
    println!("[plugin_manager] Loading plugins...");
    let data_dir = app.path().app_data_dir().map_err(|e| {
        println!("[plugin_manager] Error getting data dir: {}", e);
        e.to_string()
    })?;

    let plugins_dir = data_dir.join("plugins");
    println!("[plugin_manager] Plugins dir: {:?}", plugins_dir);

    if !plugins_dir.is_dir() {
        println!("[plugin_manager] Plugins dir not found.");
        return Ok(Vec::new());
    }

    let mut store_lock = store.0.lock().unwrap();
    store_lock.clear(); // Clear old plugins

    for entry in std::fs::read_dir(plugins_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.is_file() {
                let manifest_content = std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
                let manifest: PluginManifest = serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;
                
                let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
                
                let loaded_plugin = LoadedPlugin {
                    manifest: manifest.clone(),
                    dir_name,
                };
                
                println!("[plugin_manager] Loaded plugin: {} from {}", manifest.name, loaded_plugin.dir_name);
                store_lock.insert(manifest.id, loaded_plugin);
            }
        }
    }
    
    let plugins = store_lock.values().cloned().collect();
    println!("[plugin_manager] Loaded {} plugins.", store_lock.len());
    Ok(plugins)
}

use crate::js_runtime;

#[tauri::command]
pub fn execute_plugin_entry(
    app: tauri::AppHandle,
    store: State<PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    let store_lock = store.0.lock().unwrap();
    let plugin = store_lock.get(&plugin_id).ok_or_else(|| "Plugin not found".to_string())?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("Plugin entry file not found: {:?}", entry_path));
    }

    let js_code = std::fs::read_to_string(entry_path).map_err(|e| e.to_string())?;

    js_runtime::execute_js(&app, &js_code)
}
