//! Bookmarks Extension
//!
//! 提供浏览器书签检索与打开功能：
//! - 扫描 Chrome, Safari, Edge, Brave, Arc 的书签
//! - 提供专用的二级页面进行模糊搜索与过滤
//! - 键盘回车即刻调用默认浏览器打开网址

pub mod parser;

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult,
};

// ============================================================================
// Bookmarks 清单定义
// ============================================================================

/// Bookmarks 扩展清单
pub static BOOKMARKS_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "bookmarks",
    name: "浏览器书签",
    description: "检索并快速打开浏览器收藏的书签",
    icon: "bookmark",
    commands: &[ExtensionCommand {
        code: "search",
        name: "搜索书签",
        description: Some("进入本地浏览器书签搜索"),
        icon: Some("bookmark"),
        keywords: &["bookmark", "书签"],
        matches: None, // 仅通过关键词触发，不参与主输入预览匹配
    }],
};

// ============================================================================
// Bookmarks Extension 实现
// ============================================================================

/// Bookmarks 扩展实例
pub struct BookmarksExtension;

/// 全局静态实例
pub static BOOKMARKS_EXTENSION: BookmarksExtension = BookmarksExtension;

impl Extension for BookmarksExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &BOOKMARKS_MANIFEST
    }

    fn execute(&self, _input: &str) -> ExtensionResult {
        // 对于书签扩展，实际选择逻辑在前端二级页面处理
        ExtensionResult {
            success: true,
            value: None,
            result_type: crate::extension::types::ExtensionResultType::Conversion,
            copyable: None,
            subtitle: None,
            error: None,
        }
    }

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}
