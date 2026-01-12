//! 插件命令生成器

use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use tauri::{AppHandle, Manager};

use super::sanitize_command_name_part;

/// 生成插件命令列表
pub fn get_initial_plugin_commands(app: &AppHandle) -> Vec<Command> {
    let plugin_store: tauri::State<crate::plugin::PluginStore> = app.state();

    match crate::plugin::loader::get_loaded_plugins(plugin_store) {
        Ok(plugins) => {
            let mut commands = Vec::new();

            for plugin in plugins {
                // 跳过禁用的插件
                if !plugin.enabled {
                    continue;
                }

                let safe_plugin_id = sanitize_command_name_part(&plugin.manifest.id);

                // 1. 为插件本身创建一个 Command（用于打开插件）
                #[allow(deprecated)]
                commands.push(Command {
                    name: format!("plugin_{}", safe_plugin_id),
                    title: plugin.manifest.name.clone(),
                    description: Some(plugin.manifest.description.clone()),
                    english_name: plugin.manifest.name.clone(),
                    keywords: vec![CommandKeyword {
                        name: plugin.manifest.name.clone(),
                        disabled: None,
                        is_default: Some(true),
                    }],
                    icon: "".to_string(),
                    source: ItemSource::Plugin,
                    action: CommandAction::Plugin(plugin.manifest.id.clone()),
                    origin: None,
                    matches: None,
                });

                // 2. 为每个插件的功能指令创建 Command
                for cmd in &plugin.manifest.commands {
                    let safe_cmd_code = sanitize_command_name_part(&cmd.code);
                    let keywords: Vec<CommandKeyword> = cmd
                        .keywords
                        .iter()
                        .map(|kw| CommandKeyword {
                            name: kw.name.clone(),
                            disabled: None,
                            is_default: Some(true),
                        })
                        .collect();

                    // 转换插件匹配规则
                    let matches = if !cmd.matches.is_empty() {
                        Some(
                            cmd.matches
                                .iter()
                                .map(|m| crate::shared_types::CommandMatch {
                                    match_type: m.match_type.clone(),
                                    name: m.name.clone(),
                                    description: m.description.clone(),
                                    regexp: m.regexp.clone(),
                                    min: m.min,
                                    max: m.max,
                                    extensions: m.extensions.clone(),
                                })
                                .collect(),
                        )
                    } else {
                        None
                    };

                    commands.push(Command {
                        name: format!("plugin_cmd_{}_{}", safe_plugin_id, safe_cmd_code),
                        title: cmd.name.clone(),
                        description: Some(cmd.description.clone()),
                        english_name: cmd.name.clone(),
                        keywords,
                        icon: "icon-plugin".to_string(),
                        source: ItemSource::Plugin,
                        action: CommandAction::PluginCommand {
                            plugin_id: plugin.manifest.id.clone(),
                            command_code: cmd.code.clone(),
                        },
                        origin: None,
                        matches,
                    });
                }
            }

            commands
        }
        Err(e) => {
            eprintln!("Failed to load plugins as commands: {}", e);
            vec![]
        }
    }
}

/// 获取插件中定义的指令
pub fn get_plugin_commands(
    app: &AppHandle,
) -> Vec<(String, Vec<crate::plugin::PluginCommandManifest>)> {
    let plugin_store: tauri::State<crate::plugin::PluginStore> = app.state();

    match crate::plugin::get_loaded_plugins(plugin_store) {
        Ok(plugins) => plugins
            .into_iter()
            .filter(|plugin| plugin.enabled && !plugin.manifest.commands.is_empty())
            .map(|plugin| (plugin.manifest.name, plugin.manifest.commands))
            .collect(),
        Err(e) => {
            eprintln!("Failed to load plugin commands: {}", e);
            vec![]
        }
    }
}

/// 获取插件ID到名称的映射
pub fn get_plugin_id_name_mapping(app: &AppHandle) -> Vec<(String, String)> {
    let plugin_store: tauri::State<crate::plugin::PluginStore> = app.state();

    match crate::plugin::loader::get_loaded_plugins(plugin_store) {
        Ok(plugins) => plugins
            .into_iter()
            .filter(|plugin| plugin.enabled && !plugin.manifest.commands.is_empty())
            .map(|plugin| (plugin.manifest.name.clone(), plugin.manifest.id.clone()))
            .collect(),
        Err(e) => {
            eprintln!("Failed to load plugin id mapping: {}", e);
            vec![]
        }
    }
}

/// 生成动态命令列表
///
/// 从 dynamic_commands.json 加载插件动态注册的命令
pub fn get_initial_dynamic_commands(app: &AppHandle) -> Vec<Command> {
    let dynamic_commands = crate::command_manager::dynamic_commands::get_all_dynamic_commands(app);

    dynamic_commands
        .into_iter()
        .map(|dc| {
            let safe_plugin_id = sanitize_command_name_part(&dc.plugin_id);
            let safe_code = sanitize_command_name_part(&dc.code);

            Command {
                name: format!("dynamic_{}_{}", safe_plugin_id, safe_code),
                title: dc.name.clone(),
                description: dc.description,
                english_name: dc.name,
                keywords: dc.keywords,
                icon: "icon-plugin".to_string(),
                source: ItemSource::Plugin,
                action: CommandAction::PluginCommand {
                    plugin_id: dc.plugin_id,
                    command_code: dc.code,
                },
                origin: None,
                matches: dc.matches,
            }
        })
        .collect()
}
