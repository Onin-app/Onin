//! 剪贴板监控模块

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use super::timestamp::{
    update_clipboard_timestamp, APP_HANDLE, CLIPBOARD_MONITOR_STARTED, CLIPBOARD_TIMESTAMP,
    WINDOW_HIDE_TIMESTAMP,
};

/// 启动剪贴板监控
#[allow(dead_code)]
pub fn start_clipboard_monitor(app: AppHandle) {
    let mut started = CLIPBOARD_MONITOR_STARTED.lock().unwrap();
    if *started {
        return;
    }
    *started = true;
    drop(started);

    // 保存 AppHandle
    {
        let mut handle = APP_HANDLE.lock().unwrap();
        *handle = Some(Arc::new(app.clone()));
    }

    // 初始化时间戳
    update_clipboard_timestamp();

    // 启动剪贴板变化监控线程
    std::thread::spawn(move || {
        use clipboard_rs::{
            ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
        };

        struct ClipboardManager {
            #[allow(dead_code)]
            ctx: ClipboardContext,
        }

        impl ClipboardManager {
            #[allow(dead_code)]
            pub fn new() -> Self {
                let ctx = ClipboardContext::new().unwrap();
                ClipboardManager { ctx }
            }
        }

        impl ClipboardHandler for ClipboardManager {
            fn on_clipboard_change(&mut self) {
                update_clipboard_timestamp();
            }
        }

        let manager = ClipboardManager::new();
        let mut watcher = ClipboardWatcherContext::new().unwrap();
        watcher.add_handler(manager);
        watcher.start_watch();
    });

    // 启动定时清空检查线程
    std::thread::spawn(move || {
        start_auto_clear_thread();
    });
}

/// 自动清空检查线程
#[allow(dead_code)]
fn start_auto_clear_thread() {
    use super::timestamp::get_clipboard_timestamp;
    use crate::app_config::AppConfigState;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        // 获取 AppHandle
        let app_handle = {
            let handle = APP_HANDLE.lock().unwrap();
            handle.as_ref().map(|h| Arc::clone(h))
        };

        if let Some(app) = app_handle {
            // 检查窗口是否隐藏
            let window_hidden = if let Some(window) = app.get_webview_window("main") {
                !window.is_visible().unwrap_or(true)
            } else {
                false
            };

            // 只有在窗口隐藏时才执行自动清空逻辑
            if !window_hidden {
                let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                *hide_ts = None;
                continue;
            }

            // 记录窗口隐藏的时间戳
            let hide_timestamp = {
                let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                if hide_ts.is_none() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    *hide_ts = Some(now);
                    now
                } else {
                    hide_ts.unwrap()
                }
            };

            // 获取配置
            let config_state = app.state::<AppConfigState>();
            let config = config_state.0.lock().unwrap();
            let auto_clear_time_limit = config.auto_clear_time_limit;
            drop(config);

            // 如果设置了自动清空时间限制
            if auto_clear_time_limit > 0 && get_clipboard_timestamp().is_some() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let elapsed_since_hide = now - hide_timestamp;

                if elapsed_since_hide >= auto_clear_time_limit {

                    if let Err(_e) = app.emit("clear_app_clipboard", ()) {
                    } else {
                    }

                    // 重置时间戳
                    let mut ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
                    *ts = 0;
                    let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                    *hide_ts = None;
                }
            }
        }
    }
}


