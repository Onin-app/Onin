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
use tauri::AppHandle;
pub use types::{ExtensionPreview, ExtensionResult};

// ============================================================================
// Extension 命令生成
// ============================================================================

/// 尝试获取输入的实时预览
///
/// 如果输入匹配任何 Extension（通过 custom_matches），返回预览结果
pub fn get_preview_for_input(app: &AppHandle, input: &str) -> Option<ExtensionPreview> {
    if input.is_empty() {
        return None;
    }

    for ext in find_matching_extensions(input) {
        if !crate::app_config::is_extension_enabled(app, ext.manifest().id) {
            continue;
        }

        if let Some(preview) = ext.preview(input) {
            return Some(preview);
        }
    }

    None
}

/// 执行 Extension 命令
pub fn execute_extension_command(
    app: &AppHandle,
    extension_id: &str,
    command_code: &str,
    input: &str,
) -> ExtensionResult {
    if !crate::app_config::is_extension_enabled(app, extension_id) {
        return ExtensionResult::error(format!("Extension disabled: {}", extension_id));
    }

    match get_extension_by_id(extension_id) {
        Some(ext) => ext.execute_command(command_code, input),
        None => ExtensionResult::error(format!("Extension not found: {}", extension_id)),
    }
}

pub fn get_extension_infos(app: &AppHandle) -> Vec<types::ExtensionInfo> {
    get_all_extensions()
        .into_iter()
        .map(|ext| {
            let manifest = ext.manifest();
            let commands = manifest
                .commands
                .iter()
                .map(|command| types::ExtensionCommandInfo {
                    code: command.code.to_string(),
                    name: command.name.to_string(),
                    description: command.description.map(str::to_string),
                    icon: command.icon.unwrap_or(manifest.icon).to_string(),
                    keywords: command
                        .keywords
                        .iter()
                        .map(|keyword| keyword.to_string())
                        .collect(),
                    has_matches: command.matches.is_some_and(|matches| !matches.is_empty()),
                })
                .collect();

            types::ExtensionInfo {
                id: manifest.id.to_string(),
                name: manifest.name.to_string(),
                description: manifest.description.to_string(),
                icon: manifest.icon.to_string(),
                enabled: crate::app_config::is_extension_enabled(app, manifest.id),
                commands,
            }
        })
        .collect()
}
