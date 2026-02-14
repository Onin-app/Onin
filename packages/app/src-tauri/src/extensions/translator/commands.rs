use tauri::{
    webview::WebviewBuilder, Listener, Manager, WebviewUrl,
    WindowBuilder,
};
use tauri::{LogicalPosition, LogicalSize};

/// Open Translator Window with Multi-Webview Architecture
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app, None).await
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

    // 2. Create UI Webview (Top 50px)
    // This loads the local Svelte route: /extensions/translator
    let _ui_webview = window
        .add_child(
            WebviewBuilder::new("translator-ui", WebviewUrl::App("/extensions/translator".into()))
                .auto_resize(),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1000.0, 50.0),
        )
        .map_err(|e| e.to_string())?;

    let url_encoded_text = text.as_deref().unwrap_or("").to_string(); // Simple for now, need proper encoding
    let encoded = url::form_urlencoded::byte_serialize(url_encoded_text.as_bytes()).collect::<String>();
    
    // Helper to append query (simplified)
    // Google: ?text=...
    let google_url = if let Some(_) = &text {
        format!("https://translate.google.com/?sl=auto&tl=zh-CN&text={}", encoded)
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
            .auto_resize(),
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
            .auto_resize(),
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
            .auto_resize(),
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
        println!("Received switch event: {}", payload);
        
        // Simple check. ideally parse JSON.
        // Payload format: {"engine":"..."}
        
        let engine = if payload.contains("google") {
            "google"
        } else if payload.contains("baidu") {
            "baidu"
        } else {
            "sougou"
        };
        
        // Hide all first
        let _ = google_webview_clone.hide();
        let _ = baidu_webview_clone.hide();
        let _ = sougou_webview_clone.hide();

        match engine {
            "google" => {
                let _ = google_webview_clone.show();
                let _ = google_webview_clone.set_focus();
            }
             "baidu" => {
                let _ = baidu_webview_clone.show();
                let _ = baidu_webview_clone.set_focus();
            }
            _ => { // sougou
                let _ = sougou_webview_clone.show();
                let _ = sougou_webview_clone.set_focus();
            }
        }
    });

    Ok(())
}
