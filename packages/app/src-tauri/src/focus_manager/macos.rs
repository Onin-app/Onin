use std::process::Command;
use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, WebviewWindow, Window};

use objc2_app_kit::NSWorkspace;

pub struct PreviousForegroundApp(pub Mutex<Option<String>>);

pub fn setup(app: &mut App) {
    app.manage(PreviousForegroundApp(Mutex::new(None)));
}

pub fn capture_previous_foreground(app: &AppHandle) {
    let Some(state) = app.try_state::<PreviousForegroundApp>() else {
        return;
    };

    let bundle_id = get_frontmost_app_bundle_id();
    if bundle_id.as_deref() == current_app_bundle_id(app).as_deref() {
        return;
    }

    *state.0.lock().unwrap() = bundle_id;
}

pub fn restore_previous_foreground(app: &AppHandle) {
    let Some(bundle_id) = previous_bundle_id(app) else {
        return;
    };

    activate_app_by_bundle_id(&bundle_id);
}

pub fn focus_webview_window(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn focus_window(window: &Window) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn is_frontmost_self(app: &AppHandle) -> bool {
    get_frontmost_app_bundle_id().as_deref() == current_app_bundle_id(app).as_deref()
}

pub fn previous_bundle_id(app: &AppHandle) -> Option<String> {
    app.try_state::<PreviousForegroundApp>()
        .and_then(|state| state.0.lock().ok().and_then(|guard| guard.clone()))
}

fn current_app_bundle_id(app: &AppHandle) -> Option<String> {
    Some(app.config().identifier.clone())
}

fn get_frontmost_app_bundle_id() -> Option<String> {
    let workspace = NSWorkspace::sharedWorkspace();
    workspace
        .frontmostApplication()
        .and_then(|app| app.bundleIdentifier())
        .map(|id| id.to_string())
}

fn activate_app_by_bundle_id(bundle_id: &str) {
    if let Err(error) = Command::new("osascript")
        .args([
            "-e",
            &format!(r#"tell application id "{}" to activate"#, bundle_id),
        ])
        .output()
    {
        eprintln!("[focus_manager/macos] Failed to activate app {bundle_id}: {error}");
    }
}

