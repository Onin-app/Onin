//! # 剪贴板 API 模块
//!
//! 提供剪贴板相关的 Tauri 命令和监控功能
//!
//! ## 模块结构
//! - `types`: 类型定义
//! - `timestamp`: 时间戳管理
//! - `monitor`: 剪贴板监控
//! - `commands`: Tauri 命令

pub mod commands;
mod monitor;
mod timestamp;
mod types;

// 重新导出公共接口
pub use commands::{
    get_clipboard_content, plugin_clipboard_clear, plugin_clipboard_get_metadata,
    plugin_clipboard_read_image, plugin_clipboard_read_text, plugin_clipboard_write_image,
    plugin_clipboard_write_text,
};
pub use monitor::start_clipboard_monitor;
pub use types::{
    ClipboardContent, ClipboardContentType, ClipboardError, ClipboardFile, ClipboardMetadata,
    WriteImageOptions, WriteTextOptions,
};
