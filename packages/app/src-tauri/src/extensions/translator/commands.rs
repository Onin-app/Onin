use tauri::{
    webview::WebviewBuilder, Listener, Manager, WebviewUrl,
    WindowBuilder,
};
use tauri::{LogicalPosition, LogicalSize};

/// Open Translator Window with Multi-Webview Architecture
#[tauri::command]
pub async fn open_translator_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window(&app).await
}

pub async fn open_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("translator-host") {
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

    // 2. Create UI Webview (Top 50px)
    // This loads the local Svelte route: /extensions/translator
    let _ui_webview = window
        .add_child(
            WebviewBuilder::new("translator-ui", WebviewUrl::App("extensions/translator".into()))
                .auto_resize(),
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(1000.0, 50.0),
        )
        .map_err(|e| e.to_string())?;

    // 3. Create Google Webview (Rest of the area, default visible)
    let google_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-google",
                WebviewUrl::External("https://translate.google.com/".parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // 4. Create DeepL Webview (Rest of the area, default hidden)
    let deepl_webview = window
        .add_child(
            WebviewBuilder::new(
                "translator-deepl",
                WebviewUrl::External("https://www.deepl.com/translator".parse().unwrap()),
            )
            .auto_resize(),
            LogicalPosition::new(0.0, 50.0),
            LogicalSize::new(1000.0, 750.0),
        )
        .map_err(|e| e.to_string())?;

    // Hide DeepL initially
    deepl_webview.hide().map_err(|e| e.to_string())?;

    // 5. Listen for switch event
    // The UI webview will emit "translator_switch".
    
    let google_webview_clone = google_webview.clone();
    let deepl_webview_clone = deepl_webview.clone();

    window.listen("translator_switch", move |event| {
        let payload = event.payload();
        println!("Received switch event: {}", payload);
        
        // Simple check for now. Ideally parse JSON or use specific payloads.
        let engine = if payload.contains("deepl") {
            "deepl"
        } else {
            "google"
        };

        if engine == "deepl" {
            let _ = google_webview_clone.hide();
            let _ = deepl_webview_clone.show();
            let _ = deepl_webview_clone.set_focus();
        } else {
            let _ = deepl_webview_clone.hide();
            let _ = google_webview_clone.show();
            let _ = google_webview_clone.set_focus();
        };
    });

    Ok(())
}
