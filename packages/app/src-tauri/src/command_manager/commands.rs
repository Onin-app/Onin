//! Tauri 命令模块
//!
//! 暴露给前端的命令接口

use crate::shared_types::Command;
use tauri::AppHandle;

use super::{generators, refresh, storage};

/// 获取所有命令
#[tauri::command]
pub async fn get_commands(app: AppHandle) -> Vec<Command> {
    storage::load_commands(&app).await
}

/// 更新单个命令
#[tauri::command]
pub async fn update_command(app: AppHandle, command_to_update: Command) {
    let mut commands = storage::load_commands(&app).await;
    if let Some(command) = commands
        .iter_mut()
        .find(|cmd| cmd.name == command_to_update.name)
    {
        *command = command_to_update;
        storage::save_commands(&app, &commands);
    }
}

/// 刷新命令列表
#[tauri::command]
pub async fn refresh_commands(app: AppHandle) {
    refresh::do_refresh(&app).await;
}

/// 获取插件命令列表
#[tauri::command]
pub async fn get_plugin_commands_list(
    app: AppHandle,
) -> Vec<(String, Vec<crate::plugin::PluginCommandManifest>)> {
    generators::get_plugin_commands(&app)
}

/// 获取插件ID映射
#[tauri::command]
pub async fn get_plugin_id_mapping(app: AppHandle) -> Vec<(String, String)> {
    generators::get_plugin_id_name_mapping(&app)
}
