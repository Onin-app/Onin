use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan; // 导入 FmtSpan

mod installed_apps;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        // .with_env_filter("warn") // 设置日志级
        .with_span_events(FmtSpan::FULL) //
        .try_init()
        .ok();

    let toggle_window_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyN);
    // let close_window_shortcut = Shortcut::new(None, Code::Escape);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler({
                    move |app, shortcut, event| {
                        let window = app.get_webview_window("main").unwrap();

                        if shortcut == &toggle_window_shortcut {
                            if event.state == ShortcutState::Pressed {
                                let visible = window.is_visible().unwrap_or(false);

                                if visible {
                                    window.hide().ok();
                                    window.emit("window_visibility", false).unwrap();
                                } else {
                                    window.show().ok();
                                    // window.set_always_on_top(true).ok(); // 置顶
                                    window.set_focus().ok();
                                    window.emit("window_visibility", true).unwrap();
                                }
                            }
                        }

                        // if shortcut == &close_window_shortcut {
                        //     if event.state == ShortcutState::Pressed {
                        //         window.hide().ok();
                        //     }
                        // }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            installed_apps::get_installed_apps,
            installed_apps::open_app
        ])
        .setup(move |app| {
            #[cfg(desktop)]
            {
                println!("Registering Ctrl+N shortcut...");
                app.global_shortcut().register(toggle_window_shortcut)?;
                // app.global_shortcut().register(close_window_shortcut)?;
                println!("Registered!");

                // 获取主窗口
                let window = app.get_webview_window("main").unwrap();
                let window_clone = window.clone();

                // 监听窗口失去焦点事件
                window_clone.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        // 当窗口失去焦点时隐藏
                        // window.hide().ok();
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
