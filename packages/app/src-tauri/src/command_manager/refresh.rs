//! 刷新逻辑模块
//!
//! 处理命令刷新和防止并发刷新

use crate::shared_types::Command;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

use super::storage;

/// 全局标志：防止并发刷新
static IS_REFRESHING: AtomicBool = AtomicBool::new(false);

/// 刷新结果
#[derive(Clone, serde::Serialize)]
pub struct RefreshResult {
    pub previous_count: usize,
    pub current_count: usize,
    pub added: i32,
}

/// 执行刷新操作
///
/// 返回 `None` 表示已有刷新操作进行中
pub async fn do_refresh(app: &AppHandle) -> Option<RefreshResult> {
    // 使用 compare_exchange 防止并发刷新
    if IS_REFRESHING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return None; // 已在刷新中，跳过
    }

    // 发送刷新开始事件
    let _ = app.emit("refresh_started", ());

    // 获取刷新前的命令数量
    let path = storage::get_commands_file_path(app);
    let previous_count = if path.exists() {
        match fs::read_to_string(&path) {
            Ok(json_str) => serde_json::from_str::<Vec<Command>>(&json_str)
                .map(|cmds| cmds.len())
                .unwrap_or(0),
            Err(_) => 0,
        }
    } else {
        0
    };

    // 生成并保存新命令
    let commands = storage::generate_and_save_commands(app).await;
    let current_count = commands.len();
    let added = current_count as i32 - previous_count as i32;

    let result = RefreshResult {
        previous_count,
        current_count,
        added,
    };

    // 发送刷新完成事件
    let _ = app.emit("commands_refreshed", result.clone());

    // 重置标志
    IS_REFRESHING.store(false, Ordering::SeqCst);

    Some(result)
}
