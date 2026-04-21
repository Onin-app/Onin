use std::str::FromStr;
use serde::Deserialize;
use tauri::{webview::WebviewBuilder, Listener, Manager, WebviewUrl, WindowBuilder};
use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

const TRANSLATOR_TOP_BAR_HEIGHT: f64 = 36.0;
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

fn layout_translator_webviews(window: &tauri::Window) -> Result<(), String> {
    let inner_size = window.inner_size().map_err(|e| e.to_string())?;
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let top_bar_height = (TRANSLATOR_TOP_BAR_HEIGHT * scale_factor).round() as u32;
    let content_height = inner_size.height.saturating_sub(top_bar_height);

    if let Some(ui_webview) = window.get_webview("translator-ui") {
        ui_webview
            .set_bounds(Rect {
                position: PhysicalPosition::new(0, 0).into(),
                size: PhysicalSize::new(inner_size.width, top_bar_height).into(),
            })
            .map_err(|e| e.to_string())?;
    }

    for label in [
        "translator-google",
        "translator-baidu",
        "translator-sougou",
    ] {
        if let Some(webview) = window.get_webview(label) {
            webview
                .set_bounds(Rect {
                    position: PhysicalPosition::new(0, top_bar_height as i32).into(),
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
    if let Some(window) = app.get_window("translator-host") {
        if let Err(e) = window.set_focus() {
            return Err(e.to_string());
        }
        // If window exists and text is provided, we might want to update the tabs.
        // But for now, let's just focus.
        // TODO: Handle updating text in existing window via eval or reload.
        return Ok(());
    }

    // 1. Create the Host Window (Container)
    let window = WindowBuilder::new(app, "translator-host")
        .title("Translator")
        .inner_size(1000.0, 800.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    let esc_shortcut =
        Shortcut::from_str(crate::window_manager::CLOSE_WINDOW_SHORTCUT_STR).map_err(|e| {
            format!("Failed to parse translator ESC shortcut: {}", e)
        })?;

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
        eprintln!("[translator] Failed to register ESC shortcut immediately: {}", e);
    }

    // 2. Create UI Webview (Top 50px)
    // This loads the dedicated translator shell route.
    let _ui_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-ui",
                WebviewUrl::App("/translator-shell".into()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1000.0, TRANSLATOR_TOP_BAR_HEIGHT),
        )
        .map_err(|e| e.to_string())?;

    let url_encoded_text = text.as_deref().unwrap_or("").to_string(); // Simple for now, need proper encoding
    let encoded =
        url::form_urlencoded::byte_serialize(url_encoded_text.as_bytes()).collect::<String>();

    // Helper to append query (simplified)
    // Google: ?text=...
    let google_url = if let Some(_) = &text {
        format!(
            "https://translate.google.com/?sl=auto&tl=zh-CN&text={}",
            encoded
        )
    } else {
        "https://translate.google.com/".to_string()
    };

    // 3. Create Google Webview (Rest of the area, default visible)
    let google_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-google",
                WebviewUrl::External(google_url.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Baidu: #zh/en/text (or auto/zh)
    // https://fanyi.baidu.com/#auto/zh/text
    let baidu_url = if let Some(_) = &text {
        format!("https://fanyi.baidu.com/#auto/zh/{}", encoded)
    } else {
        "https://fanyi.baidu.com".to_string()
    };

    // Baidu
    let baidu_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-baidu",
                WebviewUrl::External(baidu_url.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Sougou: ?text=... (need to verify)
    // https://fanyi.sogou.com/text?keyword=...
    let sougou_url = if let Some(_) = &text {
        format!("https://fanyi.sogou.com/text?keyword={}", encoded)
    } else {
        "https://fanyi.sogou.com/".to_string()
    };

    // Sougou
    let sougou_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-sougou",
                WebviewUrl::External(sougou_url.parse().unwrap()),
            )
            .initialization_script(TRANSLATOR_ESC_SCRIPT),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
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
