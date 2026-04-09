//! # 插件管理模块
//!
//! 对外暴露插件系统中仍被应用其他模块直接使用的类型和入口。

pub mod context;
pub mod executor;
pub mod inline;
pub mod installer;
pub mod lifecycle;
pub mod loader;
pub mod settings;
pub mod state;
pub mod types;
pub mod window;

// 应用其他模块直接依赖的类型重导出
pub use inline::InlinePluginState;
pub use types::{
    ActivePluginWindow, LoadedPlugin, PluginCommandManifest, PluginServerPort,
    PluginSettingsSchema, PluginStore, PluginWindowCreating, PluginWindowToggleDebounce,
};

// 应用其他模块直接依赖的入口重导出
pub use executor::execute_plugin_entry;
pub use loader::{get_loaded_plugins, load_plugins};
pub use settings::register_plugin_settings_schema;
