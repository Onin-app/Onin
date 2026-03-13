//! 插件命令生成器

use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::path::Path;
use tauri::{AppHandle, Manager};

use super::sanitize_command_name_part;

/// 获取插件图标
///
/// 支持三种图标格式：
/// 1. HTTP/HTTPS URL（市场插件） - 直接返回 URL
/// 2. Data URL（已编码的图标） - 直接返回
/// 3. 本地文件路径（本地插件） - 读取并编码为 Data URL
fn get_plugin_icon_base64(app: &AppHandle, dir_name: &str, icon_path: &Option<String>) -> String {
    // 如果没有配置图标，返回空字符串
    let icon_value = match icon_path {
        Some(path) if !path.is_empty() => path,
        _ => {
            return String::new();
        }
    };

    // 如果图标已经是 URL 或 Data URL，直接返回
    if icon_value.starts_with("http://")
        || icon_value.starts_with("https://")
        || icon_value.starts_with("data:")
    {
        return icon_value.clone();
    }

    // 本地文件：获取插件目录
    let plugins_dir = match app.path().app_data_dir() {
        Ok(dir) => dir.join("plugins"),
        Err(e) => {
            eprintln!("[plugin/icon] 获取 app_data_dir 失败: {}", e);
            return String::new();
        }
    };

    let icon_full_path = plugins_dir.join(dir_name).join(icon_value);

    // 根据扩展名确定 MIME 类型
    let mime_type = match Path::new(icon_value)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png", // 默认使用 PNG
    };

    // 读取图标文件并编码为 Data URL
    match std::fs::read(&icon_full_path) {
        Ok(bytes) => {
            format!(
                "data:{};base64,{}",
                mime_type,
                BASE64_STANDARD.encode(&bytes)
            )
        }
        Err(e) => {
            eprintln!(
                "[plugin/icon] 读取插件图标失败: {} - {} (路径: {})",
                dir_name,
                e,
                icon_full_path.display()
            );
            String::new()
        }
    }
}

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

                // 获取插件图标的 Base64 编码
                let icon_base64 =
                    get_plugin_icon_base64(app, &plugin.dir_name, &plugin.manifest.icon);

                // 1. 为插件本身创建一个 Command（用于打开插件）
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
                    icon: icon_base64.clone(),
                    source: ItemSource::Plugin,
                    action: CommandAction::PluginEntry {
                        plugin_id: plugin.manifest.id.clone(),
                    },
                    origin: None,
                    matches: None,
                    requires_confirmation: false,
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
                        icon: icon_base64.clone(),
                        source: ItemSource::Plugin,
                        action: CommandAction::PluginCommand {
                            plugin_id: plugin.manifest.id.clone(),
                            command_code: cmd.code.clone(),
                        },
                        origin: None,
                        matches,
                        requires_confirmation: false,
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
                requires_confirmation: false,
            }
        })
        .collect()
}

