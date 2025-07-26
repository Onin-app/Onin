use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Runtime, Window, State, WebviewWindow, AppHandle};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::permission_manager::PermissionManager;
use crate::plugin_loader::PluginLoader;
use crate::unified_launch_manager;
use crate::installed_apps;
use crate::app_cache_manager::AppCache;
use crate::plugin_data_manager::PluginDataManager;
use crate::startup_apps_manager::StartupAppsManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PluginMessage {
    Request {
        plugin_id: String,
        method: String,
        params: serde_json::Value,
        callback_id: u64,
    },
    Response {
        callback_id: u64,
        result: Result<serde_json::Value, String>,
    },
}

pub struct PluginManager {
    callbacks: HashMap<u64, Box<dyn FnOnce(Result<serde_json::Value, String>) + Send>>,
    next_callback_id: u64,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            next_callback_id: 0,
        }
    }

    pub fn register_callback<F>(&mut self, callback: F) -> u64
    where
        F: FnOnce(Result<serde_json::Value, String>) + 'static + Send,
    {
        let id = self.next_callback_id;
        self.next_callback_id += 1;
        self.callbacks.insert(id, Box::new(callback));
        id
    }

    pub fn handle_response(&mut self, response: PluginMessage) {
        if let PluginMessage::Response { callback_id, result } = response {
            if let Some(callback) = self.callbacks.remove(&callback_id) {
                callback(result);
            }
        }
    }
}

#[tauri::command]
pub async fn handle_plugin_message<R: Runtime>(
    window: Window<R>,
    message: PluginMessage,
) {
    let app_handle = window.app_handle().clone();
    match message {
        PluginMessage::Request { plugin_id, method, params, callback_id } => {
            let window_clone = window.clone();
            tokio::spawn(async move {
                let result: Result<serde_json::Value, String> = async {
                    let permission_manager_state: State<'_, Mutex<PermissionManager>> = app_handle.state();
                    let plugin_loader_state: State<'_, Mutex<PluginLoader<R>>> = app_handle.state();
                    let app_cache_state: State<'_, AppCache> = app_handle.state();
                    let startup_manager_state: State<'_, StartupAppsManager> = app_handle.state();
                    let data_manager_state: State<'_, PluginDataManager> = app_handle.state();

                    // Initial permission check
                    {
                        let pl_lock = plugin_loader_state.lock().map_err(|_| "Failed to lock plugin loader".to_string())?;
                        let manifest = pl_lock.plugins.get(&plugin_id).ok_or_else(|| "Plugin not found".to_string())?;
                        let pm_lock = permission_manager_state.lock().map_err(|_| "Failed to lock permission manager".to_string())?;
                        if !manifest.permissions.iter().all(|p| pm_lock.has_permission(&plugin_id, p)) {
                            return Err("Permission denied".to_string());
                        }
                    }

                    match method.as_str() {
                        "readFile" => {
                            let has_perm = {
                                let pm = permission_manager_state.lock().unwrap();
                                pm.has_permission(&plugin_id, "filesystem:read")
                            };
                            if !has_perm { return Err("Permission denied: filesystem:read".to_string()); }

                            let path: String = serde_json::from_value(params.get("path").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'path' parameter: {}", e))?;
                            tokio::fs::read_to_string(path).await
                                .map(|content| serde_json::json!(content))
                                .map_err(|e| e.to_string())
                        }
                        "writeFile" => {
                            let has_perm = {
                                let pm = permission_manager_state.lock().unwrap();
                                pm.has_permission(&plugin_id, "filesystem:write")
                            };
                            if !has_perm { return Err("Permission denied: filesystem:write".to_string()); }

                            let path: String = serde_json::from_value(params.get("path").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'path' parameter: {}", e))?;
                            let content: String = serde_json::from_value(params.get("content").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'content' parameter: {}", e))?;
                            tokio::fs::write(path, content).await
                                .map(|_| serde_json::json!({ "status": "success" }))
                                .map_err(|e| e.to_string())
                        }
                        "executeCommand" => {
                             let has_perm = {
                                let pm = permission_manager_state.lock().unwrap();
                                pm.has_permission(&plugin_id, "system:commands")
                            };
                            if !has_perm { return Err("Permission denied: system:commands".to_string()); }

                            let command: String = serde_json::from_value(params.get("command").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'command' parameter: {}", e))?;
                            let output = tokio::process::Command::new("sh")
                                .arg("-c")
                                .arg(command)
                                .output()
                                .await;
                            output.map(|o| serde_json::json!(String::from_utf8_lossy(&o.stdout).to_string()))
                                  .map_err(|e| e.to_string())
                        }
                        "searchApps" => {
                            let has_perm = {
                                let pm = permission_manager_state.lock().unwrap();
                                pm.has_permission(&plugin_id, "app:search")
                            };
                            if !has_perm { return Err("Permission denied: app:search".to_string()); }

                            unified_launch_manager::get_all_launchable_items(app_cache_state, startup_manager_state).await
                                .map(|items| serde_json::json!(items))
                        }
                        "launchApp" => {
                            let has_perm = {
                                let pm = permission_manager_state.lock().unwrap();
                                pm.has_permission(&plugin_id, "app:launch")
                            };
                            if !has_perm { return Err("Permission denied: app:launch".to_string()); }

                            let path: String = serde_json::from_value(params.get("appId").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'appId' parameter: {}", e))?;
                            let main_window = app_handle.get_webview_window("main").ok_or("Main window not found")?;
                            installed_apps::open_app(path, main_window).map(|_| serde_json::json!({"status": "success"}))
                        }
                        "getConfig" => {
                            let key: String = serde_json::from_value(params.get("key").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'key' parameter: {}", e))?;
                            data_manager_state.get_config(&plugin_id, &key)
                        }
                        "setConfig" => {
                            let key: String = serde_json::from_value(params.get("key").cloned().unwrap_or_default())
                                .map_err(|e| format!("Invalid 'key' parameter: {}", e))?;
                            let value: serde_json::Value = params.get("value").cloned().unwrap_or_default();
                            data_manager_state.set_config(&plugin_id, &key, value)
                                .map(|_| serde_json::json!({ "status": "success" }))
                        }
                        _ => Err(format!("Method not found: {}", method)),
                    }
                }.await;

                let response = PluginMessage::Response {
                    callback_id,
                    result,
                };
                let _ = window_clone.emit("plugin-response", response);
            });
        }
        PluginMessage::Response { .. } => {
            let manager: State<'_, Mutex<PluginManager>> = app_handle.state();
            let mut mgr = manager.lock().unwrap();
            mgr.handle_response(message);
        }
    }
}