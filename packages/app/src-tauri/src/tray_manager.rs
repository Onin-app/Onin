use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, State,
};

pub const TRAY_ICON_ID: &str = "main_tray_icon";

pub struct TrayVisibilityState(pub Mutex<bool>);

#[tauri::command]
pub fn set_tray_visibility(
    visible: bool,
    app: AppHandle,
    state: State<'_, TrayVisibilityState>,
) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
        tray.set_visible(visible).map_err(|e| e.to_string())?;
        *state.0.lock().unwrap() = visible;
        Ok(())
    } else {
        Err("Tray icon not found.".to_string())
    }
}

#[tauri::command]
pub fn is_tray_visible(state: State<'_, TrayVisibilityState>) -> bool {
    *state.0.lock().unwrap()
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip(app.config().product_name.clone().unwrap())
        .show_menu_on_left_click(true)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    crate::focus_manager::capture_previous_foreground(&app);
                    crate::focus_manager::focus_webview_window(&window);
                }
            }
            _ => {}
        })
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .build(app)?;

    Ok(())
}
