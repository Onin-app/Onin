//! API 处理器模块
//!
//! 按功能分组的 API 处理器，负责处理具体的 API 调用
//!
//! ## 模块结构
//! - `notification`: 通知 API
//! - `request`: HTTP 请求 API
//! - `storage`: 存储 API
//! - `fs`: 文件系统 API
//! - `dialog`: 对话框 API
//! - `clipboard`: 剪贴板 API
//! - `settings`: 插件设置 API

mod clipboard;
mod dialog;
mod fs;
mod notification;
mod request;
mod settings;
mod storage;

use crate::js_runtime::types::InvokeResult;
use tauri::AppHandle;

// ============================================================================
// 通用辅助函数
// ============================================================================

/// 从 arg 中提取必需的字符串参数
///
/// # 返回
/// - `Ok(String)`: 成功提取的字符串值
/// - `Err(InvokeResult)`: 包含错误信息的结果
pub fn require_str(arg: &serde_json::Value, key: &str) -> Result<String, InvokeResult> {
    arg.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| InvokeResult::Err {
            error: format!("{} is required", key),
        })
}

/// 从 arg 中提取可选的布尔参数
pub fn get_bool(arg: &serde_json::Value, key: &str, default: bool) -> bool {
    arg.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// 包装成功结果（null 值）
pub fn ok_null() -> InvokeResult {
    InvokeResult::Ok {
        value: serde_json::Value::Null,
    }
}

/// 包装成功结果（带值）
pub fn ok_value(value: serde_json::Value) -> InvokeResult {
    InvokeResult::Ok { value }
}

/// 包装错误结果
pub fn err(message: impl Into<String>) -> InvokeResult {
    InvokeResult::Err {
        error: message.into(),
    }
}

/// 格式化错误结果
pub fn err_fmt(category: &str, error: impl std::fmt::Debug) -> InvokeResult {
    InvokeResult::Err {
        error: format!("{} error: {:?}", category, error),
    }
}

// ============================================================================
// 统一调度函数
// ============================================================================

/// 分发 API 调用到对应的处理器
///
/// 这是所有插件 API 调用的统一入口点。
/// 根据 method 名称将请求路由到对应的处理器函数。
pub async fn dispatch(
    method: &str,
    app_handle: AppHandle,
    plugin_id: String,
    arg: serde_json::Value,
) -> InvokeResult {
    match method {
        // 通知 API
        "show_notification" => notification::handle_show_notification(app_handle, arg).await,
        "is_permission_granted" => {
            notification::handle_is_permission_granted(app_handle, arg).await
        }
        "request_permission" => notification::handle_request_permission(app_handle, arg).await,

        // HTTP 请求 API
        "plugin_request" => request::handle_plugin_request(app_handle, arg).await,

        // 存储 API
        "plugin_storage_set" => storage::handle_storage_set(app_handle, arg).await,
        "plugin_storage_get" => storage::handle_storage_get(app_handle, arg).await,
        "plugin_storage_remove" => storage::handle_storage_remove(app_handle, arg).await,
        "plugin_storage_clear" => storage::handle_storage_clear(app_handle).await,
        "plugin_storage_keys" => storage::handle_storage_keys(app_handle).await,
        "plugin_storage_set_items" => storage::handle_storage_set_items(app_handle, arg).await,
        "plugin_storage_get_items" => storage::handle_storage_get_items(app_handle, arg).await,
        "plugin_storage_get_all" => storage::handle_storage_get_all(app_handle).await,
        "plugin_storage_set_all" => storage::handle_storage_set_all(app_handle, arg).await,

        // 文件系统 API
        "plugin_fs_read_file" => fs::handle_read_file(app_handle, arg).await,
        "plugin_fs_write_file" => fs::handle_write_file(app_handle, arg).await,
        "plugin_fs_exists" => fs::handle_exists(app_handle, arg).await,
        "plugin_fs_create_dir" => fs::handle_create_dir(app_handle, arg).await,
        "plugin_fs_list_dir" => fs::handle_list_dir(app_handle, arg).await,
        "plugin_fs_delete_file" => fs::handle_delete_file(app_handle, arg).await,
        "plugin_fs_delete_dir" => fs::handle_delete_dir(app_handle, arg).await,
        "plugin_fs_get_file_info" => fs::handle_get_file_info(app_handle, arg).await,
        "plugin_fs_copy_file" => fs::handle_copy_file(app_handle, arg).await,
        "plugin_fs_move_file" => fs::handle_move_file(app_handle, arg).await,

        // 对话框 API
        "plugin_dialog_message" => dialog::handle_dialog_message(app_handle, arg).await,
        "plugin_dialog_confirm" => dialog::handle_dialog_confirm(app_handle, arg).await,
        "plugin_dialog_open" => dialog::handle_dialog_open(app_handle, arg).await,
        "plugin_dialog_save" => dialog::handle_dialog_save(app_handle, arg).await,

        // 剪贴板 API
        "plugin_clipboard_read_text" => clipboard::handle_read_text(app_handle).await,
        "plugin_clipboard_write_text" => clipboard::handle_write_text(app_handle, arg).await,
        "plugin_clipboard_read_image" => clipboard::handle_read_image(app_handle).await,
        "plugin_clipboard_write_image" => clipboard::handle_write_image(app_handle, arg).await,
        "plugin_clipboard_clear" => clipboard::handle_clear(app_handle).await,
        "plugin_clipboard_get_metadata" => clipboard::handle_get_metadata(app_handle).await,

        // 插件设置 API
        "register_plugin_settings_schema" => {
            settings::handle_register_settings_schema(app_handle, plugin_id, arg).await
        }

        // 未知方法
        _ => InvokeResult::Err {
            error: "unknown method".to_string(),
        },
    }
}
