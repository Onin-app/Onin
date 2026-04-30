//! 集中管理所有 Tauri 应用状态
//!
//! 这个模块将所有分散在 lib.rs 中的 `.manage()` 调用汇总到一个地方，
//! 使状态管理更加集中和清晰。

use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;

use crate::{
    app_config, file_search, plugin, plugin_api, shortcut_manager, tray_manager, usage_tracker,
    window_manager,
};

/// 为 Tauri Builder 配置所有应用状态
pub fn setup_managed_state(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        // Plugin 相关状态
        .manage(plugin::PluginStore(Default::default()))
        .manage(plugin::ActivePluginWindow(Mutex::new(None)))
        .manage(plugin::PluginWindowCreating(Mutex::new(
            std::collections::HashSet::new(),
        )))
        .manage(plugin::PluginServerPort(Mutex::new(None)))
        .manage(plugin::PluginWindowToggleDebounce(Mutex::new(
            std::collections::HashMap::new(),
        )))
        .manage(plugin::InlinePluginState::default())
        // Plugin API 相关状态
        .manage(plugin_api::command::CommandExecutionStore(
            Default::default(),
        ))
        .manage(plugin_api::command::PluginLoadedState(Default::default()))
        // 应用配置状态
        .manage(app_config::AppConfigState(Mutex::new(
            app_config::AppConfig::default(),
        )))
        // 使用追踪状态
        .manage(usage_tracker::UsageTrackerState(Mutex::new(None)))
        // 窗口管理状态
        .manage(window_manager::WindowState {
            hiding_initiated_by_command: AtomicBool::new(false),
        })
        .manage(window_manager::WindowCloseLockState(AtomicU32::new(0)))
        .manage(window_manager::HideTaskState {
            handle: tokio::sync::Mutex::new(None),
        })
        // 托盘管理状态
        .manage(tray_manager::TrayVisibilityState(Mutex::new(true)))
        // 快捷键管理状态
        .manage(shortcut_manager::ShortcutState {
            shortcuts: Mutex::new(vec![]),
            last_executed: Mutex::new(std::collections::HashMap::new()),
        })
        // 文件搜索运行状态
        .manage(file_search::FileSearchState::default())
}
