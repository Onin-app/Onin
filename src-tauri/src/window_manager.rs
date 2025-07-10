use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tauri::{App, Emitter, Listener, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;
use crate::app_cache_manager;

// State to track if the window was hidden by a command, to prevent
// redundant hides on focus loss.
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
}

// State to prevent the window from closing on blur when certain actions are active.
// 使用计数器，以防未来有多个操作需要同时加锁。
pub struct WindowCloseLockState(pub AtomicU32);

#[tauri::command]
pub fn acquire_window_close_lock(state: State<WindowCloseLockState>) {
    state.0.fetch_add(1, Ordering::Relaxed);
}

#[tauri::command]
pub fn release_window_close_lock(state: State<WindowCloseLockState>) {
    state.0.fetch_sub(1, Ordering::Relaxed);
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
    let app_handle_for_drag = app.handle().clone();

    // 自动在文件拖放操作期间锁定窗口，解决问题 #3
    window.listen("tauri://file-drop-hover", move |_event| {
        let lock_state: State<WindowCloseLockState> = app_handle_for_drag.state();
        // 为简化，我们假定同一时间只有一个拖放操作。进入时锁住，结束时释放。
        lock_state.0.store(1, Ordering::Relaxed);
    });

    let app_handle_for_drop = app.handle().clone();
    window.listen("tauri://file-drop", move |_event| {
        let lock_state: State<WindowCloseLockState> = app_handle_for_drop.state();
        lock_state.0.store(0, Ordering::Relaxed); // 释放锁
    });

    let app_handle_for_cancel = app.handle().clone();
    window.listen("tauri://file-drop-cancelled", move |_event| {
        let lock_state: State<WindowCloseLockState> = app_handle_for_cancel.state();
        lock_state.0.store(0, Ordering::Relaxed); // 释放锁
    });

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
            let window_state: State<WindowState> = app_handle.state();
            let lock_state: State<WindowCloseLockState> = app_handle.state();

            // Always unregister "Esc" shortcut when the window loses focus.
            app_handle
                .global_shortcut()
                .unregister(close_window_shortcut)
                .unwrap_or_else(|err| {
                    eprintln!("[ERROR] Failed to unregister Esc shortcut: {}", err);
                });

            // 如果窗口被锁定（例如，文件对话框打开时），则不执行任何操作
            if lock_state.0.load(Ordering::Relaxed) > 0 {
                println!("Window focus lost, but close is locked. Skipping hide.");
                return;
            }

            // Atomically check and reset the flag.
            // If it was true, it means `close_main_window` was called.
            if window_state
                .hiding_initiated_by_command
                .swap(false, Ordering::Relaxed)
            {
                println!("Window focus lost due to command. Skipping redundant hide.");
            } else {
                // 解决问题 #1：拖动窗口导致关闭
                println!("Window lost focus naturally. Scheduling hide.");
                let window_to_hide = window_clone.clone();
                let app_handle_for_delay = app_handle.clone();
                // 启动一个异步任务，在短暂延迟后隐藏窗口
                tauri::async_runtime::spawn(async move {
                    sleep(Duration::from_millis(10)).await;
                    let lock_state: State<WindowCloseLockState> = app_handle_for_delay.state();
                    if !window_to_hide.is_focused().unwrap_or(false)
                        && lock_state.0.load(Ordering::Relaxed) == 0
                    {
                        window_to_hide.hide().ok();
                        window_to_hide
                            .emit("window_visibility", &false)
                            .unwrap_or_default();
                    }
                });
            }
        }
        _ => {}
    });

    Ok(())
}
