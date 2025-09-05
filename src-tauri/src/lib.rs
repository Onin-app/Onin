use once_cell::sync::Lazy;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
use tokio::sync::broadcast;

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod app_cache_manager;
pub mod icon_utils;
mod installed_apps;
mod js_runtime;
mod plugin_api;
mod plugin_manager;
pub mod shared_types;
mod shortcut_manager;
mod startup_apps_manager;
mod system_commands;
mod tray_manager;
mod unified_launch_manager;
mod window_manager;

// 创建一个全局的、一次性的通道，用于广播 rdev 的输入事件。
// 这样我们只需要一个系统监听线程，而不是每次失焦都创建一个。
pub static RDEV_EVENT_CHANNEL: Lazy<(
    broadcast::Sender<rdev::Event>,
    broadcast::Receiver<rdev::Event>,
)> = Lazy::new(|| broadcast::channel(128));

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .try_init()
        .ok();

    let is_tray_initially_visible = true;

    // Parse the close window shortcut once to be used in the handler
    let close_window_shortcut =
        Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR).unwrap();

    // 在一个单独的线程中启动全局事件监听器
    std::thread::spawn(|| {
        let sender = RDEV_EVENT_CHANNEL.0.clone();
        if let Err(e) = rdev::listen(move |event| {
            // 尝试发送事件，如果另一端没有监听者也无所谓
            let _ = sender.send(event);
        }) {
            eprintln!("[ERROR] rdev could not listen for events: {:?}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(plugin_manager::PluginStore(Default::default()))
        .register_uri_scheme_protocol("plugin", plugin_manager::handle_plugin_protocol)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostarted"]) // 应用自启时接收的参数
                .build(),
        )
        .manage(window_manager::WindowState {
            hiding_initiated_by_command: AtomicBool::new(false),
        })
        // 新增：托管窗口关闭锁的状态
        .manage(window_manager::WindowCloseLockState(AtomicU32::new(0)))
        .manage(window_manager::HideTaskState {
            handle: tokio::sync::Mutex::new(None),
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
                    let close_window_shortcut_clone = close_window_shortcut.clone();
                    move |app, shortcut, event| {
                        shortcut_manager::handle_toggle_shortcut(app, shortcut, &event.state);

                        if shortcut == &close_window_shortcut_clone {
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
            unified_launch_manager::get_all_launchable_items,
            installed_apps::open_app,
            // 注册新的锁命令
            window_manager::acquire_window_close_lock,
            window_manager::release_window_close_lock,
            window_manager::close_main_window, // Register the new command
            // 注册新的命令
            tray_manager::set_tray_visibility,
            tray_manager::is_tray_visible,
            // Add shortcut manager commands
            shortcut_manager::get_toggle_shortcut,
            shortcut_manager::set_toggle_shortcut,
            // Add startup items manager commands
            startup_apps_manager::get_startup_items,
            startup_apps_manager::add_startup_items,
            startup_apps_manager::remove_startup_item,
            // Add system commands
            system_commands::shutdown,
            system_commands::reboot,
            system_commands::sleep,
            system_commands::lock_screen,
            system_commands::logout,
            system_commands::open_app_data_dir,
            // 注册插件相关命令
            plugin_manager::load_plugins,
            plugin_manager::execute_plugin_entry,
            // 注册 notification 命令
            plugin_api::notification::show_notification,
        ])
        .setup(move |app| {
            // Ensure the app data directory exists on startup.
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                    eprintln!("Failed to create app data directory: {}", e);
                }
            }

            // 托管自定义启动项管理器
            app.manage(startup_apps_manager::StartupAppsManager::new(
                app.handle().clone(),
            ));
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

                // Set up window-specific event listeners.
                if let Err(e) = window_manager::setup_window_events(app) {
                    eprintln!("[ERROR] Failed to set up window events: {}", e);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
