use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{App, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::app_cache_manager;

// State to track if the window was hidden by a command, to prevent
// redundant hides on focus loss.
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
}

// The shortcut for closing the main window, used across modules.
pub const CLOSE_WINDOW_SHORTCUT_STR: &str = "escape";

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

pub fn setup_window_events(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let app_handle = app.handle().clone();
    let window_clone = window.clone();

    let close_window_shortcut = Shortcut::from_str(CLOSE_WINDOW_SHORTCUT_STR)?;

    // Listen to window events to manage the Esc shortcut and other behaviors.
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            // Register "Esc" shortcut when the window gains focus.
            println!("Window focused, registering Esc shortcut.");
            app_handle
                .global_shortcut()
                .register(close_window_shortcut)
                .unwrap_or_else(|err| {
                    eprintln!("[ERROR] Failed to register Esc shortcut: {}", err);
                });

            // Trigger a silent app list refresh.
            println!("Window focused, triggering silent app list refresh.");
            let handle = app_handle.clone();
            app_cache_manager::trigger_app_refresh(handle);
        }
        tauri::WindowEvent::Focused(false) => {
            let state: State<WindowState> = app_handle.state();

            // Always unregister "Esc" shortcut when the window loses focus.
            app_handle
                .global_shortcut()
                .unregister(close_window_shortcut)
                .unwrap_or_else(|err| {
                    eprintln!("[ERROR] Failed to unregister Esc shortcut: {}", err);
                });

            // Atomically check and reset the flag.
            // If it was true, it means `close_main_window` was called.
            if state
                .hiding_initiated_by_command
                .swap(false, Ordering::Relaxed)
            {
                println!("Window focus lost due to command. Skipping redundant hide.");
            } else {
                println!("Window lost focus naturally. Hiding window.");
                window_clone.hide().ok();
                window_clone.emit("window_visibility", &false).unwrap();
            }
        }
        _ => {}
    });

    Ok(())
}
