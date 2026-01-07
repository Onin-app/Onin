//! Tauri 命令模块

use crate::command_manager;
use crate::shared_types::Shortcut as AppShortcut;
use std::str::FromStr;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use super::state::ShortcutState;
use super::storage;

/// 获取所有快捷键
#[tauri::command]
pub async fn get_shortcuts(
    app: AppHandle,
    state: State<'_, ShortcutState>,
) -> Result<Vec<AppShortcut>, String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to lock shortcut state".to_string())?
        .clone();

    let commands = command_manager::load_commands(&app).await;

    for shortcut in shortcuts.iter_mut() {
        if let Some(command) = commands
            .iter()
            .find(|cmd| cmd.name == shortcut.command_name)
        {
            shortcut.command_title = Some(command.title.clone());
        }
    }

    Ok(shortcuts)
}

/// 添加快捷键
#[tauri::command]
pub async fn add_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut: AppShortcut,
) -> Result<(), String> {
    add_shortcut_sync(app, state, shortcut)
}

/// 同步添加快捷键
fn add_shortcut_sync(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut: AppShortcut,
) -> Result<(), String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    // 移除相同键位的旧快捷键
    if let Some(index) = shortcuts
        .iter()
        .position(|s| s.shortcut == shortcut.shortcut)
    {
        let old_shortcut = &shortcuts[index];
        if old_shortcut.command_name != shortcut.command_name {
            let tauri_shortcut =
                Shortcut::from_str(&old_shortcut.shortcut).map_err(|e| e.to_string())?;
            if app.global_shortcut().is_registered(tauri_shortcut.clone()) {
                app.global_shortcut()
                    .unregister(tauri_shortcut)
                    .map_err(|e| e.to_string())?;
            }
        }
        shortcuts.remove(index);
    }

    let tauri_shortcut = Shortcut::from_str(&shortcut.shortcut).map_err(|e| e.to_string())?;
    app.global_shortcut()
        .register(tauri_shortcut)
        .map_err(|e| e.to_string())?;

    shortcuts.push(shortcut);
    storage::save_shortcuts_to_disk(&app, &shortcuts);

    Ok(())
}

/// 移除快捷键
#[tauri::command]
pub async fn remove_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
) -> Result<(), String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    if let Some(index) = shortcuts.iter().position(|s| s.shortcut == shortcut_str) {
        let tauri_shortcut = Shortcut::from_str(&shortcut_str).map_err(|e| e.to_string())?;
        if app.global_shortcut().is_registered(tauri_shortcut.clone()) {
            app.global_shortcut()
                .unregister(tauri_shortcut)
                .map_err(|e| e.to_string())?;
        }
        shortcuts.remove(index);
        storage::save_shortcuts_to_disk(&app, &shortcuts);
    }

    Ok(())
}

/// 设置切换窗口快捷键
#[tauri::command]
pub async fn set_toggle_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
) -> Result<(), String> {
    set_named_shortcut(
        app,
        state,
        shortcut_str,
        "toggle_window",
        "显示/隐藏窗口",
    )
}

/// 获取切换窗口快捷键
#[tauri::command]
pub fn get_toggle_shortcut(state: State<ShortcutState>) -> Result<String, String> {
    get_named_shortcut(state, "toggle_window")
}

/// 设置分离窗口快捷键
#[tauri::command]
pub async fn set_detach_window_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
) -> Result<(), String> {
    set_named_shortcut(
        app,
        state,
        shortcut_str,
        "detach_window",
        "分离窗口",
    )
}

/// 获取分离窗口快捷键
#[tauri::command]
pub fn get_detach_window_shortcut(state: State<ShortcutState>) -> Result<String, String> {
    get_named_shortcut(state, "detach_window")
}

// ========== 辅助函数 ==========

/// 通用设置命名快捷键
fn set_named_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
    command_name: &str,
    command_title: &str,
) -> Result<(), String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    // 移除旧快捷键
    if let Some(index) = shortcuts.iter().position(|s| s.command_name == command_name) {
        let old_shortcut_str = &shortcuts[index].shortcut;
        let old_tauri_shortcut = Shortcut::from_str(old_shortcut_str).map_err(|e| e.to_string())?;
        if app.global_shortcut().is_registered(old_tauri_shortcut.clone()) {
            app.global_shortcut()
                .unregister(old_tauri_shortcut)
                .map_err(|e| e.to_string())?;
        }
        shortcuts.remove(index);
    }

    // 添加新快捷键
    if !shortcut_str.is_empty() {
        let new_shortcut = AppShortcut {
            shortcut: shortcut_str.clone(),
            command_name: command_name.to_string(),
            command_title: Some(command_title.to_string()),
        };
        let new_tauri_shortcut = Shortcut::from_str(&shortcut_str).map_err(|e| e.to_string())?;
        app.global_shortcut()
            .register(new_tauri_shortcut)
            .map_err(|e| e.to_string())?;
        shortcuts.push(new_shortcut);
    }

    storage::save_shortcuts_to_disk(&app, &shortcuts);

    Ok(())
}

/// 通用获取命名快捷键
fn get_named_shortcut(state: State<ShortcutState>, command_name: &str) -> Result<String, String> {
    let shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to lock shortcut state".to_string())?;

    if let Some(shortcut) = shortcuts.iter().find(|s| s.command_name == command_name) {
        Ok(shortcut.shortcut.clone())
    } else {
        Ok("".to_string())
    }
}
