use serde::Serialize;
use tauri::Manager;

#[derive(Serialize, Clone, Debug)]
pub struct ToastPayload {
    pub message: String,
    pub kind: String,
    pub duration: Option<u64>,
}

/// 向插件 Webview 发送 Toast 通知
///
/// 通过 eval() 精准投递到已注入 svelte-sonner 的插件 Webview，
/// 跳过主窗口和其他无关上下文。
#[tauri::command]
pub async fn plugin_toast(
    app: tauri::AppHandle,
    message: String,
    kind: Option<String>,
    duration: Option<u64>,
) -> Result<(), String> {
    let payload = ToastPayload {
        message,
        kind: kind.unwrap_or_else(|| "default".to_string()),
        duration,
    };

    let json_payload = serde_json::to_string(&payload).unwrap_or_default();
    let script = format!(
        "if (window.__ONIN_SHOW_TOAST__) {{ window.__ONIN_SHOW_TOAST__({}); }}",
        json_payload
    );

    // 精准投递：只向插件内容层 Webview 发送
    // - "plugin-inline": 主窗口中的内联插件子视图
    // - "plugin_*": Window 模式下的独立插件窗口
    for webview in app.webviews().values() {
        let label = webview.label();
        if label == "plugin-inline" || label.starts_with("plugin_") {
            let _ = webview.eval(&script);
        }
    }

    Ok(())
}
