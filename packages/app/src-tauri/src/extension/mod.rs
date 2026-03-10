//! # Extension 管理器模块
//!
//! Extension 系统的核心模块，提供：
//! - Extension 注册和管理
//! - 输入匹配和执行
//! - 实时预览支持

pub mod api;
pub mod registry;
pub mod types;

pub use registry::{find_matching_extensions, get_all_extensions, get_extension_by_id, Extension};
pub use types::{ExtensionPreview, ExtensionResult};

// ============================================================================
// Extension 命令生成
// ============================================================================

/// 尝试获取输入的实时预览
///
/// 如果输入匹配任何 Extension（通过 custom_matches），返回预览结果
pub fn get_preview_for_input(input: &str) -> Option<ExtensionPreview> {
    if input.is_empty() {
        return None;
    }

    for ext in find_matching_extensions(input) {
        if let Some(preview) = ext.preview(input) {
            return Some(preview);
        }
    }

    None
}

/// 执行 Extension 命令
pub fn execute_extension_command(extension_id: &str, input: &str) -> ExtensionResult {
    match get_extension_by_id(extension_id) {
        Some(ext) => ext.execute(input),
        None => ExtensionResult::error(format!("Extension not found: {}", extension_id)),
    }
}
