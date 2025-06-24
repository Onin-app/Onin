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

#[tauri::command]
fn close_main_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().ok();
        window.emit("window_visibility", false).unwrap_or_default();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        // .with_env_filter("warn") // 设置日志级
        .with_span_events(FmtSpan::FULL) //
        .try_init()
        .ok();

    let toggle_window_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyN);
    let close_window_shortcut = Shortcut::new(None, Code::Escape);

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

                        if shortcut == &close_window_shortcut {
                            if event.state == ShortcutState::Pressed {
                                window.emit("esc_key_pressed", ()).unwrap_or_default();
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
            close_main_window // Register the new command
        ])
        .setup(move |app| {
            #[cfg(desktop)]
            {
                // 切换窗口的快捷键需要一直保持，以便随时可以唤出窗口
                println!("Registering toggle shortcut (Ctrl+Shift+N)...");
                app.global_shortcut().register(toggle_window_shortcut)?;
                println!("Registered!");

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
                            // 当窗口失去焦点时，注销 "Esc" 快捷键并隐藏窗口
                            println!("Window lost focus, unregistering Esc shortcut.");
                            app_handle
                                .global_shortcut()
                                .unregister(close_window_shortcut)
                                .unwrap_or_else(|err| {
                                    eprintln!("[ERROR] Failed to unregister Esc shortcut: {}", err);
                                });
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
