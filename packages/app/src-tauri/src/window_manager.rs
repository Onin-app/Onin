use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Listener, Manager, State};
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

/// 隐藏主窗口命令
#[tauri::command]
pub fn close_main_window(app: tauri::AppHandle, state: State<WindowState>) {
    // Restore focus to the previous foreground window before hiding,
    // consistent with the toggle shortcut (Alt+Space) behavior.
    // Without this, Esc-close leaves the foreground in an unpredictable
    // state, causing the next Alt+Space open to fail to acquire focus.
    crate::focus_manager::restore_previous_foreground(&app);

    // Try get_webview_window first
    if let Some(window) = app.get_webview_window("main") {
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    } else if let Some(window) = app.get_window("main") {
        // Fallback to get_window
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    } else {
        eprintln!("[window_manager] Main window not found for closing");
    }
}

#[allow(dead_code)]
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let app_handle_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            cancel_hide_task(&app_handle_clone).await;
        });
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        // 与 focus_webview_window 一致：通过 eval 确保 WebView2 键盘焦点
        let _ = window.eval("window.focus()");
        let _ = window.emit("window_visibility", &true);
    }
}

/// 暴露给前端的显示主窗口命令
#[tauri::command]
pub fn show_main_window_cmd(app: tauri::AppHandle) {
    show_main_window(&app);
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 取消任何挂起的隐藏任务
async fn cancel_hide_task(app: &AppHandle) {
    let state: State<HideTaskState> = app.state();
    let mut handle_guard = state.handle.lock().await;
    if let Some(handle) = handle_guard.take() {
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
        release_lock_if_held(&app_handle_drop);
    });

    // 拖放取消：释放锁
    let app_handle_cancel = app_handle.clone();
    window.listen("tauri://file-drop-cancelled", move |_event| {
        release_lock_if_held(&app_handle_cancel);
    });
}

// ============================================================================
// 智能隐藏任务
// ============================================================================

/// 创建隐藏任务
///
/// 短延迟后隐藏窗口，具有以下特性：
/// - 50ms 延迟防止闪烁
/// - 隐藏前最终检查窗口状态（焦点和锁定）
/// - 文件拖拽期间由 WindowCloseLockState 保护
fn spawn_smart_hide_task(
    window: tauri::WebviewWindow,
    app_handle: AppHandle,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        // 短延迟后尝试隐藏
        sleep(Duration::from_millis(50)).await;

        // 最终检查并隐藏
        let lock_state: State<WindowCloseLockState> = app_handle.state();
        let mut is_focused = window.is_focused().unwrap_or(false);

        if !is_focused {
            // 在 Windows 上，检查前台 HWND 是否是当前窗口的子窗口
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetParent};

                if let Ok(hwnd) = window.hwnd() {
                    let fg_hwnd = unsafe { GetForegroundWindow() };
                    if fg_hwnd.0 != 0 {
                        let mut current = fg_hwnd;
                        while current.0 != 0 {
                            if current.0 == (hwnd.0 as isize) {
                                is_focused = true;
                                break;
                            }
                            current = unsafe { GetParent(current) };
                        }
                    }
                }
            }
        }

        let is_locked = lock_state.0.load(Ordering::Relaxed) > 0;

        if !is_focused && !is_locked {
            if window.is_visible().unwrap_or(false) {
                window.hide().ok();
                window.emit("window_visibility", &false).unwrap_or_default();
            } else {
            }
        } else {
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
fn handle_window_focused(app_handle: &AppHandle) {
    // 取消隐藏任务
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        cancel_hide_task(&app_handle_clone).await;
    });
}

/// 处理窗口失去焦点
fn handle_window_blur(app_handle: &AppHandle, window: &tauri::WebviewWindow) {
    let window_state: State<WindowState> = app_handle.state();
    let lock_state: State<WindowCloseLockState> = app_handle.state();

    // 如果窗口被锁定，跳过隐藏
    if lock_state.0.load(Ordering::Relaxed) > 0 {
        return;
    }

    // 检查是否是命令触发的隐藏
    if window_state
        .hiding_initiated_by_command
        .swap(false, Ordering::Relaxed)
    {
        let app_handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            cancel_hide_task(&app_handle_clone).await;
        });
        return;
    }

    // 启动智能隐藏任务
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

    // 设置窗口焦点事件
    let window_for_blur = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(true) => {
            handle_window_focused(&app_handle);
        }
        tauri::WindowEvent::Focused(false) => {
            handle_window_blur(&app_handle, &window_for_blur);
        }
        _ => {}
    });

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn setup_activation_observer(app: &AppHandle) {
    macos_activation::setup(app);
}

#[cfg(target_os = "macos")]
mod macos_activation {
    use super::show_main_window;
    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, sel, ClassType};
    use objc2_app_kit::{
        NSApplicationDidBecomeActiveNotification, NSApplicationDidUnhideNotification,
    };
    use objc2_foundation::{NSNotification, NSNotificationCenter};
    use once_cell::sync::{Lazy, OnceCell};
    use std::sync::Mutex;
    use tauri::{AppHandle, Manager};

    define_class!(
        #[unsafe(super(objc2::runtime::NSObject))]
        struct ActivationObserver;

        impl ActivationObserver {
            #[unsafe(method(handleDidBecomeActive:))]
            fn handle_did_become_active(&self, _notification: &NSNotification) {
                if let Some(app) = APP_HANDLE.get() {
                    // 检查是否有任何非 "main" 的窗口是可见的
                    let has_other_visible_window = app.windows().iter().any(|(label, window)| {
                        label != "main" && window.is_visible().unwrap_or(false)
                    });

                    if !has_other_visible_window {
                        show_main_window(app);
                    }
                }
            }
        }
    );

    static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
    static OBSERVER: Lazy<Mutex<Option<Retained<ActivationObserver>>>> =
        Lazy::new(|| Mutex::new(None));

    pub fn setup(app: &AppHandle) {
        let _ = APP_HANDLE.set(app.clone());
        let observer: Retained<ActivationObserver> =
            unsafe { msg_send![ActivationObserver::class(), new] };
        let center = NSNotificationCenter::defaultCenter();
        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                sel!(handleDidBecomeActive:),
                Some(&NSApplicationDidBecomeActiveNotification),
                None,
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(handleDidBecomeActive:),
                Some(&NSApplicationDidUnhideNotification),
                None,
            );
        }
        if let Ok(mut guard) = OBSERVER.lock() {
            *guard = Some(observer);
        }
    }
}
