//! 通知 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 处理显示通知的请求
pub async fn handle_show_notification(
    app_handle: AppHandle,
    arg: serde_json::Value,
) -> InvokeResult {
    // 首先，尝试直接从 Value 反序列化
    let options_result =
        serde_json::from_value::<plugin_api::notification::NotificationOptions>(arg.clone());

    // 如果失败，并且 arg 是一个字符串，则尝试将该字符串作为 JSON 解析
    let final_options = match options_result {
        Ok(options) => Ok(options),
        Err(_) => {
            if let Some(s) = arg.as_str() {
                serde_json::from_str(s)
            } else {
                // 提供类型注解给编译器
                let err_result: Result<plugin_api::notification::NotificationOptions, _> =
                    serde_json::from_str("");
                Err(err_result.unwrap_err())
            }
        }
    };

    match final_options {
        Ok(options) => match plugin_api::notification::show_notification(app_handle, options) {
            Ok(_) => super::ok_null(),
            Err(e) => super::err(e),
        },
        Err(e) => InvokeResult::Err {
            error: format!(
                "Invalid argument for show_notification: {}. Original arg: {}",
                e, arg
            ),
        },
    }
}

/// 处理检查通知权限的请求
pub async fn handle_is_permission_granted(
    app_handle: AppHandle,
    _arg: serde_json::Value,
) -> InvokeResult {
    match plugin_api::notification::is_permission_granted(app_handle) {
        Ok(granted) => super::ok_value(serde_json::json!(granted)),
        Err(e) => super::err(e),
    }
}

/// 处理请求通知权限的请求
pub async fn handle_request_permission(
    app_handle: AppHandle,
    _arg: serde_json::Value,
) -> InvokeResult {
    match plugin_api::notification::request_permission(app_handle).await {
        Ok(status) => super::ok_value(serde_json::json!(status)),
        Err(e) => super::err(e),
    }
}
