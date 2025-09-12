#[cfg(target_os = "windows")]
pub mod exe_to_icon;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use tauri::command;

#[derive(serde::Serialize, Clone, Debug)]
pub enum AppOrigin {
    Hkey,
    Shortcut,
    Uwp,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub keywords: Vec<String>,
    pub path: Option<String>,
    pub icon: Option<String>,
    #[cfg(target_os = "windows")]
    pub origin: Option<AppOrigin>,
    #[cfg(not(target_os = "windows"))]
    #[serde(skip_serializing)]
    pub origin: Option<AppOrigin>,
}

// 这不再是一个Tauri命令，而是一个内部调用的、执行耗时操作的函数
pub async fn fetch_installed_apps() -> Result<Vec<AppInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        windows::get_apps().await
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_apps().await
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
