use tauri::{App, AppHandle, WebviewWindow, Window};

pub fn setup(_app: &mut App) {}

pub fn capture_previous_foreground(_app: &AppHandle) {}

pub fn restore_previous_foreground(_app: &AppHandle) {}

pub fn focus_webview_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn focus_window(window: &Window) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}
