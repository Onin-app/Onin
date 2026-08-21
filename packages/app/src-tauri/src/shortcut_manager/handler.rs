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

    let app_clone = app.clone();
    let shortcut_clone = shortcut.clone();
    let _ = app.run_on_main_thread(move || {
        let shortcut_str = shortcut_clone.to_string();
        let triggered_shortcut = normalize_shortcut_string(&shortcut_str);
        let state: State<ShortcutState> = app_clone.state();

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
            execute_shortcut_action(&app_clone, app_shortcut);
        }
    });
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
        // 显示/隐藏统一走 window_manager 的状态机入口：
        // - request_show：捕获前窗 → 递增显示代数 → 同步取消隐藏任务 → 激活序列
        // - request_hide：标记命令隐藏 → 恢复前窗 → hide
        if let Some(window) = app.get_webview_window("main") {
            match window.is_visible() {
                Ok(true) => crate::window_manager::request_hide(app),
                Ok(false) => crate::window_manager::request_show(app),
                Err(e) => eprintln!("Error checking window visibility: {}", e),
            }
        } else if let Some(window) = app.get_window("main") {
            match window.is_visible() {
                Ok(true) => crate::window_manager::request_hide(app),
                Ok(false) => {
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
        let was_visible = window.is_visible().unwrap_or(false);
        if was_visible {
            crate::focus_manager::focus_webview_window(&window);
        }
        // 窗口隐藏时不主动显示/聚焦，避免闪烁首页：
        // 前端 Svelte 完成路由跳转后会调用 show_main_window_cmd（内部走
        // request_show：捕获前窗 → 递增代数 → 取消隐藏任务 → 激活序列）来显示并聚焦。
        if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
            eprintln!("Error emitting command: {}", e);
        }
    } else if let Some(window) = app.get_window("main") {
        let was_visible = window.is_visible().unwrap_or(false);
        if was_visible {
            crate::focus_manager::focus_window(&window);
        }
        if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
            eprintln!("Error emitting command (fallback): {}", e);
        }
    }
}
