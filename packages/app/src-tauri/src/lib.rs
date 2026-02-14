use std::str::FromStr;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

pub mod ai_manager;
mod app_config;
mod command_manager;
mod commands;
mod extension;
mod extensions;
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

#[cfg(target_os = "macos")]
mod macos_dialog;

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
                let _ = window.emit("escape_pressed", ());
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

    // 解析关闭窗口快捷键
    let close_window_shortcut =
        Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR).unwrap();

    // 构建并运行 Tauri 应用
    let app = state::setup_managed_state(tauri::Builder::default())
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
        // 命令
        .invoke_handler(commands::get_invoke_handler())
        // 初始化
        .setup(setup::on_app_setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        match event {
            tauri::RunEvent::Reopen { .. } => {
                window_manager::show_main_window(app_handle);
            }
            tauri::RunEvent::Resumed => {
                window_manager::show_main_window(app_handle);
            }
            _ => {}
        }
    });
}
