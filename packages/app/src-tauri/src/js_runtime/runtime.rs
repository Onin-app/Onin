//! JS 运行时创建和配置模块
//!
//! 负责创建和配置 Deno JsRuntime 实例

use deno_core::{JsRuntime, RuntimeOptions};
use tauri::AppHandle;

use super::ops::console::op_console_log;
use super::ops::invoke::op_invoke;

// 定义 Deno 扩展
//
// 注册所有可用的 ops 并设置初始状态
deno_core::extension!(
    onin_plugin_api,
    ops = [op_invoke, op_console_log],
    options = {
        app_handle: AppHandle,
        plugin_id: String,
    },
    state = |state, options| {
        state.put(options.app_handle);
        state.put(options.plugin_id);
    },
);

/// 全局 JS 设置代码
///
/// 重写 console 对象以使用我们的 op
const GLOBAL_SETUP_CODE: &str = r#"
    // 重写 console.log 以使用我们的 op
    globalThis.console = {
        log: (...args) => {
            const message = args.map(arg => 
                typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
            ).join(' ');
            Deno.core.ops.op_console_log(message);
        },
        error: (...args) => {
            const message = '[ERROR] ' + args.map(arg => 
                typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
            ).join(' ');
            Deno.core.ops.op_console_log(message);
        },
        warn: (...args) => {
            const message = '[WARN] ' + args.map(arg => 
                typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
            ).join(' ');
            Deno.core.ops.op_console_log(message);
        }
    };
"#;

/// 创建带有插件ID的 JS 运行时
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `plugin_id`: 插件标识符，用于权限验证和状态隔离
pub fn create_runtime_with_plugin_id(
    app_handle: &AppHandle,
    plugin_id: &str,
) -> Result<JsRuntime, String> {
    let ext = onin_plugin_api::init(app_handle.clone(), plugin_id.to_string());

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    // 设置全局对象
    runtime
        .execute_script("<global_setup>", GLOBAL_SETUP_CODE.to_string())
        .map_err(|e| e.to_string())?;

    Ok(runtime)
}
