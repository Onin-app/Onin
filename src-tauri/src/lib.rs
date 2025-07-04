use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::ShortcutEvent;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};
use tokio::sync::RwLock;

use crate::installed_apps::AppInfo;
use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod installed_apps;
mod shortcut_manager;
mod tray_manager;

pub struct WindowState {
    hiding_initiated_by_command: AtomicBool,
}

// 新增: 用于缓存已安装应用列表的状态
pub struct AppCache {
    apps: RwLock<Option<Vec<AppInfo>>>,
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

// 新增: 从缓存中获取应用列表的命令
#[tauri::command]
async fn get_installed_apps(cache: State<'_, AppCache>) -> Result<Vec<AppInfo>, String> {
    let apps_guard = cache.apps.read().await;
    // 立即从缓存返回数据，如果缓存为空则返回空列表
    Ok(apps_guard.clone().unwrap_or_default())
}

// 新增: 触发应用列表后台刷新的辅助函数
fn trigger_app_refresh(app: tauri::AppHandle) {
    // 在后台任务中执行刷新操作，避免阻塞UI
    tauri::async_runtime::spawn(async move {
        // 从 AppHandle 获取 State，这样可以确保生命周期正确
        let cache: State<AppCache> = app.state();
        println!("Background refreshing installed apps list...");
        // 调用我们重构的、耗时的函数
        match installed_apps::fetch_installed_apps().await {
            Ok(new_apps) => {
                let mut apps_guard = cache.apps.write().await;
                *apps_guard = Some(new_apps);
                println!("App list cache updated successfully.");
                // 通知前端列表已更新，以便前端可以重新获取
                if let Err(e) = app.emit("apps_updated", ()) {
                    eprintln!("[ERROR] Failed to emit apps_updated event: {}", e);
                }
            }
            Err(e) => eprintln!("[ERROR] Failed to refresh app list: {}", e),
        }
    });
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
        .manage(AppCache {
            apps: RwLock::new(None),
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
            // 使用新的缓存版本替换旧的命令
            get_installed_apps,
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
                trigger_app_refresh(app_handle);
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
                            trigger_app_refresh(handle);
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
