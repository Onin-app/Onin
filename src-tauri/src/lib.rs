use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::{Emitter, Manager, State};
use tokio::sync::broadcast;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tauri_plugin_dialog::DialogExt;
use crate::plugin_loader::PluginLoader;
use crate::plugin_manifest::PluginManifest;
use crate::plugin_webview_manager::PluginWebviewManager;

mod app_cache_manager;
pub mod icon_utils;
mod installed_apps;
pub mod plugin_ipc;
pub mod permission_manager;
pub mod plugin_data_manager;
pub mod plugin_loader;
pub mod plugin_manifest;
pub mod shared_types;
mod shortcut_manager;
mod startup_apps_manager;
mod tray_manager;
mod unified_launch_manager;
mod window_manager;
mod plugin_webview_manager;

pub static RDEV_EVENT_CHANNEL: Lazy<(
    broadcast::Sender<rdev::Event>,
    broadcast::Receiver<rdev::Event>,
)> = Lazy::new(|| broadcast::channel(128));

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_installed_plugins(
    plugin_loader: State<'_, Mutex<PluginLoader<tauri::Wry>>>,
) -> Result<Vec<PluginManifest>, String> {
    match plugin_loader.lock() {
        Ok(loader) => Ok(loader.plugins.values().cloned().collect()),
        Err(e) => Err(format!("Failed to lock plugin loader: {}", e)),
    }
}

#[tauri::command]
async fn install_plugin(
    window: tauri::Window,
    plugin_loader: State<'_, Mutex<PluginLoader<tauri::Wry>>>,
    webview_manager: State<'_, Mutex<PluginWebviewManager<tauri::Wry>>>,
    permission_manager: State<'_, Mutex<permission_manager::PermissionManager>>,
    lock_state: State<'_, window_manager::WindowCloseLockState>,
) -> Result<(), String> {
    lock_state.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let (tx, rx) = tokio::sync::oneshot::channel();
    window.dialog().file().add_filter("Zip", &["zip"]).set_parent(&window).pick_file(move |file_path| {
        let _ = tx.send(file_path);
    });

    let result = if let Some(file) = rx.await.unwrap() {
        let mut loader = plugin_loader.lock().unwrap();
        let mut webview_manager = webview_manager.lock().unwrap();
        let mut permission_manager = permission_manager.lock().unwrap();
        let manifest = loader.install_plugin(&file, &mut permission_manager)?;
        webview_manager.create_plugin_webview(&manifest)
    } else {
        Ok(())
    };

    if lock_state.0.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        lock_state.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    result
}

#[tauri::command]
fn uninstall_plugin(
    plugin_id: String,
    plugin_loader: State<'_, Mutex<PluginLoader<tauri::Wry>>>,
    webview_manager: State<'_, Mutex<PluginWebviewManager<tauri::Wry>>>,
    permission_manager: State<'_, Mutex<permission_manager::PermissionManager>>,
) -> Result<(), String> {
    let mut webview_manager = webview_manager.lock().unwrap();
    if let Some(webview) = webview_manager.webviews.remove(&plugin_id) {
        webview.close().map_err(|e| e.to_string())?;
    }

    let mut loader = plugin_loader.lock().unwrap();
    let mut permission_manager = permission_manager.lock().unwrap();
    loader.uninstall_plugin(&plugin_id, &mut permission_manager)
}

#[tauri::command]
fn open_plugin_config(
    plugin_id: String,
    webview_manager: State<'_, Mutex<PluginWebviewManager<tauri::Wry>>>,
) -> Result<(), String> {
    let manager = webview_manager.lock().unwrap();
    if let Some(webview) = manager.get_plugin_webview(&plugin_id) {
        webview.show().map_err(|e| e.to_string())?;
        webview.set_focus().map_err(|e| e.to_string())
    } else {
        Err("Plugin webview not found".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .try_init()
        .ok();

    let is_tray_initially_visible = true;

    let close_window_shortcut =
        Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR).unwrap();

    std::thread::spawn(|| {
        let sender = RDEV_EVENT_CHANNEL.0.clone();
        if let Err(e) = rdev::listen(move |event| {
            let _ = sender.send(event);
        }) {
            eprintln!("[ERROR] rdev could not listen for events: {:?}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostarted"])
                .build(),
        )
        .manage(window_manager::WindowState {
            hiding_initiated_by_command: AtomicBool::new(false),
        })
        .manage(window_manager::WindowCloseLockState(AtomicU32::new(
            0,
        )))
        .manage(window_manager::HideTaskState {
            handle: tokio::sync::Mutex::new(None),
        })
        .manage(tray_manager::TrayVisibilityState(Mutex::new(
            is_tray_initially_visible,
        )))
        .manage(app_cache_manager::AppCache {
            apps: tokio::sync::RwLock::new(None),
        })
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
            window_manager::acquire_window_close_lock,
            window_manager::release_window_close_lock,
            window_manager::close_main_window,
            tray_manager::set_tray_visibility,
            tray_manager::is_tray_visible,
            shortcut_manager::get_toggle_shortcut,
            shortcut_manager::set_toggle_shortcut,
            startup_apps_manager::get_startup_items,
            startup_apps_manager::add_startup_items,
            startup_apps_manager::remove_startup_item,
            plugin_ipc::handle_plugin_message,
            get_installed_plugins,
            install_plugin,
            uninstall_plugin,
            open_plugin_config
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            app.manage(startup_apps_manager::StartupAppsManager::new(
                app.handle().clone(),
            ));
            app.manage(Mutex::new(plugin_ipc::PluginManager::new()));
            app.manage(Mutex::new(permission_manager::PermissionManager::new(&app_handle)));
            app.manage(plugin_data_manager::PluginDataManager::new(&app_handle));

            let mut webview_manager = plugin_webview_manager::PluginWebviewManager::new(app.handle().clone());
            let mut plugin_loader = plugin_loader::PluginLoader::new(app.handle().clone());
            if let Err(e) = plugin_loader.load_plugins() {
                eprintln!("[ERROR] Failed to load plugins: {}", e);
                // Clear plugins if loading failed
                plugin_loader.plugins.clear();
            } else {
                for manifest in plugin_loader.plugins.values() {
                    if let Err(e) = webview_manager.create_plugin_webview(manifest) {
                        eprintln!("[ERROR] Failed to create webview for plugin {}: {}", manifest.id, e);
                    }
                }
            }

            app.manage(Mutex::new(plugin_loader));
            app.manage(Mutex::new(webview_manager));

            #[cfg(desktop)]
            {
                if let Err(e) = shortcut_manager::setup_shortcuts(app) {
                    eprintln!("[ERROR] Failed to set up shortcuts: {}", e);
                }

                if let Err(e) = tray_manager::setup_tray(app) {
                    eprintln!("[ERROR] Failed to set up tray: {}", e);
                }

                if let Err(e) = window_manager::setup_window_events(app) {
                    eprintln!("[ERROR] Failed to set up window events: {}", e);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
