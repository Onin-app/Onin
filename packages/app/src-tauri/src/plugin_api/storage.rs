use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_store::{Error, StoreBuilder};

// Plugin ID retrieval now handled by crate::plugin::context


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
    crate::plugin::context::get_current_plugin_id(app, None)
        .map_err(|e| StorageError::from(e))
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

#[tauri::command]
pub async fn plugin_storage_get_all(
    app: AppHandle,
) -> Result<HashMap<String, serde_json::Value>, StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.reload()?;
    let mut result = HashMap::new();
    for (key, value) in store.entries() {
        result.insert(key.clone(), value.clone());
    }

    println!(
        "[Storage] Got all {} items for plugin '{}'",
        result.len(),
        plugin_id
    );
    Ok(result)
}

#[tauri::command]
pub async fn plugin_storage_set_all(
    app: AppHandle,
    data: HashMap<String, serde_json::Value>,
) -> Result<(), StorageError> {
    let plugin_id = get_current_plugin_id(&app)?;
    let store_path = get_plugin_store_path(&plugin_id);
    let store = StoreBuilder::new(&app, store_path).build()?;

    store.clear();
    let data_len = data.len();
    for (key, value) in data {
        store.set(key, value);
    }
    store.save()?;

    println!(
        "[Storage] Replaced with {} items for plugin '{}'",
        data_len, plugin_id
    );
    Ok(())
}
