//! HTTP 请求 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 处理 HTTP 请求
pub async fn handle_plugin_request(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    // 处理两种格式：直接的 RequestOptions 或 {"options": RequestOptions}
    let request_options = if let Some(options) = arg.get("options") {
        options.clone()
    } else {
        arg.clone()
    };

    let options_result =
        serde_json::from_value::<plugin_api::request::RequestOptions>(request_options.clone());

    match options_result {
        Ok(options) => match plugin_api::request::make_request(app_handle, options).await {
            Ok(response) => super::ok_value(response),
            Err(e) => super::err(e),
        },
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_request: {}", e),
        },
    }
}
