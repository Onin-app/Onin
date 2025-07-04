use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::ShortcutEvent;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod app_cache_manager;
mod installed_apps;
mod shortcut_manager;
mod tray_manager;

pub struct WindowState {
    hiding_initiated_by_command: AtomicBool,
}

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
        .manage(tray_manager::TrayVisibilityState(Mutex::new(
            is_tray_initially_visible,
        )))
        // 新增：托管应用列表缓存的状态
        .manage(app_cache_manager::AppCache {
            apps: tokio::sync::RwLock::new(None),
        })
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
            app_cache_manager::get_installed_apps,
            installed_apps::open_app,
            close_main_window, // Register the new command
            // 注册新的命令
            tray_manager::set_tray_visibility,
            tray_manager::is_tray_visible,
            // Add shortcut manager commands
            shortcut_manager::get_toggle_shortcut,
            shortcut_manager::set_toggle_shortcut
        ])
        .setup(move |app| {
            // 新增：应用启动时，在后台异步获取一次应用列表
            {
                let app_handle = app.handle().clone();
                app_cache_manager::trigger_app_refresh(app_handle);
            }

            #[cfg(desktop)]
            {
                // Load and register the initial toggle shortcut from the store
                if let Err(e) = shortcut_manager::setup_shortcuts(app) {
                    eprintln!("[ERROR] Failed to set up shortcuts: {}", e);
                }

                // 创建托盘图标
                if let Err(e) = tray_manager::setup_tray(app) {
                    eprintln!("[ERROR] Failed to set up tray: {}", e);
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

                            // 新增：窗口获得焦点时，触发一次静默刷新
                            println!("Window focused, triggering silent app list refresh.");
                            let handle = app_handle.clone();
                            app_cache_manager::trigger_app_refresh(handle);
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
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
