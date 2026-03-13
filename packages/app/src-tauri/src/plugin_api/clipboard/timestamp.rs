//! 剪贴板时间戳管理
//!
//! 维护两个独立的时间戳：
//! 1. CLIPBOARD_TIMESTAMP - 应用层，用于自动粘贴/清空，会被重置
//! 2. PLUGIN_CLIPBOARD_TIMESTAMP - 插件层，不受应用清理影响

use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

// 应用层时间戳
pub(crate) static CLIPBOARD_TIMESTAMP: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// 插件层时间戳
pub(crate) static PLUGIN_CLIPBOARD_TIMESTAMP: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// 剪贴板监控状态
#[allow(dead_code)]
pub(crate) static CLIPBOARD_MONITOR_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// 全局 AppHandle
#[allow(dead_code)]
pub(crate) static APP_HANDLE: Lazy<Mutex<Option<Arc<AppHandle>>>> = Lazy::new(|| Mutex::new(None));

// 窗口隐藏时间戳
#[allow(dead_code)]
pub(crate) static WINDOW_HIDE_TIMESTAMP: Lazy<Mutex<Option<u64>>> = Lazy::new(|| Mutex::new(None));

/// 更新剪贴板时间戳
///
/// 当系统剪贴板内容变化时调用，同时更新应用层和插件层时间戳
#[allow(dead_code)]
pub fn update_clipboard_timestamp() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 更新应用层时间戳
    let mut ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
    *ts = timestamp;
    drop(ts);

    // 更新插件层时间戳
    let mut plugin_ts = PLUGIN_CLIPBOARD_TIMESTAMP.lock().unwrap();
    *plugin_ts = timestamp;
}

/// 获取应用层剪贴板时间戳
///
/// 用于应用的自动粘贴和清空功能，会被 auto_clear_time_limit 重置为 0
pub fn get_clipboard_timestamp() -> Option<u64> {
    let ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
    if *ts == 0 {
        None
    } else {
        Some(*ts)
    }
}

/// 获取插件层剪贴板时间戳
///
/// 专门为插件 API 提供，不受应用清理机制影响
pub fn get_plugin_clipboard_timestamp() -> Option<u64> {
    let ts = PLUGIN_CLIPBOARD_TIMESTAMP.lock().unwrap();
    if *ts == 0 {
        None
    } else {
        Some(*ts)
    }
}

