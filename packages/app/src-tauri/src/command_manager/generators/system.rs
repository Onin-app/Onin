//! 系统命令生成器

use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use crate::system_commands;

/// 生成系统命令列表
pub fn get_initial_system_commands() -> Vec<Command> {
    system_commands::SYSTEM_COMMANDS
        .iter()
        .map(|cmd_info| Command {
            name: cmd_info.name.to_string(),
            title: cmd_info.title.to_string(),
            description: Some(cmd_info.description.to_string()),
            keywords: build_system_keywords(cmd_info),
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Command,
            action: CommandAction::System(cmd_info.name.to_string()),
            origin: None,
            matches: None,
            requires_confirmation: cmd_info.requires_confirmation,
            command_type: crate::shared_types::CommandType::Function,
        })
        .collect()
}

/// 构建系统命令关键词
fn build_system_keywords(cmd_info: &system_commands::SystemCommandInfo) -> Vec<CommandKeyword> {
    let kws: Vec<_> = cmd_info
        .keywords
        .iter()
        .map(|&alias| CommandKeyword {
            name: alias.to_string(),
            disabled: None,
            is_default: Some(true),
            // TODO: make this configurable if system command supports keyword type in the future
            keyword_type: None,
        })
        .collect();

    kws
}
