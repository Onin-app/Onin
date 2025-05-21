#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use tauri::command;

#[command]
pub fn get_installed_apps() -> Result<Vec<String>, String> {
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
