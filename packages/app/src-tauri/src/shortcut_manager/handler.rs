//! 全局快捷键处理模块

use super::state::ShortcutState;
use super::utils::normalize_shortcut_string;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState as GlobalShortcutPluginState};

const SHORTCUT_DEBOUNCE_MS: u128 = 400;

/// 处理全局快捷键事件
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

    println!(
        "Handling shortcut: {} (normalized: {})",
        shortcut_str, triggered_shortcut
    );

    // 获取状态
    let shortcuts = match state.shortcuts.lock() {
        Ok(shortcuts) => shortcuts,
        Err(e) => {
            eprintln!("Failed to lock shortcuts state: {}", e);
            return;
        }
    };

    // 查找匹配的快捷键
    let matching_shortcut = shortcuts.iter().find(|s| {
        let stored_shortcut = normalize_shortcut_string(&s.shortcut);
        println!(
            "Comparing with stored shortcut: {} (normalized: {})",
            s.shortcut, stored_shortcut
        );
        stored_shortcut == triggered_shortcut
    });

    if let Some(app_shortcut) = matching_shortcut {
        if app_shortcut.command_name != "toggle_window"
            && should_debounce_shortcut(&state, &triggered_shortcut)
        {
            return;
        }

        println!(
            "Found matching shortcut: {} -> {}",
            app_shortcut.shortcut, app_shortcut.command_name
        );
        execute_shortcut_action(app, app_shortcut);
    } else {
        handle_special_keys(app, &triggered_shortcut);
    }
}

fn should_debounce_shortcut(state: &State<ShortcutState>, triggered_shortcut: &str) -> bool {
    if let Ok(mut last_executed) = state.last_executed.lock() {
        if let Some(last_time) = last_executed.get(triggered_shortcut) {
            let elapsed = last_time.elapsed().as_millis();
            if elapsed < SHORTCUT_DEBOUNCE_MS {
                println!(
                    "Debouncing shortcut: {} (elapsed: {}ms)",
                    triggered_shortcut, elapsed
                );
                return true;
            }
        }
        last_executed.insert(triggered_shortcut.to_string(), std::time::Instant::now());
    }

    false
}

/// 执行快捷键动作
/// 执行快捷键动作
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
        println!("Executing detach window command");
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.emit("detach_window_shortcut", ()) {
                eprintln!("Error emitting detach window command: {}", e);
            }
        } else if let Some(window) = app.get_window("main") {
            if let Err(e) = window.emit("detach_window_shortcut", ()) {
                eprintln!("Error emitting detach window command (fallback): {}", e);
            }
        }
    } else {
        println!("Executing command: {}", app_shortcut.command_name);
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
                eprintln!("Error emitting command: {}", e);
            }
        } else if let Some(window) = app.get_window("main") {
            if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
                eprintln!("Error emitting command (fallback): {}", e);
            }
        }
    }
}

/// 处理特殊按键（如 ESC）
fn handle_special_keys(app: &AppHandle, triggered_shortcut: &str) {
    if triggered_shortcut.to_uppercase() == "ESCAPE" {
        println!("ESC key detected, checking for active plugin window");

        // 检查是否有活跃的插件窗口
        if let Some(active_window_state) = app.try_state::<crate::plugin::ActivePluginWindow>() {
            if let Ok(active) = active_window_state.0.lock() {
                if let Some(window_label) = active.as_ref() {
                    println!(
                        "Active plugin window found: {}, minimizing it",
                        window_label
                    );
                    if let Some(window) = app.get_webview_window(window_label) {
                        if let Err(e) = window.minimize() {
                            eprintln!("Failed to minimize plugin window: {}", e);
                        }
                        return;
                    }
                }
            }
        }

        // 如果没有活跃的插件窗口，则隐藏主窗口
        // MODIFY: 移除后端自动隐藏逻辑，交由前端控制
        // println!("No active plugin window, hiding main window");
        // if let Some(window) = app.get_webview_window("main") {
        //     let state: State<crate::window_manager::WindowState> = app.state();
        //     state
        //         .hiding_initiated_by_command
        //         .store(true, std::sync::atomic::Ordering::Relaxed);
        //     let _ = window.hide();
        //     let _ = window.emit("window_visibility", &false);
        // }
        println!("ESC detected in backend. Delegating to frontend.");
        if let Some(window) = app.get_webview_window("main") {
            println!("Found main window, emitting escape_pressed");
            if let Err(e) = window.emit("escape_pressed", ()) {
                eprintln!("Error emitting escape_pressed event: {}", e);
            }
        } else {
            eprintln!("Main window not found when handling ESC. Available windows:");
            for (label, _) in app.webview_windows() {
                eprintln!(" - {}", label);
            }
            // Try get_window as fallback (though in v2 it might be same)
            if let Some(w) = app.get_window("main") {
                eprintln!("Found 'main' via get_window! Emitting...");
                let _ = w.emit("escape_pressed", ());
            }
        }
    } else {
        println!("No matching shortcut found for: {}", triggered_shortcut);
    }
}
