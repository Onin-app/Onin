//! # Extension 注册表模块
//!
//! 管理所有内置扩展的注册和查找

use super::types::{ExtensionManifest, ExtensionResult};
use crate::extensions;

// ============================================================================
// Extension 注册表
// ============================================================================

/// Extension trait - 所有扩展必须实现此 trait
pub trait Extension: Send + Sync {
    /// 获取扩展清单
    fn manifest(&self) -> &'static ExtensionManifest;

    /// 检查输入是否匹配此扩展
    fn matches(&self, input: &str) -> bool;

    /// 执行扩展命令
    fn execute(&self, input: &str) -> ExtensionResult;

    /// 获取预览结果（用于实时显示）
    fn preview(&self, input: &str) -> Option<super::types::ExtensionPreview>;
}

/// 获取所有已注册的扩展
pub fn get_all_extensions() -> Vec<&'static dyn Extension> {
    vec![
        &extensions::calculator::CALCULATOR_EXTENSION,
        &extensions::emoji::EMOJI_EXTENSION,
    ]
}

/// 根据 ID 获取扩展
pub fn get_extension_by_id(id: &str) -> Option<&'static dyn Extension> {
    get_all_extensions()
        .into_iter()
        .find(|ext| ext.manifest().id == id)
}

/// 查找匹配输入的所有扩展
pub fn find_matching_extensions(input: &str) -> Vec<&'static dyn Extension> {
    get_all_extensions()
        .into_iter()
        .filter(|ext| ext.matches(input))
        .collect()
}
