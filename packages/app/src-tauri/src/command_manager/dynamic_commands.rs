//! 动态命令存储模块
//!
//! 管理插件通过 SDK 动态注册的命令

use crate::shared_types::{CommandKeyword, CommandMatch, DynamicCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Webview};

/// 动态命令存储结构
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DynamicCommandsStore {
    /// 插件ID -> 命令列表
    pub commands: HashMap<String, Vec<DynamicCommand>>,
}

/// SDK 传入的命令定义（不含 plugin_id 和 created_at）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDefinition {
    pub code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Option<Vec<CommandKeywordInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<CommandMatchInput>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandKeywordInput {
    pub name: String,
    #[serde(rename = "type")]
    pub keyword_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandMatchInput {
    #[serde(rename = "type")]
    pub match_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(default)]
    pub extensions: Option<Vec<String>>,
}

/// 获取动态命令存储文件路径
pub fn get_dynamic_commands_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("dynamic_commands.json")
}

/// 加载动态命令
pub fn load_dynamic_commands(app: &AppHandle) -> DynamicCommandsStore {
    let path = get_dynamic_commands_file_path(app);
    if !path.exists() {
        return DynamicCommandsStore::default();
    }

    match fs::read_to_string(&path) {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
        Err(_) => DynamicCommandsStore::default(),
    }
}

/// 保存动态命令
pub fn save_dynamic_commands(app: &AppHandle, store: &DynamicCommandsStore) {
    let path = get_dynamic_commands_file_path(app);
    let json = serde_json::to_string_pretty(store).unwrap();
    fs::write(path, json).unwrap();
}

/// 获取当前时间戳（毫秒）
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// 注册动态命令
#[tauri::command]
pub async fn register_dynamic_command(
    app: AppHandle,
    window: Webview,
    command: CommandDefinition,
) -> Result<(), String> {
    // 从 webview 获取 plugin_id（通过 URL 参数或窗口标签）
    // 这里我们从窗口标签解析 plugin_id
    let plugin_id = crate::plugin::context::get_current_plugin_id(&app, Some(&window))?;

    let mut store = load_dynamic_commands(&app);

    // 转换输入类型为内部类型
    let keywords: Vec<CommandKeyword> = command
        .keywords
        .unwrap_or_default()
        .into_iter()
        .map(|k| CommandKeyword {
            name: k.name,
            disabled: None,
            is_default: None,
        })
        .collect();

    let matches: Option<Vec<CommandMatch>> = command.matches.map(|m| {
        m.into_iter()
            .map(|m| CommandMatch {
                match_type: m.match_type,
                name: m.name,
                description: m.description.unwrap_or_default(),
                regexp: m.regexp,
                min: m.min,
                max: m.max,
                extensions: m.extensions.unwrap_or_default(),
            })
            .collect()
    });

    let dynamic_command = DynamicCommand {
        code: command.code.clone(),
        name: command.name,
        description: command.description,
        keywords,
        matches,
        plugin_id: plugin_id.clone(),
        created_at: get_current_timestamp(),
    };

    // 获取或创建该插件的命令列表
    let plugin_commands = store.commands.entry(plugin_id.clone()).or_default();

    // 检查是否已存在相同 code 的命令，如果存在则更新
    if let Some(existing) = plugin_commands
        .iter_mut()
        .find(|c| c.code == command.code)
    {
        *existing = dynamic_command;
    } else {
        plugin_commands.push(dynamic_command);
    }

    save_dynamic_commands(&app, &store);

    // 触发命令刷新
    super::refresh::do_refresh(&app).await;

    Ok(())
}

/// 移除动态命令
#[tauri::command]
pub async fn remove_dynamic_command(
    app: AppHandle,
    window: Webview,
    command_code: String,
) -> Result<(), String> {
    let plugin_id = crate::plugin::context::get_current_plugin_id(&app, Some(&window))?;

    let mut store = load_dynamic_commands(&app);

    if let Some(plugin_commands) = store.commands.get_mut(&plugin_id) {
        let original_len = plugin_commands.len();
        plugin_commands.retain(|c| c.code != command_code);

        if plugin_commands.len() == original_len {
            return Err(format!(
                "Command '{}' not found for plugin '{}'",
                command_code, plugin_id
            ));
        }

        // 如果插件没有命令了，移除该条目
        if plugin_commands.is_empty() {
            store.commands.remove(&plugin_id);
        }

        save_dynamic_commands(&app, &store);

        // 触发命令刷新
        super::refresh::do_refresh(&app).await;

        Ok(())
    } else {
        Err(format!("No commands found for plugin '{}'", plugin_id))
    }
}

/// 获取所有动态命令（用于合并到命令列表）
pub fn get_all_dynamic_commands(app: &AppHandle) -> Vec<DynamicCommand> {
    let store = load_dynamic_commands(app);
    store
        .commands
        .into_values()
        .flatten()
        .collect()
}

// get_plugin_id_from_context has been moved to crate::plugin::context::get_current_plugin_id


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_definition_deserialization() {
        let json = r#"{
            "code": "test-cmd",
            "name": "Test Command",
            "description": "A test",
            "keywords": [{"name": "test"}],
            "matches": [{"type": "text", "name": "Text", "min": 1}]
        }"#;

        let def: CommandDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.code, "test-cmd");
        assert_eq!(def.name, "Test Command");
        assert!(def.keywords.is_some());
        assert!(def.matches.is_some());
    }

    #[test]
    fn test_dynamic_commands_store_default() {
        let store = DynamicCommandsStore::default();
        assert!(store.commands.is_empty());
    }
}
