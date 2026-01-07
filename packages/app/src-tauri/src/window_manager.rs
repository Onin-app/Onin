use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tokio::time::sleep;

// ============================================================================
// 状态定义
// ============================================================================

/// 追踪窗口是否被命令隐藏，用于防止失焦时重复隐藏
pub struct WindowState {
    pub hiding_initiated_by_command: AtomicBool,
}

/// 防止窗口在某些操作（如对话框打开）期间关闭
/// 使用计数器以支持多个操作同时加锁
pub struct WindowCloseLockState(pub AtomicU32);

/// 持有隐藏任务的句柄，以便可以取消
pub struct HideTaskState {
    pub handle: tokio::sync::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取窗口关闭锁
#[tauri::command]
pub fn acquire_window_close_lock(state: State<WindowCloseLockState>) {
    state.0.fetch_add(1, Ordering::Relaxed);
}

/// 释放窗口关闭锁
#[tauri::command]
pub fn release_window_close_lock(state: State<WindowCloseLockState>) {
    if state.0.load(Ordering::Relaxed) > 0 {
        state.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 关闭窗口快捷键字符串
pub const CLOSE_WINDOW_SHORTCUT_STR: &str = "escape";

/// 隐藏主窗口命令
#[tauri::command]
pub fn close_main_window(app: tauri::AppHandle, state: State<WindowState>) {
    if let Some(window) = app.get_webview_window("main") {
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 取消任何挂起的隐藏任务
async fn cancel_hide_task(app: &AppHandle) {
    let state: State<HideTaskState> = app.state();
    let mut handle_guard = state.handle.lock().await;
    if let Some(handle) = handle_guard.take() {
        println!("Cancelling pending hide task.");
        handle.abort();
    }
}

/// 检查并释放窗口关闭锁
fn release_lock_if_held(app_handle: &AppHandle) {
    let lock_state: State<WindowCloseLockState> = app_handle.state();
    if lock_state.0.load(Ordering::Relaxed) > 0 {
        lock_state.0.fetch_sub(1, Ordering::Relaxed);
    }
}

// ============================================================================
// 文件拖放事件处理
// ============================================================================

/// 设置文件拖放事件监听器
/// 
/// 在文件拖放期间锁定窗口，防止意外隐藏
fn setup_file_drop_listeners(window: &tauri::WebviewWindow, app_handle: &AppHandle) {
    // 文件悬停：获取锁并取消隐藏任务
    let app_handle_hover = app_handle.clone();
    let app_handle_hover_cancel = app_handle.clone();
    window.listen("tauri://file-drop-hover", move |_event| {
        println!("File drag hover detected, acquiring window close lock and cancelling hide task.");
        let lock_state: State<WindowCloseLockState> = app_handle_hover.state();
        lock_state.0.fetch_add(1, Ordering::Relaxed);

        let app_handle_clone = app_handle_hover_cancel.clone();
        tauri::async_runtime::spawn(async move {
            cancel_hide_task(&app_handle_clone).await;
        });
    });

    // 文件放下：释放锁
    let app_handle_drop = app_handle.clone();
    window.listen("tauri://file-drop", move |_event| {
        println!("File dropped, releasing window close lock.");
        release_lock_if_held(&app_handle_drop);
    });

    // 拖放取消：释放锁
    let app_handle_cancel = app_handle.clone();
    window.listen("tauri://file-drop-cancelled", move |_event| {
        println!("File drag cancelled, releasing window close lock.");
        release_lock_if_held(&app_handle_cancel);
    });
}

// ============================================================================
// 智能隐藏任务
// ============================================================================

/// 创建智能隐藏任务
/// 
/// 等待鼠标释放或超时后隐藏窗口，具有以下特性：
/// - 监听全局鼠标释放事件
/// - 2秒超时机制作为后备
/// - 隐藏前最终检查窗口状态
fn spawn_smart_hide_task(
    window: tauri::WebviewWindow,
    app_handle: AppHandle,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut rx = crate::RDEV_EVENT_CHANNEL.0.subscribe();

        // 等待鼠标释放
        let hide_on_mouse_release = async {
            loop {
                if let Ok(event) = rx.recv().await {
                    if let rdev::EventType::ButtonRelease(rdev::Button::Left) = event.event_type {
                        break;
                    }
                }
            }
        };

        // 等待鼠标释放或超时
        tokio::select! {
            _ = hide_on_mouse_release => {
                sleep(Duration::from_millis(50)).await;
                println!("Global mouse release detected. Attempting to hide window.");
            }
            _ = sleep(Duration::from_millis(2000)) => {
                println!("Timeout reached after focus loss. Attempting to hide window.");
            }
        }

        // 最终检查并隐藏
        let lock_state: State<WindowCloseLockState> = app_handle.state();
        let is_focused = window.is_focused().unwrap_or(false);
        let is_locked = lock_state.0.load(Ordering::Relaxed) > 0;

        if !is_focused && !is_locked {
            if window.is_visible().unwrap_or(false) {
                println!("Hiding window now.");
                window.hide().ok();
                window.emit("window_visibility", &false).unwrap_or_default();
            } else {
                println!("Window already hidden, skipping hide event.");
            }
        } else {
            println!("Hide cancelled at the last moment (window re-focused or locked).");
        }

        // 清理任务句柄
        let hide_task_state: State<HideTaskState> = app_handle.state();
        let mut handle_guard = hide_task_state.handle.lock().await;
        *handle_guard = None;
    })
}

/// 存储隐藏任务句柄
fn store_hide_task_handle(app_handle: &AppHandle, handle: tauri::async_runtime::JoinHandle<()>) {
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let hide_task_state: State<HideTaskState> = app_handle_clone.state();
        let mut handle_guard = hide_task_state.handle.lock().await;
        *handle_guard = Some(handle);
    });
}

// ============================================================================
// 窗口焦点事件处理
// ============================================================================

/// 处理窗口获得焦点
fn handle_window_focused(app_handle: &AppHandle, shortcut: Shortcut) {
    // 取消隐藏任务
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        cancel_hide_task(&app_handle_clone).await;
    });

    // 注册 ESC 快捷键
    println!("Window focused, registering Esc shortcut.");
    app_handle
        .global_shortcut()
        .register(shortcut)
        .unwrap_or_else(|err| {
            eprintln!("[ERROR] Failed to register Esc shortcut: {}", err);
        });
}

