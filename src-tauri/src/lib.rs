use once_cell::sync::Lazy;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
use tokio::sync::broadcast;

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod command_manager;
mod file_command_manager;
pub mod icon_utils;
mod installed_apps;
mod js_runtime;
mod plugin_api;
mod plugin_manager;
pub mod shared_types;
mod shortcut_manager;
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

    // 临时禁用 rdev 监听器以解决 macOS 崩溃问题
    #[cfg(not(target_os = "macos"))]
    {
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
    }
    
    #[cfg(target_os = "macos")]
    {
        eprintln!("[INFO] rdev listener disabled on macOS to prevent crashes");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(plugin_manager::PluginStore(Default::default()))
        .manage(plugin_api::command::CommandExecutionStore(Default::default()))
        .manage(plugin_api::command::PluginLoadedState(Default::default()))
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
        // Manage the shortcut state
        .manage(shortcut_manager::ShortcutState {
            shortcuts: Mutex::new(vec![]),
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler({
                    let close_window_shortcut_clone = close_window_shortcut.clone();
                    move |app, shortcut, event| {
                        // macOS 特殊处理：只处理按下事件，避免崩溃
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        
                        // 安全的快捷键处理逻辑
                        println!("Shortcut event: {:?}, state: {:?}", shortcut, event.state());
                        
                        // 使用 try-catch 包装快捷键处理，防止崩溃
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            shortcut_manager::handle_global_shortcut(app, shortcut, event.state());
                        }));
                        
                        if let Err(e) = result {
                            eprintln!("Error in shortcut handler: {:?}", e);
                        }

                        // ESC 快捷键处理
                        if shortcut == &close_window_shortcut_clone {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("esc_key_pressed", ());
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
            // 注册新的锁命令
            window_manager::acquire_window_close_lock,
            window_manager::release_window_close_lock,
            window_manager::close_main_window, // Register the new command
            // 注册新的命令
            tray_manager::set_tray_visibility,
            tray_manager::is_tray_visible,
            // Add shortcut manager commands
            shortcut_manager::get_shortcuts,
            shortcut_manager::add_shortcut,
            shortcut_manager::remove_shortcut,
            shortcut_manager::set_toggle_shortcut,
            shortcut_manager::get_toggle_shortcut,
            shortcut_manager::set_detach_window_shortcut,
            shortcut_manager::get_detach_window_shortcut,
            // Add startup items manager commands
            file_command_manager::get_file_commands,
            file_command_manager::add_file_commands,
            file_command_manager::remove_file_command,
            // Add system commands
            system_commands::execute_command,
            system_commands::get_basic_commands,
            // 注册插件相关命令
            plugin_manager::load_plugins,
            plugin_manager::refresh_plugins,
            plugin_manager::open_plugin_in_window,
            plugin_manager::execute_plugin_entry,
            plugin_manager::toggle_plugin,
            // 注册 notification 命令
            plugin_api::notification::show_notification,
            plugin_api::command::execute_plugin_command,
            plugin_api::command::plugin_command_result,
            plugin_api::request::plugin_request,
            // 注册 storage 命令
            plugin_api::storage::plugin_storage_set,
            plugin_api::storage::plugin_storage_get,
            plugin_api::storage::plugin_storage_remove,
            plugin_api::storage::plugin_storage_clear,
            plugin_api::storage::plugin_storage_keys,
            plugin_api::storage::plugin_storage_set_items,
            plugin_api::storage::plugin_storage_get_items,
            // 注册文件系统命令
            plugin_api::fs::plugin_fs_read_file,
            plugin_api::fs::plugin_fs_write_file,
            plugin_api::fs::plugin_fs_exists,
            plugin_api::fs::plugin_fs_create_dir,
            plugin_api::fs::plugin_fs_list_dir,
            plugin_api::fs::plugin_fs_delete_file,
            plugin_api::fs::plugin_fs_delete_dir,
            plugin_api::fs::plugin_fs_get_file_info,
            plugin_api::fs::plugin_fs_copy_file,
            plugin_api::fs::plugin_fs_move_file,
            // 注册对话框命令
            plugin_api::dialog::plugin_dialog_message,
            plugin_api::dialog::plugin_dialog_confirm,
            plugin_api::dialog::plugin_dialog_open,
            plugin_api::dialog::plugin_dialog_save,
            // 注册剪贴板命令
            plugin_api::clipboard::plugin_clipboard_read_text,
            plugin_api::clipboard::plugin_clipboard_write_text,
            plugin_api::clipboard::plugin_clipboard_read_image,
            plugin_api::clipboard::plugin_clipboard_write_image,
            plugin_api::clipboard::plugin_clipboard_clear,
            // Command manager commands
            command_manager::get_commands,
            command_manager::update_command,
            command_manager::refresh_commands,
            command_manager::get_plugin_commands_list,
            command_manager::get_plugin_id_mapping,
        ])
        .setup(move |app| {
            // Ensure the app data directory exists on startup.
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                    eprintln!("Failed to create app data directory: {}", e);
                }
            }

            // Initialize the command manager asynchronously
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                command_manager::init(&app_handle).await;
                // Initialize plugin runtime manager
                js_runtime::init_plugin_runtime_manager(app_handle.clone()).await;
            });

            // 托管自定义启动项管理器
            app.manage(file_command_manager::FileCommandManager::new(
                app.handle().clone(),
            ));
            #[cfg(desktop)]
            {
                // Load and register the initial toggle shortcut from the store
                if let Err(e) = shortcut_manager::setup_shortcuts(app) {
                    eprintln!("[ERROR] Failed to set up shortcuts: {}", e);
                }

                // Register the ESC shortcut
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let close_window_shortcut =
                    Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR).unwrap();
                if !app
                    .global_shortcut()
                    .is_registered(close_window_shortcut.clone())
                {
                    if let Err(e) = app.global_shortcut().register(close_window_shortcut) {
                        eprintln!("[ERROR] Failed to register ESC shortcut: {}", e);
                    }
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
