//! 对话框 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 显示消息对话框
pub async fn handle_dialog_message(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::dialog::MessageDialogOptions>(arg.clone());

    match options_result {
        Ok(options) => match plugin_api::dialog::plugin_dialog_message(app_handle, options).await {
            Ok(_) => super::ok_null(),
            Err(e) => super::err_fmt("Dialog", e),
        },
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_dialog_message: {}", e),
        },
    }
}

/// 显示确认对话框
pub async fn handle_dialog_confirm(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::dialog::ConfirmDialogOptions>(arg.clone());

    match options_result {
        Ok(options) => {
            match plugin_api::dialog::plugin_dialog_confirm_for_app(app_handle, options).await {
                Ok(result) => super::ok_value(serde_json::json!(result)),
                Err(e) => super::err_fmt("Dialog", e),
            }
        }
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_dialog_confirm: {}", e),
        },
    }
}

/// 显示打开文件对话框
pub async fn handle_dialog_open(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::dialog::OpenDialogOptions>(arg.clone());

    match options_result {
        Ok(options) => match plugin_api::dialog::plugin_dialog_open(app_handle, options).await {
            Ok(result) => super::ok_value(result.unwrap_or(serde_json::Value::Null)),
            Err(e) => super::err_fmt("Dialog", e),
        },
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_dialog_open: {}", e),
        },
    }
}

/// 显示保存文件对话框
pub async fn handle_dialog_save(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let options_result =
        serde_json::from_value::<plugin_api::dialog::SaveDialogOptions>(arg.clone());

    match options_result {
        Ok(options) => match plugin_api::dialog::plugin_dialog_save(app_handle, options).await {
            Ok(result) => super::ok_value(
                result
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null),
            ),
            Err(e) => super::err_fmt("Dialog", e),
        },
        Err(e) => InvokeResult::Err {
            error: format!("Invalid argument for plugin_dialog_save: {}", e),
        },
    }
}
