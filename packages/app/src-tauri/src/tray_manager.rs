use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, State,
};

// 托盘图标的唯一ID
pub const TRAY_ICON_ID: &str = "main_tray_icon";

// 用于管理托盘图标可见性的状态
pub struct TrayVisibilityState(pub Mutex<bool>);

// 设置托盘图标可见性的命令
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

// 获取托盘图标当前可见性状态的命令
#[tauri::command]
pub fn is_tray_visible(state: State<'_, TrayVisibilityState>) -> bool {
    *state.0.lock().unwrap()
}

// 用于构建和初始化托盘图标的函数
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;

    TrayIconBuilder::with_id(TRAY_ICON_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
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