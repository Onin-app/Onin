//! # 命令管理器模块
//!
//! 管理应用中所有可用命令的加载、存储和刷新
//!
//! ## 模块结构
//! - `storage`: 命令存储（读写 commands.json）
//! - `refresh`: 刷新逻辑和事件
//! - `commands`: Tauri 命令
//! - `generators`: 命令生成器

pub mod commands;
mod generators;
mod refresh;
mod storage;

// 重新导出公共接口（非 Tauri 命令）
pub use generators::{get_plugin_commands, get_plugin_id_name_mapping};
pub use refresh::RefreshResult;
pub use storage::load_commands;

use tauri::{AppHandle, Emitter};

/// 初始化命令管理器
///
/// 加载命令并通知前端命令已就绪
pub async fn init(app: &AppHandle) {
    storage::load_commands(app).await;
    // 通知前端命令已就绪
    app.emit("commands_ready", ()).unwrap();
}
