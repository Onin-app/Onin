use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

static IS_SYNCING_ON_EXIT: AtomicBool = AtomicBool::new(false);
static SYNC_ON_EXIT_COMPLETED: AtomicBool = AtomicBool::new(false);

use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

pub mod ai_manager;
mod app_config;
mod command_manager;
mod commands;
pub mod data_manager;
mod extension;
mod extensions;
mod file_command_manager;
mod file_search;
mod focus_manager;
pub mod icon_utils;
mod installed_apps;
mod internal_commands;
mod js_runtime;
mod plugin;
mod plugin_api;
mod plugin_server;
mod setup;
pub mod shared_types;
mod shortcut_manager;
mod state;
pub mod sync;
mod system_commands;
mod telemetry;
mod toast_overlay;
mod tray_manager;
mod unified_launch_manager;
mod usage_tracker;
// TODO: 提升至 pub(crate) 预留供给后续屏幕录像扩展或自定义扩展控制窗口层次与焦点管理重构使用
pub(crate) mod window_manager;

#[cfg(target_os = "macos")]
mod macos_dialog;

/// 创建全局快捷键处理器
fn create_shortcut_handler(
) -> impl Fn(&tauri::AppHandle, &Shortcut, tauri_plugin_global_shortcut::ShortcutEvent)
       + Send
       + Sync
       + 'static {
    move |app, shortcut, event| {
        // macOS 特殊处理：只处理按下事件，避免崩溃
        if event.state() != ShortcutState::Pressed {
            return;
        }

        // 使用 catch_unwind 包装快捷键处理，防止崩溃
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            shortcut_manager::handle_global_shortcut(app, shortcut, event.state());
        }));

        if let Err(e) = result {
            eprintln!("Error in shortcut handler: {:?}", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化并进入 Tokio 运行时上下文，防止依赖异步运行时的插件（如 Aptabase/reqwest）因缺少 reactor 而崩溃
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    let _glitchtip = telemetry::init_glitchtip();

    // 初始化日志
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tauri_plugin_aptabase=warn"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::FULL))
        .with(env_filter)
        .with(sentry_tracing::layer())
        .init();

    let client = reqwest::Client::new();
    // 读取 Aptabase 统计服务的 App Key，如果未设置则使用占位符以确保正常编译
    let aptabase_key = option_env!("APTABASE_KEY")
        .or(option_env!("VITE_APTABASE_KEY"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("A-EU-0000000000");

    // 构建并运行 Tauri 应用
    let app = state::setup_managed_state(tauri::Builder::default())
        .manage(client)
        // 插件
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_aptabase::Builder::new(aptabase_key).build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostarted"])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(create_shortcut_handler())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // 命令
        // 命令
        .invoke_handler(commands::get_invoke_handler())
        // 初始化
        .setup(setup::on_app_setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // tao 在 Windows 上偶发的事件循环重入断言（见 `telemetry::filter_tao_event_loop_race`）
    // 会导致事件循环内部捕获 panic 后于主循环重新抛出；若不兜底，进程会直接崩溃退出。
    // 这里捕获后记录日志并以非零退出码结束，保证可观测性同时避免硬崩溃。
    let event_loop_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        app.run(|app_handle, _event| match _event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                let webdav_config = {
                    let state = app_handle.state::<crate::app_config::AppConfigState>();
                    let x = match state.0.lock() {
                        Ok(lock) => lock.webdav.clone(),
                        Err(_) => crate::app_config::WebDavConfig::default(),
                    };
                    x
                };

                if webdav_config.enabled && webdav_config.sync_on_exit {
                    // 如果备份已完成，直接允许退出（不再拦截）
                    if SYNC_ON_EXIT_COMPLETED.load(Ordering::SeqCst) {
                        return;
                    }

                    api.prevent_exit();

                    if IS_SYNCING_ON_EXIT.swap(true, Ordering::SeqCst) {
                        return;
                    }

                    println!("[sync] 检测到退出自动备份已启用，正在执行自动上传备份...");

                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        match crate::sync::trigger_webdav_sync(
                            app_handle_clone.clone(),
                            "backup".to_string(),
                        )
                        .await
                        {
                            Ok(_) => println!("[sync] 退出自动备份成功！"),
                            Err(e) => eprintln!("[sync] 退出自动备份失败: {}", e),
                        }
                        SYNC_ON_EXIT_COMPLETED.store(true, Ordering::SeqCst);
                        app_handle_clone.exit(0);
                    });
                }
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                window_manager::show_main_window(app_handle);
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Resumed => {
                window_manager::show_main_window(app_handle);
            }
            _ => {}
        });
    }));

    if let Err(payload) = event_loop_result {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".to_string());
        tracing::error!(target: "onin::lib", "event loop terminated by panic: {message}");
        eprintln!("[onin] event loop terminated by panic: {message}");
        std::process::exit(1);
    }
}
