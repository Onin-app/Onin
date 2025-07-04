use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager, State};

// State to track if the window was hidden by a command, to prevent
// redundant hides on focus loss.
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
}

// Command to hide the main window.
#[tauri::command]
pub fn close_main_window(app: tauri::AppHandle, state: State<WindowState>) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🥳 这是ESC");
        // Informs the window event listener that this hide is intentional,
        // so it doesn't try to hide it again on focus loss.
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    }
}