use crate::{command_manager, shared_types::Shortcut as AppShortcut};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{App, AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as GlobalShortcutPluginState,
};

// State to hold the currently configured shortcuts
pub struct ShortcutState {
    pub shortcuts: Mutex<Vec<AppShortcut>>,
}

fn get_shortcuts_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("shortcuts.json")
}

fn load_shortcuts_from_disk(app: &AppHandle) -> Vec<AppShortcut> {
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

fn save_shortcuts_to_disk(app: &AppHandle, shortcuts: &[AppShortcut]) {
    let path = get_shortcuts_file_path(app);
    let json = serde_json::to_string_pretty(shortcuts).unwrap();
    if let Err(e) = fs::write(path, json) {
        eprintln!("Failed to write to shortcuts.json: {}", e);
    }
}

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

#[tauri::command]
pub async fn add_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut: AppShortcut,
) -> Result<(), String> {
    add_shortcut_sync(app, state, shortcut)
}

fn add_shortcut_sync(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut: AppShortcut,
) -> Result<(), String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    // Remove any existing shortcut with the same key combination
    if let Some(index) = shortcuts
        .iter()
        .position(|s| s.shortcut == shortcut.shortcut)
    {
        let old_shortcut = &shortcuts[index];
        if old_shortcut.command_name != shortcut.command_name {
            // If the command is different, we need to unregister the old shortcut
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
    save_shortcuts_to_disk(&app, &shortcuts);

    Ok(())
}

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
        save_shortcuts_to_disk(&app, &shortcuts);
    }

    Ok(())
}

pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // macOS 特定检查
    #[cfg(target_os = "macos")]
    {
        // 检查辅助功能权限
        if !check_accessibility_permissions() {
            eprintln!("Warning: Accessibility permissions not granted. Global shortcuts may not work properly.");
            eprintln!("Please grant accessibility permissions in System Preferences > Security & Privacy > Privacy > Accessibility");
        }
    }

    let shortcuts = load_shortcuts_from_disk(&app.handle());
    let state: State<ShortcutState> = app.state();
    *state.shortcuts.lock().unwrap() = shortcuts.clone();

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

#[cfg(target_os = "macos")]
fn check_accessibility_permissions() -> bool {
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get name of every process")
        .output();

    match output {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Helper function to normalize shortcut strings for comparison
fn normalize_shortcut_string(shortcut_str: &str) -> String {
    let mut parts: Vec<&str> = shortcut_str.split('+').collect();
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for part in parts.iter_mut() {
        let lower_part = part.to_lowercase();
        match lower_part.as_str() {
            "ctrl" | "control" => modifiers.push("ctrl"),
            "alt" => modifiers.push("alt"),
            "shift" => modifiers.push("shift"),
            "cmd" | "command" | "meta" | "super" => modifiers.push("cmd"),
            _ => {
                // Handle cases like "KeyN" -> "N" or "B" -> "B"
                let mut key_part = *part;
                if key_part.starts_with("Key") && key_part.len() > 3 {
                    key_part = &key_part[3..];
                }
                key = key_part.to_string().to_uppercase();
            }
        }
    }

    modifiers.sort();

    if key.is_empty() {
        modifiers.join("+")
    } else if modifiers.is_empty() {
        key
    } else {
        format!("{}+{}", modifiers.join("+"), key)
    }
}

pub fn handle_global_shortcut(
    app: &AppHandle,
    shortcut: &Shortcut,
    event: GlobalShortcutPluginState,
) {
    if event != GlobalShortcutPluginState::Pressed {
        return;
    }

    // 安全地获取快捷键字符串
    let shortcut_str = shortcut.to_string();
    let triggered_shortcut = normalize_shortcut_string(&shortcut_str);

    println!(
        "Handling shortcut: {} (normalized: {})",
        shortcut_str, triggered_shortcut
    );

    // 安全地获取状态
    let state: State<ShortcutState> = app.state();
    let shortcuts = match state.shortcuts.lock() {
        Ok(shortcuts) => shortcuts,
        Err(e) => {
            eprintln!("Failed to lock shortcuts state: {}", e);
            return;
        }
    };

    // 查找匹配的快捷键
    let matching_shortcut = shortcuts.iter().find(|s| {
        let stored_shortcut = normalize_shortcut_string(&s.shortcut);
        println!(
            "Comparing with stored shortcut: {} (normalized: {})",
            s.shortcut, stored_shortcut
        );
        stored_shortcut == triggered_shortcut
    });

    if let Some(app_shortcut) = matching_shortcut {
        println!(
            "Found matching shortcut: {} -> {}",
            app_shortcut.shortcut, app_shortcut.command_name
        );

        // 安全地执行快捷键动作
        execute_shortcut_action(app, app_shortcut);
    } else {
        println!("No matching shortcut found for: {}", triggered_shortcut);
    }
}

// 分离快捷键动作执行逻辑，便于错误处理
fn execute_shortcut_action(app: &AppHandle, app_shortcut: &crate::shared_types::Shortcut) {
    if app_shortcut.command_name == "toggle_window" {
        if let Some(window) = app.get_webview_window("main") {
            match window.is_visible() {
                Ok(true) => {
                    let _ = window.hide();
                }
                Ok(false) => {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                Err(e) => {
                    eprintln!("Error checking window visibility: {}", e);
                }
            }
        }
    } else {
        println!("Executing command: {}", app_shortcut.command_name);
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.emit("execute_command_by_name", &app_shortcut.command_name) {
                eprintln!("Error emitting command: {}", e);
            }
        }
    }
}

#[tauri::command]
pub async fn set_toggle_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
) -> Result<(), String> {
    let mut shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    let toggle_command_name = "toggle_window";

    // Remove old toggle shortcut if it exists
    if let Some(index) = shortcuts
        .iter()
        .position(|s| s.command_name == toggle_command_name)
    {
        let old_shortcut_str = &shortcuts[index].shortcut;
        let old_tauri_shortcut = Shortcut::from_str(old_shortcut_str).map_err(|e| e.to_string())?;
        if app
            .global_shortcut()
            .is_registered(old_tauri_shortcut.clone())
        {
            app.global_shortcut()
                .unregister(old_tauri_shortcut)
                .map_err(|e| e.to_string())?;
        }
        shortcuts.remove(index);
    }

    // Add new toggle shortcut
    if !shortcut_str.is_empty() {
        let new_shortcut = AppShortcut {
            shortcut: shortcut_str.clone(),
            command_name: toggle_command_name.to_string(),
            command_title: Some("显示/隐藏窗口".to_string()),
        };
        let new_tauri_shortcut = Shortcut::from_str(&shortcut_str).map_err(|e| e.to_string())?;
        app.global_shortcut()
            .register(new_tauri_shortcut)
            .map_err(|e| e.to_string())?;
        shortcuts.push(new_shortcut);
    }

    save_shortcuts_to_disk(&app, &shortcuts);

    Ok(())
}

#[tauri::command]
pub fn get_toggle_shortcut(state: State<ShortcutState>) -> Result<String, String> {
    let shortcuts = state
        .shortcuts
        .lock()
        .map_err(|_| "Failed to lock shortcut state".to_string())?;

    let toggle_command_name = "toggle_window";

    if let Some(shortcut) = shortcuts
        .iter()
        .find(|s| s.command_name == toggle_command_name)
    {
        Ok(shortcut.shortcut.clone())
    } else {
        Ok("".to_string())
    }
}
