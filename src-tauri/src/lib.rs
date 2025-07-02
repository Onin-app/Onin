use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::ShortcutEvent;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod installed_apps;
mod shortcut_manager;

pub struct WindowState {
    hiding_initiated_by_command: AtomicBool,
}

// 新增：用于管理托盘图标可见性的状态
pub struct TrayVisibilityState(pub Mutex<bool>);

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn close_main_window(app: tauri::AppHandle, state: State<WindowState>) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🥳 这是ESC");
        // 在隐藏窗口前设置标志位为 true
        // 这表示后续的失焦事件是预期的
        state
            .hiding_initiated_by_command
            .store(true, Ordering::Relaxed);
        window.hide().ok();
        window.emit("window_visibility", &false).unwrap_or_default();
    }
}

const TRAY_ICON_ID: &str = "main_tray_icon";

// 新增：设置托盘图标可见性的命令
#[tauri::command]
fn set_tray_visibility(
    visible: bool,
    app: tauri::AppHandle,
    state: State<'_, TrayVisibilityState>,
) -> Result<(), String> {
    // 通过 app_handle 获取托盘图标的引用
    if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
        // 调用 set_visible 方法来显示或隐藏图标
        tray.set_visible(visible).map_err(|e| e.to_string())?;
        // 更新我们自己维护的可见性状态
        *state.0.lock().unwrap() = visible;
        Ok(())
    } else {
        Err("Tray icon not found.".to_string())
    }
}

// 新增：获取托盘图标当前可见性状态的命令
#[tauri::command]
fn is_tray_visible(state: State<'_, TrayVisibilityState>) -> bool {
    *state.0.lock().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        // .with_env_filter("warn") // 设置日志级
        .with_span_events(FmtSpan::FULL) //
        .try_init()
        .ok();

    // 托盘图标默认是可见的
    let is_tray_initially_visible = true;

    // The toggle shortcut is now managed by shortcut_manager.rs
    let close_window_shortcut = Shortcut::new(None, Code::Escape);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostarted"]) // 应用自启时接收的参数
                .build(),
        )
        .manage(WindowState {
            hiding_initiated_by_command: AtomicBool::new(false),
        })
        // 托管托盘图标的可见性状态
        .manage(TrayVisibilityState(Mutex::new(is_tray_initially_visible)))
        // Manage the shortcut state
        .manage(shortcut_manager::ShortcutState {
            toggle_shortcut: Mutex::new(
                Shortcut::from_str(shortcut_manager::DEFAULT_TOGGLE_SHORTCUT).unwrap(),
            ),
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler({
                    move |app: &tauri::AppHandle, shortcut: &Shortcut, event: ShortcutEvent| {
                        // Let the shortcut manager handle its own shortcut, passing a reference to the event
                        shortcut_manager::handle_toggle_shortcut(app, shortcut, &event.state);

                        if shortcut == &close_window_shortcut {
                            if event.state == ShortcutState::Pressed {
                                if let Some(window) = app.get_webview_window("main") {
                                    window.emit("esc_key_pressed", ()).unwrap_or_default();
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            installed_apps::get_installed_apps,
            installed_apps::open_app,
            close_main_window, // Register the new command
            // 注册新的命令
            set_tray_visibility,
            is_tray_visible,
            // Add shortcut manager commands
            shortcut_manager::get_toggle_shortcut,
            shortcut_manager::set_toggle_shortcut
        ])
        .setup(move |app| {
            #[cfg(desktop)]
            {
                // Load and register the initial toggle shortcut from the store
                if let Err(e) = shortcut_manager::setup_shortcuts(app) {
                    eprintln!("[ERROR] Failed to set up shortcuts: {}", e);
                }

                // 获取主窗口
                let window = app.get_webview_window("main").unwrap();
                let window_clone = window.clone();
                let app_handle = app.handle().clone();

                // 监听窗口焦点事件，动态管理 Esc 快捷键
                window_clone.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(true) => {
                            // 当窗口获得焦点时，注册 "Esc" 快捷键
                            println!("Window focused, registering Esc shortcut.");
                            app_handle
                                .global_shortcut()
                                .register(close_window_shortcut)
                                .unwrap_or_else(|err| {
                                    eprintln!("[ERROR] Failed to register Esc shortcut: {}", err);
                                });
                        }
                        tauri::WindowEvent::Focused(false) => {
                            let state: State<WindowState> = app_handle.state();

                            // 窗口失焦时总是注销快捷键
                            app_handle
                                .global_shortcut()
                                .unregister(close_window_shortcut)
                                .unwrap_or_else(|err| {
                                    eprintln!("[ERROR] Failed to unregister Esc shortcut: {}", err);
                                });

                            // 原子地检查并重置标志位
                            // 如果之前是 true，说明是由 close_main_window 命令触发的
                            if state
                                .hiding_initiated_by_command
                                .swap(false, Ordering::Relaxed)
                            {
                                println!(
                                    "Window focus lost due to command. Skipping redundant hide."
                                );
                            } else {
                                println!("Window lost focus naturally. Hiding window.");
                                window.hide().ok();
                                window.emit("window_visibility", &false).unwrap();
                            }
                        }
                        _ => {}
                    }
                });

                // 创建托盘图标
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&quit_i])?;
                let _tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_tray_icon_event(|tray, event| match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            println!("left click pressed and released");
                            // in this example, let's show and focus the main window when the tray is clicked
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {
                            println!("unhandled event {event:?}");
                        }
                    })
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            println!("quit menu item was clicked");
                            app.exit(0);
                        }
                        _ => {
                            println!("menu item {:?} not handled", event.id);
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
