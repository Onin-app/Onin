//! Extension 命令生成器
//!
//! 将内置 Extension 转换为可搜索的命令

use crate::extension::get_all_extensions;
use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};

/// 生成 Extension 命令列表
///
/// 将所有内置 Extension 转换为 Command，使其可以在搜索中匹配
pub fn get_initial_extension_commands() -> Vec<Command> {
    get_all_extensions()
        .iter()
        .flat_map(|ext| {
            let manifest = ext.manifest();
            manifest.commands.iter().map(move |cmd| {
                // 构建关键词
                let keywords: Vec<CommandKeyword> = cmd
                    .keywords
                    .iter()
                    .map(|kw| CommandKeyword {
                        name: kw.to_string(),
                        disabled: None,
                        is_default: Some(true),
                    })
                    .collect();

                Command {
                    name: format!("extension:{}:{}", manifest.id, cmd.code),
                    title: manifest.name.to_string(),
                    description: Some(manifest.description.to_string()),
                    english_name: manifest.id.to_string(),
                    keywords,
                    icon: manifest.icon.to_string(),
                    source: ItemSource::Extension,
                    action: CommandAction::Extension {
                        extension_id: manifest.id.to_string(),
                        command_code: cmd.code.to_string(),
                    },
                    origin: None,
                    matches: None,
                }
            })
        })
        .collect()
}
