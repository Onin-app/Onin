use std::str::FromStr;
use std::sync::Mutex;
use tauri::{App, AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as GlobalShortcutPluginState,
};
use tauri_plugin_store::StoreExt;

// State to hold the currently configured shortcut
pub struct ShortcutState {
    pub toggle_shortcut: Mutex<Shortcut>,
}

// Constants for store key and default value
const TOGGLE_SHORTCUT_KEY: &str = "toggle_shortcut";
pub const DEFAULT_TOGGLE_SHORTCUT: &str = "Ctrl+Shift+N";

// Command to get the current shortcut from the state
#[tauri::command]
pub fn get_toggle_shortcut(state: State<ShortcutState>) -> Result<String, String> {
    let shortcut = state
        .toggle_shortcut
        .lock()
        .map_err(|_| "Failed to lock shortcut state".to_string())?;
    Ok(shortcut.to_string())
}

// Command to set a new shortcut, register it, and save it to the store
#[tauri::command]
pub async fn set_toggle_shortcut(
    app: AppHandle,
    state: State<'_, ShortcutState>,
    shortcut_str: String,
) -> Result<(), String> {
    let new_shortcut = Shortcut::from_str(&shortcut_str).map_err(|e| e.to_string())?;

    let mut current_shortcut = state
        .toggle_shortcut
        .lock()
        .map_err(|_| "Failed to acquire lock on shortcut state".to_string())?;

    if new_shortcut == *current_shortcut {
        return Ok(()); // No change needed
    }

    // Unregister the old shortcut if it was registered
    if app
        .global_shortcut()
        .is_registered(current_shortcut.clone())
    {
        app.global_shortcut()
            .unregister(current_shortcut.clone())
            .map_err(|e| e.to_string())?;
    }

    // Register the new shortcut. This will return an error if the shortcut is already taken.
    app.global_shortcut()
        .register(new_shortcut.clone())
        .map_err(|e| e.to_string())?;

    // Update the state
    *current_shortcut = new_shortcut.clone();

    // Save to the persistent store - 修复：使用正确的 Store API
    let store = app.store(".settings.dat").map_err(|e| e.to_string())?;
    store.set(TOGGLE_SHORTCUT_KEY, shortcut_str);
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

// Function to be called during app setup to load and register the initial shortcut
pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // 修复：使用正确的 Store API
    let store = app.store(".settings.dat")?;

    let shortcut_str = store
        .get(TOGGLE_SHORTCUT_KEY)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| DEFAULT_TOGGLE_SHORTCUT.to_string());

    let shortcut = Shortcut::from_str(&shortcut_str)
        .unwrap_or_else(|_| Shortcut::from_str(DEFAULT_TOGGLE_SHORTCUT).unwrap());

    let state: State<ShortcutState> = app.state();
    *state
        .toggle_shortcut
        .lock()
        .map_err(|_| "Failed to lock shortcut state for setup")? = shortcut.clone();

    app.global_shortcut().register(shortcut.clone())?;
    Ok(())
}

// Function to handle the global shortcut toggle event
pub fn handle_toggle_shortcut(
    app: &AppHandle,
    shortcut: &Shortcut,
    event: &GlobalShortcutPluginState,
) {
    let state: State<ShortcutState> = app.state();
    let toggle_shortcut_lock = state.toggle_shortcut.lock();

    if let Ok(toggle_shortcut) = toggle_shortcut_lock {
        if shortcut == &*toggle_shortcut && *event == GlobalShortcutPluginState::Pressed {
            // 修复：使用正确的窗口获取方法
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    println!("🥳 这是快捷键, 隐藏窗口");
                    let _ = window.hide();
                    let _ = window.emit("window_visibility", false);
                } else {
                    println!("🥳 这是快捷键, 显示窗口");
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("window_visibility", true);
                }
            }
        }
    } else {
        eprintln!("[ERROR] Could not acquire lock on shortcut state in handler.");
    }
}
