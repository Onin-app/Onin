use serde::Deserialize;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewBuilder, WebviewUrl,
};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;

use super::types::PluginStore;

/// 编译期嵌入的插件运行时注入层（Toast + Bridge API）
/// 由 packages/onin-inject 编译产出
const PLUGIN_INJECT_SCRIPT: &str = include_str!("../../templates/onin-inject.js");

pub struct InlinePluginState {
    pub is_visible: AtomicBool,
    pub current_plugin_id: std::sync::Mutex<Option<String>>,
}

impl Default for InlinePluginState {
    fn default() -> Self {
        Self {
            is_visible: AtomicBool::new(false),
            current_plugin_id: std::sync::Mutex::new(None),
        }
    }
}

fn resolve_host_window<R: Runtime>(app: &AppHandle<R>) -> Result<tauri::Window<R>, String> {
    app.get_window("main")
        .or_else(|| app.windows().values().next().cloned())
        .ok_or_else(|| {
            let labels = app.windows().keys().cloned().collect::<Vec<_>>().join(", ");
            format!("Main window not found, available windows: [{}]", labels)
        })
}
#[tauri::command]
pub async fn show_inline_plugin<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, InlinePluginState>,
    store: State<'_, PluginStore>,
    url: String,
    plugin_id: String,
    rect: Rect,
) -> Result<(), String> {
    state.is_visible.store(true, Ordering::Relaxed);
    {
        let mut id_lock = state.current_plugin_id.lock().unwrap();
        *id_lock = Some(plugin_id.clone());
    }

    // 获取版本号以进行注入
    let version = {
        let store_lock = store.0.lock().unwrap();
        super::types::find_plugin_by_id(&store_lock, &plugin_id)
            .map(|p| p.manifest.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string())
    };

    let window = resolve_host_window(&app)?;

    if let Some(webview) = window.get_webview("plugin-inline") {
        let current_url_str = webview.url().unwrap().to_string();
        let is_different =
            if let (Ok(u1), Ok(u2)) = (Url::parse(&current_url_str), Url::parse(&url)) {
                u1.as_str().trim_end_matches('/') != u2.as_str().trim_end_matches('/')
            } else {
                current_url_str != url
            };

        if is_different {
            webview
                .eval(&format!("window.location.replace('{}')", url))
                .map_err(|e| e.to_string())?;
        }

        webview
            .set_bounds(tauri::Rect {
                position: PhysicalPosition::new(rect.x as i32, rect.y as i32).into(),
                size: PhysicalSize::new(rect.width as u32, rect.height as u32).into(),
            })
            .map_err(|e| e.to_string())?;

        webview.show().map_err(|e| e.to_string())?;
        webview.set_focus().map_err(|e| e.to_string())?;
    } else {
        let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
            WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?)
        } else {
            WebviewUrl::App(url.into())
        };

        // 注入运行时信息和 Toast/Bridge 脚本
        let runtime_script = format!(
            r#"
            window.__ONIN_RUNTIME__ = {{
                mode: "inline",
                pluginId: "{id}",
                version: "{version}",
                mainWindowLabel: "main"
            }};
            {inject}
            "#,
            id = plugin_id,
            version = version,
            inject = PLUGIN_INJECT_SCRIPT,
        );

        let webview_builder = WebviewBuilder::new("plugin-inline", webview_url)
            .devtools(true)
            .initialization_script(&runtime_script)
            .on_page_load(|webview, _payload| {
                use tauri::Emitter;
                if let Err(e) = webview.window().emit("plugin-inline-loaded", ()) {
                    eprintln!("[plugin/inline] Failed to emit plugin-inline-loaded: {}", e);
                }
            });

        window
            .add_child(
                webview_builder,
                tauri::Position::Physical(PhysicalPosition::new(rect.x as i32, rect.y as i32)),
                tauri::Size::Physical(PhysicalSize::new(rect.width as u32, rect.height as u32)),
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
#[tauri::command]
pub fn update_inline_plugin_bounds<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, InlinePluginState>,
    rect: Rect,
) -> Result<(), String> {
    if !state.is_visible.load(Ordering::Relaxed) {
        return Ok(());
    }

    let window = resolve_host_window(&app)?;
    if let Some(webview) = window.get_webview("plugin-inline") {
        webview
            .set_bounds(tauri::Rect {
                position: PhysicalPosition::new(rect.x as i32, rect.y as i32).into(),
                size: PhysicalSize::new(rect.width as u32, rect.height as u32).into(),
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub fn hide_inline_plugin<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, InlinePluginState>,
) -> Result<(), String> {
    state.is_visible.store(false, Ordering::Relaxed);
    {
        let mut id_lock = state.current_plugin_id.lock().map_err(|e| e.to_string())?;
        *id_lock = None;
    }

    let window = resolve_host_window(&app)?;
    if let Some(webview) = window.get_webview("plugin-inline") {
        webview.hide().map_err(|e| e.to_string())?;
        webview
            .set_bounds(tauri::Rect {
                position: PhysicalPosition::new(0, 0).into(),
                size: PhysicalSize::new(0, 0).into(),
            })
            .ok();
    }
    Ok(())
}
#[tauri::command]
pub fn close_inline_plugin<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, InlinePluginState>,
) -> Result<(), String> {
    state.is_visible.store(false, Ordering::Relaxed);
    {
        let mut id_lock = state.current_plugin_id.lock().map_err(|e| e.to_string())?;
        *id_lock = None;
    }

    let window = resolve_host_window(&app)?;

    if let Some(webview) = window.get_webview("plugin-inline") {
        webview.close().map_err(|e| e.to_string())?;
    }

    window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}
#[tauri::command]
pub fn post_inline_plugin_message<R: Runtime>(
    app: AppHandle<R>,
    message: serde_json::Value,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview("plugin-inline") {
        let json_message = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        webview
            .eval(&format!("window.postMessage({}, '*')", json_message))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_inline_plugin_devtools<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(webview) = app.get_webview("plugin-inline") {
        #[cfg(debug_assertions)]
        webview.open_devtools();
        Ok(())
    } else {
        Err("插件视图未找到".to_string())
    }
}
#[tauri::command]
pub fn reload_inline_plugin<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(webview) = app.get_webview("plugin-inline") {
        webview
            .eval("window.location.reload()")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_inline_plugin<R: Runtime>(
    app: AppHandle<R>,
    store: State<'_, PluginStore>,
    state: State<'_, InlinePluginState>,
) -> Result<(), String> {
    let (plugin_id, url, rect) = {
        let id_lock = state.current_plugin_id.lock().unwrap();
        if let Some(id) = id_lock.as_ref() {
            if let Some(webview) = app.get_webview("plugin-inline") {
                let r = webview.bounds().map_err(|e| e.to_string())?;
                let (x, y) = match r.position {
                    tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                    tauri::Position::Logical(l) => (l.x, l.y),
                };
                let (width, height) = match r.size {
                    tauri::Size::Physical(s) => (s.width as f64, s.height as f64),
                    tauri::Size::Logical(l) => (l.width, l.height),
                };
                let u = webview.url().map_err(|e| e.to_string())?.to_string();
                (
                    Some(id.clone()),
                    u,
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                )
            } else {
                return Err("内联插件未运行".to_string());
            }
        } else {
            return Err("未找到当前运行的插件 ID".to_string());
        }
    };

    if let Some(id) = plugin_id {
        // 1. 关闭现有内联插件
        close_inline_plugin(app.clone(), state.clone())?;

        // 2. 等待清理完成
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // 3. 重新打开
        show_inline_plugin(app, state, store, url, id, rect).await?;
    }
    Ok(())
}
