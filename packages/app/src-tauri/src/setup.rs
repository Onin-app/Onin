//! 应用初始化逻辑
//!
//! 这个模块封装了 Tauri `setup()` 闭包中的所有初始化逻辑，
//! 使 `lib.rs` 更加简洁，同时提高代码的可读性和可维护性。

use std::str::FromStr;
use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{
    app_config, command_manager, file_command_manager, js_runtime, plugin, plugin_api,
    plugin_server, shortcut_manager, system_commands, tray_manager, window_manager,
};

/// 应用启动时的主要初始化逻辑
///
/// 这个函数在 Tauri 的 `setup()` 阶段被调用，负责：
/// 1. 创建必要的目录结构
/// 2. 加载应用配置
/// 3. 启动平台特定的服务
/// 4. 初始化异步任务
/// 5. 设置快捷键和托盘
/// 6. 配置窗口事件
pub fn on_app_setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 确保应用数据目录存在
    ensure_app_data_dir(app)?;

    // 2. 加载并应用应用配置
    load_app_config(app);

    // 3. 启动平台特定服务 (仅 Windows)
    #[cfg(target_os = "windows")]
    plugin_api::clipboard::start_clipboard_monitor(app.handle().clone());

    // Initialize Clipboard Extension (Native)
    crate::extensions::clipboard::init(app.handle());

    // 4. 初始化调度器状态
    init_scheduler_state(app);

    // 5. 注册文件命令管理器
    app.manage(file_command_manager::FileCommandManager::new(
        app.handle().clone(),
    ));

    // 6. 启动异步初始化任务
    spawn_async_init_tasks(app.handle().clone());

    // 7. 桌面平台特定设置
    #[cfg(desktop)]
    setup_desktop_features(app)?;

    Ok(())
}

/// 确保应用数据目录存在
fn ensure_app_data_dir(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
            eprintln!("Failed to create app data directory: {}", e);
        }
    }
    Ok(())
}

/// 加载应用配置并更新状态
fn load_app_config(app: &App) {
    let config = app_config::load_config(app.handle()).unwrap_or_default();
    let config_state = app.state::<app_config::AppConfigState>();
    if let Ok(mut current_config) = config_state.0.lock() {
        *current_config = config;
    };
}

/// 初始化调度器状态
fn init_scheduler_state(app: &mut App) {
    // macOS: 初始化前一个应用追踪器
    #[cfg(target_os = "macos")]
    app.manage(system_commands::MacOSPreviousApp(std::sync::Mutex::new(None)));
    
    let scheduler_state = tauri::async_runtime::block_on(async {
        plugin_api::scheduler::SchedulerState::new().await
    });

    match scheduler_state {
        Ok(state) => {
            app.manage(state);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
        }
    }
}

/// 启动异步初始化任务
///
/// 这些任务包括：
/// - 启动插件 HTTP 服务器
/// - 加载插件
/// - 初始化命令管理器
/// - 初始化插件运行时管理器
/// - 初始化调度器
fn spawn_async_init_tasks(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 1. 启动插件 HTTP 服务器
        let plugins_dir = match app_handle.path().app_data_dir() {
            Ok(dir) => dir.join("plugins"),
            Err(e) => {
                eprintln!("[ERROR] Failed to get plugins directory: {}", e);
                return;
            }
        };

        match plugin_server::start_plugin_server(plugins_dir).await {
            Ok(port) => {
                println!("[INFO] Plugin server started on port {}", port);
                let server_port = app_handle.state::<plugin::PluginServerPort>();
                *server_port.0.lock().unwrap() = Some(port);
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to start plugin server: {}", e);
            }
        }

        // 2. 加载插件
        let plugin_store = app_handle.state::<plugin::PluginStore>();
        if let Err(e) = plugin::load_plugins(app_handle.clone(), plugin_store) {
            eprintln!("[ERROR] Failed to load plugins on startup: {}", e);
        }

        // 3. 初始化命令管理器
        command_manager::init(&app_handle).await;

        // 4. 初始化插件运行时管理器
        js_runtime::init_plugin_runtime_manager(app_handle.clone()).await;

        // 5. 初始化调度器
        if let Err(e) = plugin_api::scheduler::init_scheduler(&app_handle).await {
            eprintln!("[ERROR] Failed to initialize scheduler: {}", e);
        }
    });
}

/// 设置桌面平台特有的功能
#[cfg(desktop)]
fn setup_desktop_features(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 设置快捷键
    if let Err(e) = shortcut_manager::setup_shortcuts(app) {
        eprintln!("[ERROR] Failed to set up shortcuts: {}", e);
    }

    // 2. 注册 ESC 快捷键
    let close_window_shortcut = Shortcut::from_str(window_manager::CLOSE_WINDOW_SHORTCUT_STR)?;
    if !app
        .global_shortcut()
        .is_registered(close_window_shortcut.clone())
    {
        if let Err(e) = app.global_shortcut().register(close_window_shortcut) {
            eprintln!("[ERROR] Failed to register ESC shortcut: {}", e);
        }
    }

    // 3. 创建托盘图标
    if let Err(e) = tray_manager::setup_tray(app) {
        eprintln!("[ERROR] Failed to set up tray: {}", e);
    }

    // 4. 设置窗口事件监听
    if let Err(e) = window_manager::setup_window_events(app) {
        eprintln!("[ERROR] Failed to set up window events: {}", e);
    }

    Ok(())
}
