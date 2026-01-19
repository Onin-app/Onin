//! # Extension API 模块
//!
//! 提供 Extension 相关的 Tauri 命令

use crate::extension::{self, ExtensionPreview};
use tauri::command;

/// 获取输入的实时预览结果
///
/// 前端在用户输入时调用此命令，获取 Extension 的预览结果
#[command]
pub fn get_extension_preview(input: String) -> Option<ExtensionPreview> {
    extension::get_preview_for_input(&input)
}

/// 执行 Extension 命令并复制结果
#[command]
pub fn execute_extension(extension_id: String, input: String) -> extension::ExtensionResult {
    extension::execute_extension_command(&extension_id, &input)
}
