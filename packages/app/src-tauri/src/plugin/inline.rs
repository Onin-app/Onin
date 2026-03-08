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
use tauri::State; // Ensure State is imported

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
    
    let window = app.get_window("main").ok_or("Main window not found")?;

    // Check if webview exists
    if let Some(webview) = window.get_webview("plugin-inline") {
        // Update URL if different (Normalized comparison)
        let current_url_str = webview.url().unwrap().to_string();
        let is_different = if let (Ok(u1), Ok(u2)) = (Url::parse(&current_url_str), Url::parse(&url)) {
            // Compare normalized URLs (ignores trailing slashes, etc.)
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

        // Update bounds
        webview
            .set_bounds(tauri::Rect {
                position: PhysicalPosition::new(rect.x as i32, rect.y as i32).into(),
                size: PhysicalSize::new(rect.width as u32, rect.height as u32).into(),
            })
            .map_err(|e| e.to_string())?;

        // Show
        webview.show().map_err(|e| e.to_string())?;
        webview.set_focus().map_err(|e| e.to_string())?;
    } else {
        println!("[plugin/inline] Creating webview with URL: {}", url);
        // Create new webview
        let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
            WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?)
        } else {
            WebviewUrl::App(url.into())
        };

        // Minimal configuration to avoid potential Windows hang
        let webview_builder = WebviewBuilder::new("plugin-inline", webview_url)
            .devtools(true)
            .initialization_script(
                r#"
                window.addEventListener('keydown', (e) => {
                    if (e.key === 'Escape') {
                        // Use standard invoke if available (for internal URLs)
                        if (window.__TAURI__ && window.__TAURI__.core) {
                            window.__TAURI__.core.invoke('close_inline_plugin').catch(() => {});
                        } 
                // Fallback to IPC (works for external if configured or just raw)
                        // This sends a message to the rust ipc_handler
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
            // .ipc_handler(|app, req| { ... }) // REMOVED: method not found on WebviewBuilder

        // println!("[plugin/inline] Adding child to window...");
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

        // Initialization might take a moment.
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
        // println!("[plugin/inline] Ignoring bounds update because plugin is hidden");
        return Ok(());
    }

    // println!("[plugin/inline] Updating bounds: {:?}", rect);
    let window = app.get_window("main").ok_or("Main window not found")?;
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

    let window = app.get_window("main").ok_or("Main window not found")?;
    if let Some(webview) = window.get_webview("plugin-inline") {
        webview.hide().map_err(|e| e.to_string())?;
        // Enforce 0x0
        webview.set_bounds(tauri::Rect {
            position: PhysicalPosition::new(0, 0).into(),
            size: PhysicalSize::new(0, 0).into(),
        }).ok();
    }
    Ok(())
}

#[tauri::command]
pub fn close_inline_plugin<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, InlinePluginState>,
) -> Result<(), String> {
    state.is_visible.store(false, Ordering::Relaxed);

    let window = app.get_window("main").ok_or("Main window not found")?;
    
    if let Some(webview) = window.get_webview("plugin-inline") {
        // Revert to close() (destroy) mechanism.
        // The original freeze issue was due to a deadlock in window_manager, which is now fixed.
        // Destroying the webview ensures no "ghost" overlay remains.
        webview.close().map_err(|e| e.to_string())?;
    }

    // Restore focus to main window explicitly
    window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn send_inline_plugin_message<R: Runtime>(
    app: AppHandle<R>,
    message: serde_json::Value,
) -> Result<(), String> {
    // let main_window = app.get_window("main").ok_or("Main window not found")?; // UNUSED

    if let Some(webview) = app.get_webview("plugin-inline") {
        let json_message = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        webview
            .eval(&format!("window.postMessage({}, '*')", json_message))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
