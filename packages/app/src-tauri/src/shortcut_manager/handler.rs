//! 全局快捷键处理模块

use super::state::ShortcutState;
use super::utils::normalize_shortcut_string;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState as GlobalShortcutPluginState};

const SHORTCUT_DEBOUNCE_MS: u128 = 400;

pub fn handle_global_shortcut(
    app: &AppHandle,
    shortcut: &Shortcut,
    event: GlobalShortcutPluginState,
) {
    if event != GlobalShortcutPluginState::Pressed {
        return;
    }

    let shortcut_str = shortcut.to_string();
    let triggered_shortcut = normalize_shortcut_string(&shortcut_str);
    let state: State<ShortcutState> = app.state();

    let shortcuts = match state.shortcuts.lock() {
        Ok(shortcuts) => shortcuts,
        Err(e) => {
            eprintln!("Failed to lock shortcuts state: {}", e);
            return;
        }
    };

    let matching_shortcut = shortcuts.iter().find(|s| {
        let stored_shortcut = normalize_shortcut_string(&s.shortcut);
        stored_shortcut == triggered_shortcut
    });

    if let Some(app_shortcut) = matching_shortcut {
        if app_shortcut.command_name != "toggle_window"
            && should_debounce_shortcut(&state, &triggered_shortcut)
        {
            return;
        }
        execute_shortcut_action(app, app_shortcut);
    } else {
        if should_debounce_shortcut(&state, &triggered_shortcut) {
            return;
        }
        handle_special_keys(app, &triggered_shortcut);
    }
}

fn should_debounce_shortcut(state: &State<ShortcutState>, triggered_shortcut: &str) -> bool {
    if let Ok(mut last_executed) = state.last_executed.lock() {
        if let Some(last_time) = last_executed.get(triggered_shortcut) {
            let elapsed = last_time.elapsed().as_millis();
            if elapsed < SHORTCUT_DEBOUNCE_MS {
                return true;
            }
        }
        last_executed.insert(triggered_shortcut.to_string(), std::time::Instant::now());
    }

    false
}

fn execute_shortcut_action(app: &AppHandle, app_shortcut: &crate::shared_types::Shortcut) {
    if app_shortcut.command_name == "toggle_window" {
        if let Some(window) = app.get_webview_window("main") {
            match window.is_visible() {
                Ok(true) => {
                    crate::focus_manager::restore_previous_foreground(app);
                    let _ = window.hide();
                    let _ = window.emit("window_visibility", &false);
                }
                Ok(false) => {
                    crate::focus_manager::capture_previous_foreground(app);
                    crate::focus_manager::focus_webview_window(&window);
                    let _ = window.emit("window_visibility", &true);
                }
                Err(e) => eprintln!("Error checking window visibility: {}", e),
            }
        } else if let Some(window) = app.get_window("main") {
            match window.is_visible() {
                Ok(true) => {
                    crate::focus_manager::restore_previous_foreground(app);
                    let _ = window.hide();
                    let _ = window.emit("window_visibility", &false);
                }
                Ok(false) => {
                    crate::focus_manager::capture_previous_foreground(app);
                    crate::focus_manager::focus_window(&window);
                    let _ = window.emit("window_visibility", &true);
                }
                Err(e) => eprintln!("Error checking window visibility (fallback): {}", e),
            }
        } else {
            eprintln!("Main window not found for toggle_window");
        }
    } else if app_shortcut.command_name == "detach_window" {
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.emit("detach_window_shortcut", ()) {
                eprintln!("Error emitting detach window command: {}", e);
            }
        } else if let Some(window) = app.get_window("main") {
            if let Err(e) = window.emit("detach_window_shortcut", ()) {
                eprintln!("Error emitting detach window command (fallback): {}", e);
            }
        }
    } else if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
            eprintln!("Error emitting command: {}", e);
        }
    } else if let Some(window) = app.get_window("main") {
        if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
            eprintln!("Error emitting command (fallback): {}", e);
        }
    }
}

pub fn handle_escape_action(app: &AppHandle) {
    if let Some(active_window_state) = app.try_state::<crate::plugin::ActivePluginWindow>() {
        if let Ok(active) = active_window_state.0.lock() {
            if let Some(window_label) = active.as_ref() {
                if let Some(window) = app.get_webview_window(window_label) {
                    if let Err(e) = window.minimize() {
                        eprintln!("Failed to minimize plugin window: {}", e);
                    }
                    return;
                }
            }
        }
    }

    if let Some(window) = app.get_window("translator-host") {
        match window.is_visible() {
            Ok(true) => {
                if let Err(e) = window.close() {
                    eprintln!("Failed to close translator window: {}", e);
                }
                return;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("Failed to check translator window visibility: {}", e);
            }
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.emit("escape_pressed", ()) {
            eprintln!("Error emitting escape_pressed event: {}", e);
        }
    } else if let Some(window) = app.get_window("main") {
        let _ = window.emit("escape_pressed", ());
    } else {
        eprintln!("Main window not found when handling ESC");
    }
}

fn handle_special_keys(app: &AppHandle, triggered_shortcut: &str) {
    if triggered_shortcut.to_uppercase() == "ESCAPE" {
        handle_escape_action(app);
    }
}
