//! # Extension API 模块
//!
//! 提供 Extension 相关的 Tauri 命令

use crate::extension::types::EmojiGridData;
use crate::extension::{self, ExtensionPreview};
use tauri::{command, AppHandle, Emitter, Manager};

/// 获取输入的实时预览结果
///
/// 前端在用户输入时调用此命令，获取 Extension 的预览结果
#[command]
pub fn get_extension_preview(app: AppHandle, input: String) -> Option<ExtensionPreview> {
    extension::get_preview_for_input(&app, &input)
}

/// 执行 Extension 命令并复制结果
#[command]
pub fn execute_extension(
    app: AppHandle,
    extension_id: String,
    command_code: String,
    input: String,
) -> extension::ExtensionResult {
    extension::execute_extension_command(&app, &extension_id, &command_code, &input)
}

#[command]
pub fn get_extensions(app: AppHandle) -> Vec<extension::types::ExtensionInfo> {
    extension::get_extension_infos(&app)
}

#[command]
pub async fn toggle_extension(
    app: AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<(), String> {
    if extension::get_extension_by_id(&extension_id).is_none() {
        return Err(format!("Extension not found: {}", extension_id));
    }

    let config_state = app.state::<crate::app_config::AppConfigState>();
    {
        let mut config = config_state.0.lock().map_err(|e| e.to_string())?;
        let is_disabled = config
            .disabled_extension_ids
            .iter()
            .any(|id| id == &extension_id);

        if enabled && is_disabled {
            config
                .disabled_extension_ids
                .retain(|id| id != &extension_id);
        } else if !enabled && !is_disabled {
            config.disabled_extension_ids.push(extension_id.clone());
        }

        crate::app_config::save_config(&app, &config)?;
    }

    crate::command_manager::commands::refresh_commands(app.clone()).await;
    let _ = app.emit("extensions_changed", ());

    Ok(())
}

/// 获取 Emoji 数据
///
/// Emoji 页面专用 API，直接获取 emoji 数据而不经过 preview 机制
#[command]
pub fn get_emoji_data(search_query: String) -> Option<EmojiGridData> {
    use crate::extensions::emoji::data;

    let groups = if search_query.is_empty() {
        data::get_all_groups()
    } else {
        data::search_emojis(&search_query)
    };

    if groups.is_empty() || groups.iter().all(|g| g.emojis.is_empty()) {
        return None;
    }

    Some(EmojiGridData { groups })
}
