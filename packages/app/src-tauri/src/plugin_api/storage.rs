use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_store::{Error, StoreBuilder};

// 线程本地存储用于保存当前插件ID
thread_local! {
    static CURRENT_PLUGIN_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

// 设置当前插件ID（在插件调用开始时调用）
pub fn set_current_plugin_id(plugin_id: String) {
    CURRENT_PLUGIN_ID.with(|id| {
        *id.borrow_mut() = Some(plugin_id);
    });
}

// 清除当前插件ID（在插件调用结束时调用）
pub fn clear_current_plugin_id() {
    CURRENT_PLUGIN_ID.with(|id| {
        *id.borrow_mut() = None;
    });
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageError {
    pub name: String,
    pub message: String,
}

impl From<String> for StorageError {
    fn from(message: String) -> Self {
        StorageError {
            name: "StorageError".to_string(),
            message,
        }
    }
}

impl From<&str> for StorageError {
    fn from(message: &str) -> Self {
        StorageError {
            name: "StorageError".to_string(),
            message: message.to_string(),
        }
    }
}

impl From<Error> for StorageError {
    fn from(error: Error) -> Self {
        StorageError {
            name: "StorageError".to_string(),
            message: error.to_string(),
        }
    }
}

// 获取插件存储路径
fn get_plugin_store_path(plugin_id: &str) -> String {
    format!("plugin_data/{}/storage.json", plugin_id)
}

// 获取当前执行插件的 ID
pub fn get_current_plugin_id<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<String, StorageError> {
    // 1. 尝试从线程局部变量获取（适用于通过 executor 执行的 JS 插件）
    if let Some(id) = CURRENT_PLUGIN_ID.with(|id| id.borrow().clone()) {
        return Ok(id);
    }

    // 2. 优先从内联插件状态获取（避免被其他插件窗口焦点污染上下文）
    if let Some(inline_state) = tauri::Manager::try_state::<crate::plugin::InlinePluginState>(app) {
        if inline_state.is_visible.load(std::sync::atomic::Ordering::Relaxed) {
            if let Ok(id_lock) = inline_state.current_plugin_id.lock() {
                if let Some(id) = id_lock.as_ref() {
                    return Ok(id.to_string());
                }
            }
        }
    }

    // 3. 尝试从当前获取焦点的插件窗口 label 解析
    // 注意：显式使用 tauri::Manager 特性的方法
    for window in tauri::Manager::webview_windows(app).values() {
        let label = window.label();
        if label.starts_with("plugin_") && window.is_focused().unwrap_or(false) {
            let plugin_id = label.strip_prefix("plugin_").unwrap().replace('_', ".");
            return Ok(plugin_id);
        }
    }

    Err(StorageError::from("No plugin context found. Storage API must be called from within a plugin execution context."))
}

#[tauri::command]
pub async fn plugin_storage_set(
    app: AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.set(key.clone(), value);
    store.save()?;

    println!("[Storage] Set key '{}' for plugin '{}'", key, plugin_id);
    Ok(())
}

#[tauri::command]
pub async fn plugin_storage_get(
    app: AppHandle,
    key: String,
) -> Result<Option<serde_json::Value>, StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.reload()?;
    let value = store.get(&key).map(|v| v.clone());

    println!(
        "[Storage] Get key '{}' for plugin '{}': {:?}",
        key,
        plugin_id,
        value.is_some()
    );
    Ok(value)
}

#[tauri::command]
pub async fn plugin_storage_remove(
    app: AppHandle,
    key: String,
) -> Result<(), StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.delete(&key);
    store.save()?;

    println!("[Storage] Removed key '{}' for plugin '{}'", key, plugin_id);
    Ok(())
}

#[tauri::command]
pub async fn plugin_storage_clear(
    app: AppHandle,
) -> Result<(), StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.clear();
    store.save()?;

    println!("[Storage] Cleared all data for plugin '{}'", plugin_id);
    Ok(())
}

#[tauri::command]
pub async fn plugin_storage_keys(
    app: AppHandle,
) -> Result<Vec<String>, StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.reload()?;
    let keys: Vec<String> = store.keys().iter().cloned().collect();

    println!("[Storage] Got {} keys for plugin '{}'", keys.len(), plugin_id);
    Ok(keys)
}

#[tauri::command]
pub async fn plugin_storage_set_items(
    app: AppHandle,
    items: HashMap<String, serde_json::Value>,
) -> Result<(), StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;
    let items_len = items.len();

    for (key, value) in items {
        store.set(key, value);
    }
    store.save()?;

    println!(
        "[Storage] Set {} items for plugin '{}'",
        items_len, plugin_id
    );
    Ok(())
}

#[tauri::command]
pub async fn plugin_storage_get_items(
    app: AppHandle,
    keys: Vec<String>,
) -> Result<HashMap<String, serde_json::Value>, StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.reload()?;
    let mut result = HashMap::new();
    for key in keys {
        if let Some(value) = store.get(&key) {
            result.insert(key, value.clone());
        }
    }

    println!(
        "[Storage] Got {} items for plugin '{}'",
        result.len(),
        plugin_id
    );
    Ok(result)
}
