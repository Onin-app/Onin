use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, Runtime, Webview};

// 线程本地存储用于保存当前插件ID（适用于 headless 插件）
thread_local! {
    pub static CURRENT_PLUGIN_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// 设置当前线程的插件ID
pub fn set_current_plugin_id(plugin_id: String) {
    CURRENT_PLUGIN_ID.with(|id| {
        *id.borrow_mut() = Some(plugin_id);
    });
}

/// 清除当前线程的插件ID
pub fn clear_current_plugin_id() {
    CURRENT_PLUGIN_ID.with(|id| {
        *id.borrow_mut() = None;
    });
}

/// RAII Guard 用于自动管理线程本地存储的插件ID
pub struct PluginContextGuard;

impl PluginContextGuard {
    pub fn new(plugin_id: String) -> Self {
        set_current_plugin_id(plugin_id);
        PluginContextGuard
    }
}

impl Drop for PluginContextGuard {
    fn drop(&mut self) {
        clear_current_plugin_id();
    }
}

/// 获取当前执行上下文中的插件 ID
///
/// 优先级：
/// 1. 显式传入的 Webview 的 label
/// 2. 线程局部变量 (适用于 headless 插件执行器)
/// 3. InlinePluginState (当前活跃的内联插件)
/// 4. 当前获取焦点的插件窗口 label
pub fn get_current_plugin_id<R: Runtime>(
    app: &AppHandle<R>,
    webview: Option<&Webview<R>>,
) -> Result<String, String> {
    // 1. 如果提供了 webview，尝试从其 label 解析
    if let Some(wv) = webview {
        let label = wv.label();
        if let Some(id) = parse_plugin_id_from_label(label) {
            return Ok(id);
        }

        // 如果是内联插件 webview，则尝试从状态获取
        if label == "plugin-inline" {
            if let Some(inline_state) = app.try_state::<crate::plugin::InlinePluginState>() {
                if let Ok(id_lock) = inline_state.current_plugin_id.lock() {
                    if let Some(id) = id_lock.as_ref() {
                        return Ok(id.clone());
                    }
                }
            }
        }
    }

    // 2. 尝试从线程局部变量获取（适用于通过 executor 执行的 JS 插件）
    if let Some(id) = CURRENT_PLUGIN_ID.with(|id| id.borrow().clone()) {
        return Ok(id);
    }

    // 3. 尝试从内联插件状态获取
    if let Some(inline_state) = app.try_state::<crate::plugin::InlinePluginState>() {
        // 如果内联插件可见，则尝试获取其 ID
        if inline_state.is_visible.load(Ordering::Relaxed) {
            if let Ok(id_lock) = inline_state.current_plugin_id.lock() {
                if let Some(id) = id_lock.as_ref() {
                    return Ok(id.clone());
                }
            }
        }
    }

    // 4. 尝试从当前获取焦点的 WebView 解析
    for wv in app.webviews().values() {
        if wv.window().is_focused().unwrap_or(false) {
            if let Some(id) = parse_plugin_id_from_label(wv.label()) {
                return Ok(id);
            }
        }
    }

    // 5. 尝试从记录的活跃窗口获取 (处理失焦瞬间触发的命令)
    if let Some(active_window_state) = app.try_state::<crate::plugin::types::ActivePluginWindow>() {
        if let Ok(active_lock) = active_window_state.0.lock() {
            if let Some(label) = active_lock.as_ref() {
                if let Some(id) = parse_plugin_id_from_label(label) {
                    return Ok(id);
                }
            }
        }
    }

    Err("Could not determine plugin ID from context".to_string())
}

/// 从窗口 label 解析插件 ID
/// 使用 HEX 编码以确保唯一性并避免 _ 到 . 的误判风险
/// plugin_hex(id) -> id
pub fn parse_plugin_id_from_label(label: &str) -> Option<String> {
    if label.starts_with("plugin_") {
        let hex_part = label.strip_prefix("plugin_").unwrap();
        if hex_part.is_empty() || hex_part.len() % 2 != 0 {
            return None;
        }

        // 尝试解析 HEX
        let mut bytes = Vec::new();
        for i in (0..hex_part.len()).step_by(2) {
            if let Ok(byte) = u8::from_str_radix(&hex_part[i..i + 2], 16) {
                bytes.push(byte);
            } else {
                return None;
            }
        }

        String::from_utf8(bytes).ok()
    } else {
        None
    }
}

/// 将插件 ID 转换为窗口 label 使用的 HEX 格式
pub fn make_label_from_plugin_id(plugin_id: &str) -> String {
    let hex_id: String = plugin_id
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    format!("plugin_{}", hex_id)
}