/// 处理窗口失去焦点
fn handle_window_blur(
    app_handle: &AppHandle,
    window: &tauri::WebviewWindow,
    shortcut: Shortcut,
) {
    let window_state: State<WindowState> = app_handle.state();
    let lock_state: State<WindowCloseLockState> = app_handle.state();

    // 注销 ESC 快捷键
    app_handle
        .global_shortcut()
        .unregister(shortcut)
        .unwrap_or_else(|err| {
            eprintln!("[ERROR] Failed to unregister Esc shortcut: {}", err);
        });

    // 如果窗口被锁定，跳过隐藏
    if lock_state.0.load(Ordering::Relaxed) > 0 {
        println!("Window focus lost, but close is locked. Skipping hide.");
        return;
    }

    // 检查是否是命令触发的隐藏
    if window_state
        .hiding_initiated_by_command
        .swap(false, Ordering::Relaxed)
    {
        println!("Window focus lost due to command. Skipping redundant hide.");
        let app_handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            cancel_hide_task(&app_handle_clone).await;
        });
        return;
    }

    // 启动智能隐藏任务
    println!("Window lost focus. Scheduling smart hide task.");
    let handle = spawn_smart_hide_task(window.clone(), app_handle.clone());
    store_hide_task_handle(app_handle, handle);
}

// ============================================================================
// 主设置函数
// ============================================================================

/// 设置窗口事件监听器
pub fn setup_window_events(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let app_handle = app.handle().clone();

    // 设置文件拖放事件
    setup_file_drop_listeners(&window, &app_handle);

    // 解析快捷键
    let close_window_shortcut = Shortcut::from_str(CLOSE_WINDOW_SHORTCUT_STR)?;

    // 设置窗口焦点事件
    let window_for_blur = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            handle_window_focused(&app_handle, close_window_shortcut);
        }
        tauri::WindowEvent::Focused(false) => {
            handle_window_blur(&app_handle, &window_for_blur, close_window_shortcut);
        }
        _ => {}
    });

    Ok(())
}
