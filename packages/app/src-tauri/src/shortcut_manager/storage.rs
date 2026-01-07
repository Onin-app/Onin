//! 快捷键存储模块
//!
//! 负责快捷键的持久化

use crate::shared_types::Shortcut as AppShortcut;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 获取快捷键存储文件路径
pub fn get_shortcuts_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("shortcuts.json")
}

/// 从磁盘加载快捷键
pub fn load_shortcuts_from_disk(app: &AppHandle) -> Vec<AppShortcut> {
    let path = get_shortcuts_file_path(app);
    if !path.exists() {
        return vec![];
    }

    match fs::read_to_string(&path) {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_else(|e| {
            eprintln!(
                "Failed to parse shortcuts.json: {}. Returning empty list.",
                e
            );
            vec![]
        }),
        Err(e) => {
            eprintln!(
                "Failed to read shortcuts.json: {}. Returning empty list.",
                e
            );
            vec![]
        }
    }
}

/// 保存快捷键到磁盘
pub fn save_shortcuts_to_disk(app: &AppHandle, shortcuts: &[AppShortcut]) {
    let path = get_shortcuts_file_path(app);
    let json = serde_json::to_string_pretty(shortcuts).unwrap();
    if let Err(e) = fs::write(path, json) {
        eprintln!("Failed to write to shortcuts.json: {}", e);
    }
}
