use tauri::{App, AppHandle, WebviewWindow, Window};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
use unsupported as platform;
#[cfg(target_os = "windows")]
use windows as platform;

pub fn setup(app: &mut App) {
    platform::setup(app);
}

pub fn capture_previous_foreground(app: &AppHandle) {
    platform::capture_previous_foreground(app);
}

pub fn restore_previous_foreground(app: &AppHandle) {
    platform::restore_previous_foreground(app);
}

pub fn focus_webview_window(window: &WebviewWindow) {
    platform::focus_webview_window(window);
}

pub fn focus_window(window: &Window) {
    platform::focus_window(window);
}

#[cfg(target_os = "macos")]
pub fn is_frontmost_self(app: &AppHandle) -> bool {
    platform::is_frontmost_self(app)
}

#[cfg(target_os = "macos")]
pub fn previous_bundle_id(app: &AppHandle) -> Option<String> {
    platform::previous_bundle_id(app)
}
