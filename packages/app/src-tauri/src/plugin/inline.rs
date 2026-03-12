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
    url: String,
    plugin_id: String,
    rect: Rect,
) -> Result<(), String> {
    state.is_visible.store(true, Ordering::Relaxed);
    {
        let mut id_lock = state.current_plugin_id.lock().unwrap();
        *id_lock = Some(plugin_id);
    }

    let window = resolve_host_window(&app)?;

    if let Some(webview) = window.get_webview("plugin-inline") {
        let current_url_str = webview.url().unwrap().to_string();
        let is_different = if let (Ok(u1), Ok(u2)) = (Url::parse(&current_url_str), Url::parse(&url)) {
            u1.as_str().trim_end_matches('/') != u2.as_str().trim_end_matches('/')
        } else {
            current_url_str != url
        };

        if is_different {
            println!("[plugin/inline] URL changed from {} to {}, reloading", current_url_str, url);
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
        println!("[plugin/inline] Creating webview with URL: {}", url);
        let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
            WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?)
        } else {
            WebviewUrl::App(url.into())
        };

        let webview_builder = WebviewBuilder::new("plugin-inline", webview_url)
            .devtools(true)
            .initialization_script(
                r#"
                window.addEventListener('keydown', (e) => {
                    if (e.key === 'Escape') {
                        if (window.__TAURI__ && window.__TAURI__.core) {
                            window.__TAURI__.core.invoke('close_inline_plugin').catch(() => {});
                        }

                        if (window.__TAURI_IPC__) {
                            window.__TAURI_IPC__('escape_pressed');
                        }
                    }
                });
                "#,
            )
            .on_page_load(|webview, _payload| {
                println!("[plugin/inline] Page loaded, emitting event");
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
            .map_err(|e| {
                println!("[plugin/inline] ERROR adding child: {}", e);
                e.to_string()
            })?;

        println!("[plugin/inline] Child added successfully.");
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
        webview.open_devtools();
        Ok(())
    } else {
        Err("插件视图未找到".to_string())
    }
}

