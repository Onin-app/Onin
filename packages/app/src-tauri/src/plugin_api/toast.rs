use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Deserialize, Debug)]
pub struct ToastOptions {
    pub message: String,
    pub kind: Option<String>,
    pub duration: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ToastPayload {
    pub message: String,
    pub kind: String,
    pub duration: Option<u64>,
}

#[tauri::command]
pub async fn plugin_toast(
    window: tauri::Window,
    message: String,
    kind: Option<String>,
    duration: Option<u64>,
) -> Result<(), String> {
    let payload = ToastPayload {
        message,
        kind: kind.unwrap_or_else(|| "default".to_string()),
        duration,
    };

    // Emit the event to the window that called the command
    window
        .emit("plugin-toast", payload)
        .map_err(|e| e.to_string())
}
