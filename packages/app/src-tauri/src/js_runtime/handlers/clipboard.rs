//! 剪贴板 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 读取剪贴板文本
pub async fn handle_read_text(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::clipboard::plugin_clipboard_read_text(app_handle).await {
        Ok(text) => super::ok_value(
            text.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        ),
        Err(e) => super::err_fmt("Clipboard", e),
    }
}

/// 写入剪贴板文本
pub async fn handle_write_text(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::clipboard::WriteTextOptions>(arg.clone());

    match options_result {
        Ok(options) => {
            match plugin_api::clipboard::plugin_clipboard_write_text(app_handle, options).await {
                Ok(_) => super::ok_null(),
                Err(e) => super::err_fmt("Clipboard", e),
            }
        }
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_clipboard_write_text: {}", e),
        },
    }
}

/// 读取剪贴板图片
pub async fn handle_read_image(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::clipboard::plugin_clipboard_read_image(app_handle).await {
        Ok(image) => super::ok_value(
            image
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        ),
        Err(e) => super::err_fmt("Clipboard", e),
    }
}

/// 写入剪贴板图片
pub async fn handle_write_image(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::clipboard::WriteImageOptions>(arg.clone());

    match options_result {
        Ok(options) => {
            match plugin_api::clipboard::plugin_clipboard_write_image(app_handle, options).await {
                Ok(_) => super::ok_null(),
                Err(e) => super::err_fmt("Clipboard", e),
            }
        }
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_clipboard_write_image: {}", e),
        },
    }
}

/// 清空剪贴板
pub async fn handle_clear(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::clipboard::plugin_clipboard_clear(app_handle).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("Clipboard", e),
    }
}

/// 获取剪贴板元数据
pub async fn handle_get_metadata(app_handle: AppHandle) -> InvokeResult {
    match plugin_api::clipboard::plugin_clipboard_get_metadata(app_handle).await {
        Ok(metadata) => {
            super::ok_value(serde_json::to_value(metadata).unwrap_or(serde_json::Value::Null))
        }
        Err(e) => super::err_fmt("Clipboard", e),
    }
}
