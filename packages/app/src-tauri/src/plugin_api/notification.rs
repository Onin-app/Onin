use serde::Deserialize;
use tauri_plugin_notification::NotificationExt;

#[derive(Deserialize, Debug)]
pub struct NotificationOptions {
    pub title: String,
    pub body: Option<String>,
    pub icon: Option<String>,
    pub sound: Option<String>,
}

#[tauri::command]
pub fn is_permission_granted(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let result = app_handle
        .notification()
        .permission_state();

    result
        .map(|s| s == tauri_plugin_notification::PermissionState::Granted)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn request_permission(app_handle: tauri::AppHandle) -> Result<tauri_plugin_notification::PermissionState, String> {
    let result = app_handle
        .notification()
        .request_permission();

    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_notification(
    app_handle: tauri::AppHandle,
    options: NotificationOptions,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Maintenance note:
        // - macOS dev mode (tauri dev) can behave inconsistently with the notification plugin.
        // - We intentionally use `osascript` only in dev for stable local debugging.
        // - Packaged builds must keep using the official plugin for cross-platform consistency.
        // See: packages/app/docs/NOTIFICATION_STRATEGY.md
        if tauri::is_dev() {
            if let Err(e) = show_notification_macos_dev(&options) {
                return Err(e);
            }
            return Ok(());
        }
    }

    let mut builder = app_handle
        .notification()
        .builder()
        .title(options.title)
        .body(options.body.unwrap_or_default());

    if let Some(icon) = options.icon {
        builder = builder.icon(icon);
    }

    if let Some(sound) = options.sound {
        builder = builder.sound(sound);
    }

    builder.show().map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn show_notification_macos_dev(
    options: &NotificationOptions,
) -> Result<(), String> {
    let title = escape_for_applescript(&options.title);
    let body = escape_for_applescript(options.body.as_deref().unwrap_or(""));
    let script = if let Some(sound) = &options.sound {
        let sound = escape_for_applescript(sound);
        format!(
            "display notification \"{}\" with title \"{}\" sound name \"{}\"",
            body, title, sound
        )
    } else {
        format!("display notification \"{}\" with title \"{}\"", body, title)
    };

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript spawn failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "osascript failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "macos")]
fn escape_for_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
