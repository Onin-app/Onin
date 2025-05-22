#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use tauri::command;

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: String,
    pub path: Option<String>,
}

#[command]
pub fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        windows::get_apps()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_apps()
    }

    #[cfg(target_os = "linux")]
    {
        linux::get_apps()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform.".to_string())
    }
}

#[command]
pub fn open_app(path: String, window: tauri::WebviewWindow) -> Result<(), String> {
    println!("Opening app: {path}");

    #[cfg(target_os = "windows")]
    {
        windows::open_app(&path)?;
    }

    #[cfg(target_os = "macos")]
    {
        macos::open_app(&path)?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::open_app(&path)?;
    }

    window.hide().map_err(|e| e.to_string())?;

    Ok(())
}
