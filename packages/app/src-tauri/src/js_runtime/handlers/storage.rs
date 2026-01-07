//! 存储 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 设置存储项
pub async fn handle_storage_set(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let key = match super::require_str(&arg, "key") {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = match arg.get("value") {
        Some(v) => v.clone(),
        None => return super::err("value is required"),
    };

    match plugin_api::storage::plugin_storage_set(app_handle, key, value).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 获取存储项
pub async fn handle_storage_get(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let key = match super::require_str(&arg, "key") {
        Ok(k) => k,
        Err(e) => return e,
    };

    match plugin_api::storage::plugin_storage_get(app_handle, key).await {
        Ok(value) => super::ok_value(value.unwrap_or(serde_json::Value::Null)),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 删除存储项
pub async fn handle_storage_remove(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let key = match super::require_str(&arg, "key") {
        Ok(k) => k,
        Err(e) => return e,
    };

    match plugin_api::storage::plugin_storage_remove(app_handle, key).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 清除所有存储项
pub async fn handle_storage_clear(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::storage::plugin_storage_clear(app_handle).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 获取所有存储键
pub async fn handle_storage_keys(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::storage::plugin_storage_keys(app_handle).await {
        Ok(keys) => super::ok_value(serde_json::json!(keys)),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 批量设置存储项
pub async fn handle_storage_set_items(
    app_handle: AppHandle,
    arg: serde_json::Value,
) -> InvokeResult {
    let items = match arg.get("items") {
        Some(items_value) => match items_value.as_object() {
            Some(obj) => obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            None => return super::err("items must be an object"),
        },
        None => return super::err("items is required"),
    };

    match plugin_api::storage::plugin_storage_set_items(app_handle, items).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("Storage", e),
    }
}

/// 批量获取存储项
pub async fn handle_storage_get_items(
    app_handle: AppHandle,
    arg: serde_json::Value,
) -> InvokeResult {
    let keys = match arg.get("keys") {
        Some(keys_value) => match keys_value.as_array() {
            Some(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            None => return super::err("keys must be an array"),
        },
        None => return super::err("keys is required"),
    };

    match plugin_api::storage::plugin_storage_get_items(app_handle, keys).await {
        Ok(items) => super::ok_value(serde_json::json!(items)),
        Err(e) => super::err_fmt("Storage", e),
    }
}
