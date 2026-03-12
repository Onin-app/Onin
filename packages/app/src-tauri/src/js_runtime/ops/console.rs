//! Console 操作模块
//!
//! 提供 console.log/error/warn 的 Rust 后端实现

use deno_core::op2;
use deno_core::OpState;
use tauri::{AppHandle, Emitter, Manager};

/// 同步 op：处理 console 输出
///
/// 将插件的日志输出到 Rust 控制台，并尝试发送到前端
#[op2(fast)]
pub fn op_console_log(state: &OpState, #[string] message: String) {
    println!("[Plugin Console] {}", message);

    // 尝试发送到前端
    if let Some(app_handle) = state.try_borrow::<AppHandle>() {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit(
                "plugin_console_log",
                serde_json::json!({
                    "message": message,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }),
            );
        }
    }
}
