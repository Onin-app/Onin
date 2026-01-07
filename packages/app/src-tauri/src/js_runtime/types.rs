//! 类型定义模块
//!
//! 包含 JS 运行时所需的核心类型定义

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// 插件调用结果类型
///
/// 用于在 JS 和 Rust 之间传递调用结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InvokeResult {
    /// 成功结果
    #[serde(rename = "ok")]
    Ok { value: serde_json::Value },
    /// 错误结果
    #[serde(rename = "error")]
    Err { error: String },
}

/// 插件任务类型
///
/// 用于在主线程和 JS 运行时线程之间传递任务
#[derive(Debug)]
pub enum PluginTask {
    /// 初始化插件
    InitPlugin {
        plugin_id: String,
        js_code: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// 执行命令
    ExecuteCommand {
        plugin_id: String,
        command: String,
        args: serde_json::Value,
        response: oneshot::Sender<Result<serde_json::Value, String>>,
    },
    /// 清除单个插件
    ClearPlugin {
        plugin_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// 清除所有插件
    ClearAllPlugins {
        response: oneshot::Sender<Result<(), String>>,
    },
}
