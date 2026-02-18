//! # 插件管理模块
//!
//! 该模块负责插件系统的完整生命周期管理，包括：
//! - 插件类型定义和数据结构
//! - 插件状态持久化
//! - 插件加载和刷新
//! - 插件窗口管理
//! - 插件安装、导入和卸载
//! - 插件执行
//! - 自定义协议处理
//! - Tauri API 桥接
//!
//! ## 模块结构
//!
//! - `types`: 数据结构定义
//! - `state`: 状态持久化
//! - `loader`: 插件加载
//! - `settings`: 插件设置管理
//! - `window`: 窗口管理
//! - `executor`: 插件执行
//! - `installer`: 安装/卸载
//! - `protocol`: 协议处理
//! - `bridge`: API 桥接

pub mod bridge;
pub mod executor;
pub mod inline;
pub mod installer;
pub mod loader;
pub mod protocol;
pub mod settings;
pub mod state;
pub mod types;
pub mod window;

// ============================================================================
// 重导出公共 API
// ============================================================================

// 类型重导出
pub use types::{
    ActivePluginWindow, InstallSource, LoadedPlugin, PluginCommandKeyword, PluginCommandManifest,
    PluginCommandMatch, PluginDetail, PluginManifest, PluginPermissions, PluginServerPort,
    PluginSettingsSchema, PluginState, PluginStates, PluginStore, PluginWindowCreating,
    PluginWindowStates, PluginWindowToggleDebounce, SettingField, SettingOption, WindowBounds,
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, MAX_WINDOW_HEIGHT, MAX_WINDOW_WIDTH,
    MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH,
};

// 辅助函数重导出
pub use types::{
    find_all_versions, find_plugin_by_id, find_plugin_by_id_mut, make_plugin_dir_name,
    parse_plugin_dir_name,
};

// 状态操作重导出
pub use state::{
    get_plugin_settings_path, get_plugin_states_path, get_plugin_window_states_path,
    load_plugin_settings, load_plugin_states, load_plugin_window_states, save_plugin_states,
    save_plugin_window_state,
};

// 桥接功能重导出
// 桥接功能重导出
pub use bridge::{fix_asset_paths, PLUGIN_WINDOW_CONTROLS_SCRIPT, PLUGIN_WINDOW_TOPBAR_TEMPLATE};

// 协议处理重导出
pub use protocol::handle_plugin_protocol;

// ============================================================================
// Tauri 命令重导出
// ============================================================================

// 加载器命令
pub use loader::{get_loaded_plugins, load_plugins, refresh_plugins};

// 设置命令
pub use settings::{
    get_plugin_detail, get_plugin_server_port, get_plugin_settings, get_plugin_with_schema,
    register_plugin_settings_schema, save_plugin_settings, toggle_plugin,
    toggle_plugin_auto_detach,
};

// 窗口命令
pub use window::{
    open_plugin_in_window, plugin_close_window, plugin_is_maximized, plugin_maximize_window,
    plugin_minimize_window, plugin_set_focus, plugin_show_window, plugin_start_dragging,
    plugin_unmaximize_window, plugin_unminimize_window,
};

// 内联插件命令
pub use inline::{
    close_inline_plugin, hide_inline_plugin, send_inline_plugin_message, show_inline_plugin,
    update_inline_plugin_bounds, InlinePluginState,
};

// 执行器命令
pub use executor::execute_plugin_entry;

// 安装器命令
pub use installer::{download_and_install_plugin, import_plugin, uninstall_plugin};

// ============================================================================
// 初始化函数
// ============================================================================

/// 创建插件相关的应用状态结构
///
/// 返回一个元组，包含所有需要注册到 Tauri 应用的状态
pub fn create_plugin_states() -> (
    PluginStore,
    ActivePluginWindow,
    PluginWindowCreating,
    PluginServerPort,
    PluginWindowToggleDebounce,
) {
    (
        PluginStore(std::sync::Mutex::new(std::collections::HashMap::new())),
        ActivePluginWindow(std::sync::Mutex::new(None)),
        PluginWindowCreating(std::sync::Mutex::new(std::collections::HashSet::new())),
        PluginServerPort(std::sync::Mutex::new(None)),
        PluginWindowToggleDebounce(std::sync::Mutex::new(std::collections::HashMap::new())),
    )
}
