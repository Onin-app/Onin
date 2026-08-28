//! 剪贴板自动清空与监控辅助模块

use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use super::timestamp::{
    get_clipboard_timestamp, update_clipboard_timestamp, CLIPBOARD_TIMESTAMP, WINDOW_HIDE_TIMESTAMP,
};
use crate::app_config::AppConfigState;

/// 初始化剪贴板辅助服务（初始化时间戳并启动自动清空检查任务）
pub fn init_clipboard_service(app: &AppHandle) {
    // 初始化时间戳
    update_clipboard_timestamp();

    // 启动跨平台通用的定时清空检查异步任务
    start_auto_clear_task(app.clone());
}

/// 自动清空检查任务
fn start_auto_clear_task(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;

            // 检查窗口是否隐藏
            let window_hidden = if let Some(window) = app.get_webview_window("main") {
                !window.is_visible().unwrap_or(true)
            } else {
                false
            };

            // 只有在窗口隐藏时才执行自动清空逻辑
            if !window_hidden {
                if let Ok(mut hide_ts) = WINDOW_HIDE_TIMESTAMP.lock() {
                    *hide_ts = None;
                }
                continue;
            }

            // 记录窗口隐藏的时间戳
            let hide_timestamp = {
                if let Ok(mut hide_ts) = WINDOW_HIDE_TIMESTAMP.lock() {
                    if hide_ts.is_none() {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        *hide_ts = Some(now);
                        now
                    } else {
                        hide_ts.unwrap()
                    }
                } else {
                    continue;
                }
            };

            // 获取配置
            let auto_clear_time_limit = match app.state::<AppConfigState>().0.lock() {
                Ok(config) => config.auto_clear_time_limit,
                Err(_) => 0,
            };

            // 如果设置了自动清空时间限制
            if auto_clear_time_limit > 0 && get_clipboard_timestamp().is_some() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let elapsed_since_hide = now.saturating_sub(hide_timestamp);

                if elapsed_since_hide >= auto_clear_time_limit {
                    let _ = app.emit("clear_app_clipboard", ());

                    // 重置时间戳
                    if let Ok(mut ts) = CLIPBOARD_TIMESTAMP.lock() {
                        *ts = 0;
                    }
                    if let Ok(mut hide_ts) = WINDOW_HIDE_TIMESTAMP.lock() {
                        *hide_ts = None;
                    }
                }
            }
        }
    });
}
