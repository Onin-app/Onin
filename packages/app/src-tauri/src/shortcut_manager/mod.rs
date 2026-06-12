//! # 快捷键管理器模块
//!
//! 管理全局快捷键的注册、存储和处理
//!
//! ## 模块结构
//! - `state`: 状态类型定义
//! - `storage`: 快捷键存储
//! - `handler`: 全局快捷键处理
//! - `commands`: Tauri 命令
//! - `utils`: 工具函数

pub mod commands;
mod handler;
mod state;
mod storage;
mod utils;

// 重新导出公共接口
pub use handler::handle_global_shortcut;
pub use state::ShortcutState;

use crate::shared_types::Shortcut as AppShortcut;
use std::str::FromStr;
use tauri::{App, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// 设置快捷键
///
/// 加载保存的快捷键，添加默认快捷键，并注册到系统
pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // macOS 特定检查
    #[cfg(target_os = "macos")]
    {
        if !utils::check_accessibility_permissions() {
            eprintln!("Warning: Accessibility permissions not granted. Global shortcuts may not work properly.");
            eprintln!("Please grant accessibility permissions in System Preferences > Security & Privacy > Privacy > Accessibility");
        }
    }

    let mut shortcuts = storage::load_shortcuts_from_disk(&app.handle());

    // 设置默认快捷键（如果用户从未设置过）
    let has_toggle_window = shortcuts.iter().any(|s| s.command_name == "toggle_window");
    let has_detach_window = shortcuts.iter().any(|s| s.command_name == "detach_window");

    // 默认显示/隐藏窗口快捷键
    if !has_toggle_window {
        shortcuts.push(AppShortcut {
            shortcut: "alt+Space".to_string(),
            command_name: "toggle_window".to_string(),
            command_title: Some("显示/隐藏窗口".to_string()),
        });
    }

    // 默认分离窗口快捷键
    if !has_detach_window {
        shortcuts.push(AppShortcut {
            shortcut: "cmd+shift+D".to_string(),
            command_name: "detach_window".to_string(),
            command_title: Some("分离窗口".to_string()),
        });
    }

    // 保存更新后的快捷键
    if !has_toggle_window || !has_detach_window {
        storage::save_shortcuts_to_disk(&app.handle(), &shortcuts);
    }

    // 更新状态
    let state: State<ShortcutState> = app.state();
    *state.shortcuts.lock().unwrap() = shortcuts.clone();

    // 注册所有快捷键
    for shortcut in shortcuts {
        match Shortcut::from_str(&shortcut.shortcut) {
            Ok(tauri_shortcut) => {
                if let Err(e) = app.global_shortcut().register(tauri_shortcut) {
                    eprintln!("Failed to register shortcut {}: {}", shortcut.shortcut, e);
                }
            }
            Err(e) => {
                eprintln!("Invalid shortcut format {}: {}", shortcut.shortcut, e);
            }
        }
    }

    Ok(())
}
