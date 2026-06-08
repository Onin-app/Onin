//! 内部页面命令生成器

use crate::internal_commands;
use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};

/// 生成内部页面命令列表
pub fn get_initial_internal_commands() -> Vec<Command> {
    internal_commands::INTERNAL_COMMANDS
        .iter()
        .map(|cmd_info| Command {
            name: cmd_info.name.to_string(),
            title: cmd_info.title.to_string(),
            description: Some(cmd_info.description.to_string()),
            keywords: build_internal_keywords(cmd_info),
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Internal,
            action: CommandAction::Internal(cmd_info.name.to_string()),
            origin: None,
            matches: None,
            requires_confirmation: cmd_info.requires_confirmation,
        })
        .collect()
}

/// 构建内部命令关键词
fn build_internal_keywords(
    cmd_info: &internal_commands::InternalCommandInfo,
) -> Vec<CommandKeyword> {
    let kws: Vec<_> = cmd_info
        .keywords
        .iter()
        .map(|&alias| CommandKeyword {
            name: alias.to_string(),
            disabled: None,
            is_default: Some(true),
        })
        .collect();

    kws
}
