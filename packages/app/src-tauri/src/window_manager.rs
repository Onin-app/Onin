use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;

// State to track if the window was hidden by a command, to prevent
// redundant hides on focus loss.
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
}

// State to prevent the window from closing on blur when certain actions are active.
// 使用计数器，以防未来有多个操作需要同时加锁。
pub struct WindowCloseLockState(pub AtomicU32);

// State to hold the handle of the hide-on-blur task, so it can be cancelled.
pub struct HideTaskState {
    pub handle: tokio::sync::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

#[tauri::command]
pub fn acquire_window_close_lock(state: State<WindowCloseLockState>) {
    state.0.fetch_add(1, Ordering::Relaxed);
}

#[tauri::command]
pub fn release_window_close_lock(state: State<WindowCloseLockState>) {
    // fetch_sub returns the previous value, ensure we don't underflow.
    if state.0.load(Ordering::Relaxed) > 0 {
        state.0.fetch_sub(1, Ordering::Relaxed);
    }
}

// The shortcut for closing the main window, used across modules.
pub const CLOSE_WINDOW_SHORTCUT_STR: &str = "escape";

// Command to hide the main window.
#[tauri::command]
pub fn close_main_window(app: tauri::AppHandle, state: State<WindowState>) {
    if let Some(window) = app.get_webview_window("main") {
        // Informs the window event listener that this hide is intentional,
        // so it doesn't try to hide it again on focus loss.
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    }
}

// Helper to cancel any pending hide task.
async fn cancel_hide_task(app: &AppHandle) {
    let state: State<HideTaskState> = app.state();
    // Lock the mutex and then take the handle. This explicit scoping avoids potential lifetime issues
    // with the MutexGuard temporary that the compiler was flagging.
    let mut handle_guard = state.handle.lock().await;
    if let Some(handle) = handle_guard.take() {
        println!("Cancelling pending hide task.");
        handle.abort();
    }
}

pub fn setup_window_events(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let app_handle = app.handle().clone();
    let window_clone = window.clone();
    // TODO：下面这三个事件监听不到，后续考虑删除，目前在前端 webview 监听了
    let app_handle_for_drag = app.handle().clone();
    let app_handle_for_drop = app.handle().clone();
    let app_handle_for_cancel = app.handle().clone();

    // 自动在文件拖放操作期间锁定窗口，解决问题 #3
    let app_handle_for_drag_clone = app_handle_for_drag.clone();
    window.listen("tauri://file-drop-hover", move |_event| {
        println!("File drag hover detected, acquiring window close lock and cancelling hide task.");
        let lock_state: State<WindowCloseLockState> = app_handle_for_drag.state();
        // 使用 fetch_add 以支持多个锁，使其更健壮
        lock_state.0.fetch_add(1, Ordering::Relaxed);

        let app_handle_clone = app_handle_for_drag_clone.clone();
        tauri::async_runtime::spawn(async move {
            cancel_hide_task(&app_handle_clone).await;
        });
    });

    window.listen("tauri://file-drop", move |_event| {
        println!("File dropped, releasing window close lock.");
        let lock_state: State<WindowCloseLockState> = app_handle_for_drop.state();
        if lock_state.0.load(Ordering::Relaxed) > 0 {
            lock_state.0.fetch_sub(1, Ordering::Relaxed); // 释放锁
        }
    });

    window.listen("tauri://file-drop-cancelled", move |_event| {
        println!("File drag cancelled, releasing window close lock.");
        let lock_state: State<WindowCloseLockState> = app_handle_for_cancel.state();
        if lock_state.0.load(Ordering::Relaxed) > 0 {
            lock_state.0.fetch_sub(1, Ordering::Relaxed); // 释放锁
        }
    });

    let close_window_shortcut = Shortcut::from_str(CLOSE_WINDOW_SHORTCUT_STR)?;

    // Listen to window events to manage the Esc shortcut and other behaviors.
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            // Cancel any hide task that might be running.
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                cancel_hide_task(&app_handle_clone).await;
            });

            // Register "Esc" shortcut when the window gains focus.
            println!("Window focused, registering Esc shortcut.");
            app_handle
                .global_shortcut()
                .register(close_window_shortcut)
                .unwrap_or_else(|err| {
                    eprintln!("[ERROR] Failed to register Esc shortcut: {}", err);
                });

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
                // Also cancel any lingering hide task, just in case.
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    cancel_hide_task(&app_handle_clone).await;
                });
            } else {
                // This is the new robust logic for hiding on blur.
                println!("Window lost focus. Scheduling smart hide task.");
                let window_to_hide = window_clone.clone();
                let app_handle_for_task = app_handle.clone();

                let new_handle = tauri::async_runtime::spawn(async move {
                    let mut rx = crate::RDEV_EVENT_CHANNEL.0.subscribe();

                    let hide_on_mouse_release = async {
                        loop {
                            if let Ok(event) = rx.recv().await {
                                if let rdev::EventType::ButtonRelease(rdev::Button::Left) = event.event_type {
                                    break;
                                }
                            }
                        }
                    };

                    tokio::select! {
                        _ = hide_on_mouse_release => {
                            // Add a tiny delay to allow a potential focus event to be processed first
                            // if the user clicks back on the window very quickly.
                            sleep(Duration::from_millis(50)).await;
                            println!("Global mouse release detected. Attempting to hide window.");
                        }
                        _ = sleep(Duration::from_millis(2000)) => {
                            // A long timeout for non-mouse events like Alt-Tab,
                            // or if the rdev listener fails for some reason.
                            println!("Timeout reached after focus loss. Attempting to hide window.");
                        }
                    }

                    // Final check before hiding
                    let lock_state: State<WindowCloseLockState> = app_handle_for_task.state();
                    if !window_to_hide.is_focused().unwrap_or(false) && lock_state.0.load(Ordering::Relaxed) == 0 {
                        // 只有在窗口可见时才隐藏并发送事件
                        if window_to_hide.is_visible().unwrap_or(false) {
                            println!("Hiding window now.");
                            window_to_hide.hide().ok();
                            window_to_hide.emit("window_visibility", &false).unwrap_or_default();
                        } else {
                            println!("Window already hidden, skipping hide event.");
                        }
                    } else {
                        println!("Hide cancelled at the last moment (window re-focused or locked).");
                    }

                    // Clear the handle from the state after completion
                    // Bind the state to a variable to extend its lifetime, preventing the temporary value from being dropped while borrowed.
                    let hide_task_state: State<HideTaskState> = app_handle_for_task.state();
                    let mut handle_guard = hide_task_state.handle.lock().await;
                    *handle_guard = None;
                });

                // Store the new handle
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    // Same pattern here: bind the state to a variable to extend its lifetime.
                    let hide_task_state: State<HideTaskState> = app_handle_clone.state();
                    let mut handle_guard = hide_task_state.handle.lock().await;
                    *handle_guard = Some(new_handle);
                });
            }
        }
        _ => {}
    });

    Ok(())
}
