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

    /// 自定义匹配逻辑（可选覆盖）
    ///
    /// - 返回 `None`：使用声明式 `CommandMatch` 配置（默认行为）
    /// - 返回 `Some(true)`：自定义判断为匹配
    /// - 返回 `Some(false)`：自定义判断为不匹配
    ///
    /// Calculator 等需要复杂语义分析的 Extension 应覆盖此方法。
    /// Translator 等使用标准匹配规则的 Extension 无需覆盖。
    fn custom_matches(&self, _input: &str) -> Option<bool> {
        None
    }

    /// 执行扩展命令
    fn execute(&self, input: &str) -> ExtensionResult;

    /// 执行指定扩展命令
    ///
    /// 默认情况下，内置扩展只有一个命令，因此直接复用 `execute()`。
    fn execute_command(&self, _command_code: &str, input: &str) -> ExtensionResult {
        self.execute(input)
    }

    /// 获取预览结果（用于实时显示）
    fn preview(&self, input: &str) -> Option<super::types::ExtensionPreview>;
}

/// 获取所有已注册的扩展
pub fn get_all_extensions() -> Vec<&'static dyn Extension> {
    vec![
        &extensions::calculator::CALCULATOR_EXTENSION,
        &extensions::emoji::EMOJI_EXTENSION,
        &extensions::clipboard::CLIPBOARD_EXTENSION,
        &extensions::translator::TRANSLATOR_EXTENSION,
    ]
}

/// 根据 ID 获取扩展
pub fn get_extension_by_id(id: &str) -> Option<&'static dyn Extension> {
    get_all_extensions()
        .into_iter()
        .find(|ext| ext.manifest().id == id)
}

/// 查找匹配输入的所有扩展
///
/// 优先使用 Extension 的 `custom_matches` 钩子，
/// 如果未覆盖（返回 None），则不在此处匹配（由前端的 matchCommand.ts 处理）
pub fn find_matching_extensions(input: &str) -> Vec<&'static dyn Extension> {
    get_all_extensions()
        .into_iter()
        .filter(|ext| ext.custom_matches(input).unwrap_or(false))
        .collect()
}
