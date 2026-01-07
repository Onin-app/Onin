//! 文件命令生成器

use crate::file_command_manager;
use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use tauri::{AppHandle, Manager};

/// 生成文件命令列表
pub async fn get_initial_file_commands(app: &AppHandle) -> Vec<Command> {
    let file_manager = app.state::<file_command_manager::FileCommandManager>();
    let file_items = file_manager.get_items().await;

    file_items
        .into_iter()
        .map(|item| Command {
            name: format!("file_{}", item.path),
            title: item.name.clone(),
            english_name: item.name.clone(),
            keywords: vec![CommandKeyword {
                name: item.name,
                disabled: None,
                is_default: Some(true),
            }],
            icon: item.icon,
            source: ItemSource::FileCommand,
            action: CommandAction::File(item.path),
            origin: None,
            matches: None,
        })
        .collect()
}
