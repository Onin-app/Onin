//! # JS 运行时模块
//!
//! 此模块提供插件的 JavaScript 运行时环境，基于 Deno Core 实现。
//!
//! ## 模块结构
//! - `types`: 类型定义（InvokeResult, PluginTask）
//! - `ops`: Deno 操作定义（console, invoke）
//! - `handlers`: API 处理器（按功能分组）
//! - `runtime`: JsRuntime 创建和配置
//! - `manager`: 插件运行时管理器
//! - `global`: 全局实例管理
//! - `executor`: JS 代码执行

mod executor;
mod global;
mod handlers;
mod manager;
mod ops;
mod runtime;
mod types;

// 重新导出公共接口
pub use executor::execute_js;
pub use global::{
    clear_all_plugin_runtimes, clear_plugin_runtime, get_plugin_runtime_manager,
    init_plugin_runtime_manager,
};
pub use manager::PluginRuntimeManager;
pub use runtime::{create_runtime, create_runtime_with_plugin_id};
