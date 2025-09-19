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
    let shortcuts = load_shortcuts_from_disk(&app.handle());
    let state: State<ShortcutState> = app.state();
    *state.shortcuts.lock().unwrap() = shortcuts.clone();

    for shortcut in shortcuts {
        let tauri_shortcut = Shortcut::from_str(&shortcut.shortcut)?;
        app.global_shortcut().register(tauri_shortcut)?;
    }

    Ok(())
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

    let triggered_shortcut = normalize_shortcut_string(&shortcut.to_string());
    println!(
        "Handling shortcut: {} (normalized: {})",
        shortcut.to_string(),
        triggered_shortcut
    );

    let state: State<ShortcutState> = app.state();
    let shortcuts = state.shortcuts.lock().unwrap();

    for s in shortcuts.iter() {
        let stored_shortcut = normalize_shortcut_string(&s.shortcut);
        println!(
            "Comparing with stored shortcut: {} (normalized: {})",
            s.shortcut, stored_shortcut
        );
    }

    if let Some(app_shortcut) = shortcuts
        .iter()
        .find(|s| normalize_shortcut_string(&s.shortcut) == triggered_shortcut)
    {
        println!(
            "Found matching shortcut: {} -> {}",
            app_shortcut.shortcut, app_shortcut.command_name
        );
        if app_shortcut.command_name == "toggle_window" {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        } else {
            println!("Executing command: {}", app_shortcut.command_name);
            let _ = app
                .get_webview_window("main")
                .unwrap()
                .emit("execute_command_by_name", &app_shortcut.command_name);
        }
    } else {
        println!("No matching shortcut found for: {}", triggered_shortcut);
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
