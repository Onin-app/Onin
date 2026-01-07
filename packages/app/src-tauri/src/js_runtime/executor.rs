//! JS 代码执行模块
//!
//! 提供执行任意 JS 代码的功能

use deno_core::PollEventLoopOptions;
use tauri::AppHandle;

use super::runtime::create_runtime_with_plugin_id;

/// 执行 JS 代码
///
/// 支持多种代码格式：
/// - ES 模块（包含 import 语句）
/// - IIFE（立即执行函数表达式）
/// - 普通代码（会自动包装为 async IIFE）
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `js_code`: 要执行的 JS 代码
/// - `plugin_id`: 可选的插件ID，用于权限验证
pub async fn execute_js(
    app_handle: &AppHandle,
    js_code: &str,
    plugin_id: Option<&str>,
) -> Result<(), String> {
    // 使用 plugin_id 创建运行时，这样 OpState 中就有正确的 plugin_id
    let plugin_id_str = plugin_id.unwrap_or("");
    let mut runtime = create_runtime_with_plugin_id(app_handle, plugin_id_str)?;

    // 检测代码格式并进行适当的包装
    let wrapped_code = wrap_code(js_code, plugin_id);

    // 执行插件代码
    let result = runtime.execute_script("<plugin>", wrapped_code);
    match result {
        Ok(_) => runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// 包装 JS 代码
///
/// 根据代码类型进行适当的包装：
/// - ES 模块：转换为动态 import
/// - IIFE：直接使用，但注入 plugin_id
/// - 普通代码：包装为 async IIFE
fn wrap_code(js_code: &str, plugin_id: Option<&str>) -> String {
    // 检测代码是否已经是 IIFE 格式
    let is_iife =
        js_code.trim_start().starts_with("(function") || js_code.trim_start().starts_with("(()");

    // 检测代码是否包含 import 语句
    let has_import =
        js_code.contains("import ") && (js_code.contains("from \"") || js_code.contains("from '"));

    if has_import {
        wrap_es_module(js_code, plugin_id)
    } else if is_iife {
        wrap_iife(js_code, plugin_id)
    } else {
        wrap_async_iife(js_code, plugin_id)
    }
}

/// 包装 ES 模块代码
fn wrap_es_module(js_code: &str, plugin_id: Option<&str>) -> String {
    // 将 ES 模块代码转换为 data URL
    let code_with_id = if let Some(id) = plugin_id {
        format!("globalThis.__PLUGIN_ID__ = '{}';\\n{}", id, js_code)
    } else {
        js_code.to_string()
    };

    // 使用 data URL 和动态 import
    let data_url = format!(
        "data:text/javascript;base64,{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            code_with_id.as_bytes()
        )
    );

    format!(
        "(async () => {{\n  await import('{}');\n}})().catch(err => console.error('Plugin module error:', err));",
        data_url
    )
}

/// 包装 IIFE 代码
fn wrap_iife(js_code: &str, plugin_id: Option<&str>) -> String {
    if let Some(id) = plugin_id {
        format!("globalThis.__PLUGIN_ID__ = '{}';\\n{}", id, js_code)
    } else {
        js_code.to_string()
    }
}

/// 包装普通代码为 async IIFE
fn wrap_async_iife(js_code: &str, plugin_id: Option<&str>) -> String {
    if let Some(id) = plugin_id {
        format!(
            "(async () => {{\n  globalThis.__PLUGIN_ID__ = '{}';\\n{}\n}})().catch(err => console.error('Plugin error:', err));",
            id, js_code
        )
    } else {
        format!(
            "(async () => {{\n{}\n}})().catch(err => console.error('Plugin error:', err));",
            js_code
        )
    }
}
