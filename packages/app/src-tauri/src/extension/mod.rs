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
pub use types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
    StaticCommandMatch,
};

use crate::shared_types::{Command, CommandAction, CommandKeyword, CommandMatch, ItemSource};

// ============================================================================
// Extension 命令生成
// ============================================================================

/// 将所有 Extension 命令转换为通用 Command 格式
///
/// 用于将 Extension 命令集成到现有的命令搜索系统中
pub fn get_extension_commands() -> Vec<Command> {
    get_all_extensions()
        .iter()
        .flat_map(|ext| {
            let manifest = ext.manifest();
            manifest.commands.iter().map(move |cmd| {
                // 将 StaticCommandMatch 转换为运行时的 CommandMatch
                let matches: Option<Vec<CommandMatch>> = cmd.matches.map(|static_matches| {
                    static_matches
                        .iter()
                        .map(|m| CommandMatch {
                            match_type: m.match_type.to_string(),
                            name: m.name.to_string(),
                            description: m.description.to_string(),
                            regexp: m.regexp.map(|r| r.to_string()),
                            min: m.min,
                            max: m.max,
                            extensions: vec![],
                        })
                        .collect()
                });

                Command {
                    name: format!("extension:{}:{}", manifest.id, cmd.code),
                    title: cmd.name.to_string(),
                    description: Some(cmd.description.to_string()),
                    english_name: cmd.code.to_string(),
                    keywords: cmd
                        .keywords
                        .iter()
                        .map(|k| CommandKeyword {
                            name: k.to_string(),
                            disabled: None,
                            is_default: None,
                        })
                        .collect(),
                    icon: manifest.icon.to_string(),
                    source: ItemSource::Extension,
                    action: CommandAction::Extension {
                        extension_id: manifest.id.to_string(),
                        command_code: cmd.code.to_string(),
                    },
                    origin: None,
                    matches,
                    requires_confirmation: false,
                }
            })
        })
        .collect()
}

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
