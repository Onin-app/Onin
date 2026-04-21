use serde::Deserialize;
use std::str::FromStr;
use tauri::{webview::WebviewBuilder, Listener, Manager, WebviewUrl, WindowBuilder};
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

const TRANSLATOR_TOP_BAR_HEIGHT: f64 = 36.0;
#[cfg(target_os = "macos")]
const TRANSLATOR_TITLEBAR_INSET: f64 = 28.0;
#[cfg(not(target_os = "macos"))]
const TRANSLATOR_TITLEBAR_INSET: f64 = 0.0;
const TRANSLATOR_ESC_SCRIPT: &str = r#"
(() => {
  const invoke = (cmd, args = {}) => {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    if (window.__TAURI_INTERNALS__) {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    return Promise.reject(new Error("Tauri invoke bridge unavailable"));
  };

  window.addEventListener(
    "keydown",
    (event) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      invoke("close_translator_window").catch((error) => {
        console.error("[translator] Failed to close window from Escape:", error);
      });
    },
    true
  );
})();
"#;

#[derive(Deserialize)]
struct TranslatorSwitchPayload {
    engine: Option<String>,
}

struct TranslatorUrls {
    google: String,
    baidu: String,
    sougou: String,
}

fn resolve_translator_text(
    app: &tauri::AppHandle,
    text: Option<String>,
) -> Result<Option<String>, String> {
    let explicit_text = text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if explicit_text.is_some() {
        return Ok(explicit_text);
    }

    match app.clipboard().read_text() {
        Ok(clipboard_text) => {
            let trimmed = clipboard_text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(_) => Ok(None),
    }
}

fn build_translator_urls(text: Option<&str>) -> TranslatorUrls {
    let encoded = text
        .map(|value| url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>())
        .unwrap_or_default();

    let google = if text.is_some() {
        format!(
            "https://translate.google.com/?sl=auto&tl=zh-CN&text={}",
            encoded
        )
    } else {
        "https://translate.google.com/".to_string()
    };

    let baidu = if text.is_some() {
        format!("https://fanyi.baidu.com/#auto/zh/{}", encoded)
    } else {
        "https://fanyi.baidu.com".to_string()
    };

    let sougou = if text.is_some() {
        format!("https://fanyi.sogou.com/text?keyword={}", encoded)
    } else {
        "https://fanyi.sogou.com/".to_string()
    };

    TranslatorUrls {
        google,
        baidu,
        sougou,
    }
}

fn replace_webview_location(
    window: &tauri::Window,
    label: &str,
    target_url: &str,
) -> Result<(), String> {
    if let Some(webview) = window.get_webview(label) {
        let current_url = webview.url().map_err(|e| e.to_string())?.to_string();
        if current_url != target_url {
            let js_url = serde_json::to_string(target_url).map_err(|e| e.to_string())?;
            webview
                .eval(&format!("window.location.replace({});", js_url))
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn refresh_translator_webviews(window: &tauri::Window, text: Option<&str>) -> Result<(), String> {
    let urls = build_translator_urls(text);
    replace_webview_location(window, "translator-google", &urls.google)?;
    replace_webview_location(window, "translator-baidu", &urls.baidu)?;
    replace_webview_location(window, "translator-sougou", &urls.sougou)?;
    Ok(())
}

fn layout_translator_webviews(window: &tauri::Window) -> Result<(), String> {
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let titlebar_inset = (TRANSLATOR_TITLEBAR_INSET * scale_factor).round() as u32;
    let top_bar_height = (TRANSLATOR_TOP_BAR_HEIGHT * scale_factor).round() as u32;
    let webview_top = titlebar_inset.saturating_add(top_bar_height);
    let content_height = inner_size.height.saturating_sub(webview_top);

    if let Some(ui_webview) = window.get_webview("translator-ui") {
        ui_webview
            .set_bounds(Rect {
                position: PhysicalPosition::new(0, titlebar_inset as i32).into(),
                size: PhysicalSize::new(inner_size.width, top_bar_height).into(),
            })
            .map_err(|e| e.to_string())?;
    }

    for label in ["translator-google", "translator-baidu", "translator-sougou"] {
        if let Some(webview) = window.get_webview(label) {
            webview
                .set_bounds(Rect {
                    position: PhysicalPosition::new(0, webview_top as i32).into(),
                    size: PhysicalSize::new(inner_size.width, content_height).into(),
                })
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Open Translator Window with Multi-Webview Architecture
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app, None).await
}

#[tauri::command]
pub fn close_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("translator-host") {
        window.close().map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

pub async fn open_window(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    let resolved_text = resolve_translator_text(app, text)?;

    if let Some(window) = app.get_window("translator-host") {
        if resolved_text.is_some() {
            refresh_translator_webviews(&window, resolved_text.as_deref())?;
        }
        if let Err(e) = window.set_focus() {
            return Err(e.to_string());
        }
        return Ok(());
    }

    // 1. Create the Host Window (Container)
    let window = WindowBuilder::new(app, "translator-host")
        .title("Translator")
        .inner_size(1000.0, 800.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    let esc_shortcut = Shortcut::from_str(crate::window_manager::CLOSE_WINDOW_SHORTCUT_STR)
        .map_err(|e| format!("Failed to parse translator ESC shortcut: {}", e))?;

    let app_for_window_event = app.clone();
    let esc_shortcut_for_event = esc_shortcut.clone();
    let window_for_layout = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            let _ = app_for_window_event
                .global_shortcut()
                .unregister(esc_shortcut_for_event.clone());
            if let Err(e) = app_for_window_event
                .global_shortcut()
                .register(esc_shortcut_for_event.clone())
            {
                eprintln!("[translator] Failed to register ESC shortcut: {}", e);
            }
        }
        tauri::WindowEvent::Focused(false) | tauri::WindowEvent::CloseRequested { .. } => {
            if let Err(e) = app_for_window_event
                .global_shortcut()
                .unregister(esc_shortcut_for_event.clone())
            {
                let msg = e.to_string();
                if !msg.contains("not registered") {
                    eprintln!("[translator] Failed to unregister ESC shortcut: {}", msg);
                }
            }
        }
        tauri::WindowEvent::Resized(..) => {
            if let Err(e) = layout_translator_webviews(&window_for_layout) {
                eprintln!("[translator] Failed to update webview layout: {}", e);
            }
        }
        _ => {}
    });

    let _ = app.global_shortcut().unregister(esc_shortcut.clone());
    if let Err(e) = app.global_shortcut().register(esc_shortcut) {
        eprintln!(
            "[translator] Failed to register ESC shortcut immediately: {}",
            e
        );
    }

    // 2. Create UI Webview (Top 50px)
    // This loads the dedicated translator shell route.
    let titlebar_inset = TRANSLATOR_TITLEBAR_INSET;

    let _ui_webview = window
        .add_child(
            WebviewBuilder::new("translator-ui", WebviewUrl::App("/translator-shell".into()))
                .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, titlebar_inset),
            LogicalSize::new(1000.0, TRANSLATOR_TOP_BAR_HEIGHT),
        )
        .map_err(|e| e.to_string())?;

    let urls = build_translator_urls(resolved_text.as_deref());

    // 3. Create Google Webview (Rest of the area, default visible)
    let google_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-google",
                WebviewUrl::External(urls.google.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
            LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
        )
        .map_err(|e| e.to_string())?;

    // Baidu: #zh/en/text (or auto/zh)
    // https://fanyi.baidu.com/#auto/zh/text
    // Baidu
    let baidu_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-baidu",
                WebviewUrl::External(urls.baidu.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
            LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
        )
        .map_err(|e| e.to_string())?;

    // Sougou: ?text=... (need to verify)
    // https://fanyi.sogou.com/text?keyword=...
    // Sougou
    let sougou_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-sougou",
                WebviewUrl::External(urls.sougou.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, titlebar_inset + TRANSLATOR_TOP_BAR_HEIGHT),
            LogicalSize::new(1000.0, 800.0 - titlebar_inset - TRANSLATOR_TOP_BAR_HEIGHT),
        )
        .map_err(|e| e.to_string())?;

    // Hide others initially
    // Default show sougou (since it's first in the list in frontend)
    google_webview.hide().map_err(|e| e.to_string())?;
    baidu_webview.hide().map_err(|e| e.to_string())?;
    // sougou_webview is shown by default

    // 5. Listen for switch event
    // The UI webview will emit "translator_switch".

    let google_webview_clone = google_webview.clone();
    let baidu_webview_clone = baidu_webview.clone();
    let sougou_webview_clone = sougou_webview.clone();

    window.listen("translator_switch", move |event| {
        let payload = event.payload();
        let engine = serde_json::from_str::<TranslatorSwitchPayload>(payload)
            .ok()
            .and_then(|payload| payload.engine)
            .filter(|engine| matches!(engine.as_str(), "google" | "baidu" | "sougou"))
            .unwrap_or_else(|| "sougou".to_string());

        // Hide all first
        let _ = google_webview_clone.hide();
        let _ = baidu_webview_clone.hide();
        let _ = sougou_webview_clone.hide();

        match engine.as_str() {
            "google" => {
                let _ = google_webview_clone.show();
                let _ = google_webview_clone.set_focus();
            }
            "baidu" => {
                let _ = baidu_webview_clone.show();
                let _ = baidu_webview_clone.set_focus();
            }
            _ => {
                // sougou
                let _ = sougou_webview_clone.show();
                let _ = sougou_webview_clone.set_focus();
            }
        }
    });

    layout_translator_webviews(&window)?;

    Ok(())
}
