//! Extension 命令生成器
//!
//! 将内置 Extension 转换为可搜索的命令

use crate::extension::get_all_extensions;
use crate::shared_types::{Command, CommandAction, CommandKeyword, CommandMatch, ItemSource};

/// 生成 Extension 命令列表
///
/// 将所有内置 Extension 转换为 Command，使其可以在搜索中匹配。
/// StaticCommandMatch 会被转换为运行时的 CommandMatch，
/// 使前端 matchCommand.ts 能统一处理 Extension 和 Plugin 的匹配指令。
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

                // 将 StaticCommandMatch 转换为 CommandMatch
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
                    matches,
                    requires_confirmation: false,
                }
            })
        })
        .collect()
}
