//! Emoji Extension
//!
//! 提供 emoji 选择器功能：
//! - 按分类浏览所有 emoji
//! - 搜索 emoji 名称
//! - 复制选中的 emoji

pub mod data;

use crate::extension::registry::Extension;
use crate::extension::types::{
    EmojiGridData, EmojiGroup, EmojiItem, ExtensionCommand, ExtensionManifest, ExtensionMatch,
    ExtensionPreview, ExtensionResult, PreviewViewType,
};

// ============================================================================
// Emoji 清单定义
// ============================================================================

/// Emoji 扩展清单
pub static EMOJI_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "emoji",
    name: "Emoji",
    description: "搜索和选择 Emoji 表情",
    icon: "smiley",
    commands: &[ExtensionCommand {
        code: "search",
        name: "搜索 Emoji",
        description: "浏览和搜索 Emoji 表情",
        keywords: &["emoji", "表情", "😀", "smiley", "emoticon"],
        matches: None, // 移除正则匹配，使用关键字匹配
    }],
};

// ============================================================================
// Emoji Extension 实现
// ============================================================================

/// Emoji 扩展实例
pub struct EmojiExtension;

/// 全局静态实例
pub static EMOJI_EXTENSION: EmojiExtension = EmojiExtension;

impl Extension for EmojiExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &EMOJI_MANIFEST
    }

    fn matches(&self, _input: &str) -> bool {
        // Emoji 现在作为命令出现在列表中，不再需要 preview 匹配
        // Preview 只用于 calculator 这类需要即时显示结果的 extension
        false
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        // 对于 emoji 扩展，execute 返回选中的 emoji
        // 这里返回空结果，实际选择逻辑在前端处理
        ExtensionResult {
            success: true,
            value: None,
            result_type: crate::extension::types::ExtensionResultType::Conversion,
            copyable: None,
            subtitle: None,
            error: None,
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let trimmed = input.trim().to_lowercase();

        // 检查是否匹配 emoji 关键字
        if !self.matches(input) {
            return None;
        }

        // 提取搜索关键词（emoji 后面的部分）
        let search_query = if trimmed.starts_with("emoji ") {
            trimmed.strip_prefix("emoji ").unwrap_or("").trim()
        } else {
            ""
        };

        // 获取 emoji 数据
        let groups = if search_query.is_empty() {
            // 无搜索词，返回所有分类
            data::get_all_groups()
        } else {
            // 有搜索词，返回过滤后的结果
            data::search_emojis(search_query)
        };

        // 如果没有结果，返回 None
        if groups.is_empty() || groups.iter().all(|g| g.emojis.is_empty()) {
            return None;
        }

        Some(ExtensionPreview {
            extension_id: "emoji".to_string(),
            command_code: "search".to_string(),
            title: "Emoji".to_string(),
            description: "选择 emoji · 回车进入".to_string(),
            icon: "smiley".to_string(),
            copyable: String::new(),
            view_type: PreviewViewType::Grid,
            grid_data: Some(EmojiGridData { groups }),
        })
    }
}
