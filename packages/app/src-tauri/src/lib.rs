use once_cell::sync::Lazy;
use std::str::FromStr;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
use tokio::sync::broadcast;

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

mod app_config;
mod command_manager;
mod commands;
mod file_command_manager;
pub mod icon_utils;
mod installed_apps;
mod js_runtime;
mod plugin;
mod plugin_api;
mod plugin_server;
mod setup;
pub mod shared_types;
mod shortcut_manager;
mod state;
mod system_commands;
mod tray_manager;
mod unified_launch_manager;
mod usage_tracker;
mod window_manager;

// 创建一个全局的、一次性的通道，用于广播 rdev 的输入事件。
// 这样我们只需要一个系统监听线程，而不是每次失焦都创建一个。
pub static RDEV_EVENT_CHANNEL: Lazy<(
    broadcast::Sender<rdev::Event>,
    broadcast::Receiver<rdev::Event>,
)> = Lazy::new(|| broadcast::channel(128));

/// 启动 rdev 全局事件监听器
///
/// 在一个单独的线程中监听全局输入事件，并通过 channel 广播出去。
/// macOS 上暂时禁用以避免崩溃问题。
fn start_rdev_listener() {
    #[cfg(not(target_os = "macos"))]
    {
        std::thread::spawn(|| {
            let sender = RDEV_EVENT_CHANNEL.0.clone();
            if let Err(e) = rdev::listen(move |event| {
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
}

/// 创建全局快捷键处理器
fn create_shortcut_handler(
    close_window_shortcut: Shortcut,
) -> impl Fn(&tauri::AppHandle, &Shortcut, tauri_plugin_global_shortcut::ShortcutEvent)
       + Send
       + Sync
       + 'static {
    move |app, shortcut, event| {
        // macOS 特殊处理：只处理按下事件，避免崩溃
        if event.state() != ShortcutState::Pressed {
            return;
        }

        println!("Shortcut event: {:?}, state: {:?}", shortcut, event.state());

        // 使用 catch_unwind 包装快捷键处理，防止崩溃
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            shortcut_manager::handle_global_shortcut(app, shortcut, event.state());
        }));

        if let Err(e) = result {
            eprintln!("Error in shortcut handler: {:?}", e);
        }

        // ESC 快捷键处理
        if shortcut == &close_window_shortcut {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("esc_key_pressed", ());
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .try_init()
        .ok();

    // 启动全局事件监听器
    start_rdev_listener();

    // 解析关闭窗口快捷键
    let close_window_shortcut =
        Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR).unwrap();

    // 构建并运行 Tauri 应用
    state::setup_managed_state(tauri::Builder::default())
        // 插件
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostarted"])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(create_shortcut_handler(close_window_shortcut))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        // 自定义协议
        .register_uri_scheme_protocol("plugin", plugin::handle_plugin_protocol)
        // 命令
        .invoke_handler(commands::get_invoke_handler())
        // 初始化
        .setup(setup::on_app_setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
