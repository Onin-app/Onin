use serde::Deserialize;
use tauri_plugin_notification::NotificationExt;

#[derive(Deserialize, Debug)]
pub struct NotificationOptions {
    pub title: String,
    pub body: Option<String>,
}

pub fn show_notification(
    app_handle: &tauri::AppHandle,
    options: NotificationOptions,
) -> Result<(), String> {
    app_handle
        .notification()
        .builder()
        .title(options.title)
        .body(options.body.unwrap_or_default())
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}
