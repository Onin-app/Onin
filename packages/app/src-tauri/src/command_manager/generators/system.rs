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
            description: None,
            english_name: cmd_info.english_name.to_string(),
            keywords: build_system_keywords(cmd_info),
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Command,
            action: CommandAction::System(cmd_info.name.to_string()),
            origin: None,
            matches: None,
        })
        .collect()
}

/// 构建系统命令关键词
fn build_system_keywords(cmd_info: &system_commands::SystemCommandInfo) -> Vec<CommandKeyword> {
    let mut kws: Vec<_> = cmd_info
        .keywords
        .iter()
        .map(|&alias| CommandKeyword {
            name: alias.to_string(),
            disabled: None,
            is_default: Some(true),
        })
        .collect();

    // 确保 title 和 english_name 也在关键词列表中
    if !kws.iter().any(|kw| kw.name == cmd_info.title) {
        kws.push(CommandKeyword {
            name: cmd_info.title.to_string(),
            disabled: None,
            is_default: Some(true),
        });
    }
    if !kws.iter().any(|kw| kw.name == cmd_info.english_name) {
        kws.push(CommandKeyword {
            name: cmd_info.english_name.to_string(),
            disabled: None,
            is_default: Some(true),
        });
    }

    kws
}
