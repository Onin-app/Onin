//! 插件设置 API 处理器

use crate::js_runtime::types::InvokeResult;
use tauri::{AppHandle, Manager};

/// 注册插件设置 Schema
///
/// # 安全检查
/// 验证插件只能注册自己的设置，防止跨插件操作
pub async fn handle_register_settings_schema(
    app_handle: AppHandle,
    plugin_id: String,
    arg: serde_json::Value,
) -> InvokeResult {
    // 从 arg 中提取 pluginId 和 schema
    let plugin_id_from_arg = match arg.get("pluginId").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return InvokeResult::Err {
                error: "Missing pluginId in register_plugin_settings_schema".to_string(),
            };
        }
    };

    // 🔒 安全检查：验证插件只能注册自己的设置
    if plugin_id_from_arg != plugin_id {
        eprintln!(
            "[js_runtime] Security violation: Plugin '{}' attempted to register settings for plugin '{}'",
            plugin_id, plugin_id_from_arg
        );
        return InvokeResult::Err {
            error: format!(
                "Permission denied: Plugin can only register its own settings (expected: {}, got: {})",
                plugin_id, plugin_id_from_arg
            ),
        };
    }

    let schema_value = match arg.get("schema") {
        Some(s) => s.clone(),
        None => {
            return InvokeResult::Err {
                error: "Missing schema in register_plugin_settings_schema".to_string(),
            };
        }
    };

    // 反序列化 schema
    let schema: crate::plugin::PluginSettingsSchema = match serde_json::from_value(schema_value) {
        Ok(s) => s,
        Err(e) => {
            return InvokeResult::Err {
                error: format!("Failed to deserialize schema: {}", e),
            };
        }
    };

    // 调用 plugin_manager 的函数
    let store = app_handle.state::<crate::plugin::PluginStore>();
    match crate::plugin::register_plugin_settings_schema(
        app_handle.clone(),
        store,
        plugin_id_from_arg,
        schema,
    ) {
        Ok(_) => super::ok_null(),
        Err(e) => super::err(e),
    }
}
