use serde::Serialize;
use tauri::{Emitter, LogicalPosition, Manager, WebviewUrl, WebviewWindowBuilder};

const TOAST_OVERLAY_LABEL: &str = "toast-overlay";
const TOAST_WIDTH: f64 = 420.0;
const TOAST_HEIGHT: f64 = 112.0;
const TOAST_TOP_MARGIN: f64 = 76.0;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToastOverlayPayload {
    message: String,
    kind: String,
    duration: u64,
}

#[tauri::command]
pub async fn show_toast_overlay(
    app: tauri::AppHandle,
    message: String,
    kind: Option<String>,
    duration: Option<u64>,
) -> Result<(), String> {
    let payload = ToastOverlayPayload {
        message,
        kind: kind.unwrap_or_else(|| "success".to_string()),
        duration: duration.unwrap_or(1400),
    };

    let position = toast_position(&app).await;

    if let Some(window) = app.get_webview_window(TOAST_OVERLAY_LABEL) {
        if let Some((x, y)) = position {
            let _ = window.set_position(LogicalPosition::new(x, y));
        }
        let _ = window.show();
        let window_clone = window.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            let _ = window_clone.emit("toast_overlay_show", payload);
        });
        return Ok(());
    }

    let route = format!("/toast-overlay?{}", toast_query(&payload));
    let window =
        WebviewWindowBuilder::new(&app, TOAST_OVERLAY_LABEL, WebviewUrl::App(route.into()))
            .title("Onin Toast")
            .inner_size(TOAST_WIDTH, TOAST_HEIGHT)
            .decorations(false)
            .transparent(true)
            .visible(false)
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .focused(false)
            .shadow(false);

    let window = if let Some((x, y)) = position {
        window.position(x, y)
    } else {
        window
    }
    .build()
    .map_err(|err| err.to_string())?;
    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = window_clone.show();
    });

    Ok(())
}

fn toast_query(payload: &ToastOverlayPayload) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("message", &payload.message);
    serializer.append_pair("kind", &payload.kind);
    serializer.append_pair("duration", &payload.duration.to_string());
    serializer.finish()
}

async fn toast_position(app: &tauri::AppHandle) -> Option<(f64, f64)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let position = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();

    let logical_x = position.x as f64 / scale + (size.width as f64 / scale - TOAST_WIDTH) / 2.0;
    let logical_y = position.y as f64 / scale + TOAST_TOP_MARGIN;

    Some((logical_x.round(), logical_y.round()))
}
