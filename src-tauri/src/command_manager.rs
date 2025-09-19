use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use crate::{file_command_manager, installed_apps, plugin_manager, system_commands};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

fn get_commands_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("commands.json")
}

async fn generate_and_save_commands(app: &AppHandle) -> Vec<Command> {
    let mut initial_commands = get_initial_system_commands();
    let app_commands = get_initial_app_commands().await;
    initial_commands.extend(app_commands);
    let file_commands = get_initial_file_commands(app).await;
    initial_commands.extend(file_commands);
    let plugin_commands = get_initial_plugin_commands(app);
    initial_commands.extend(plugin_commands);
    save_commands(app, &initial_commands);
    initial_commands
}

pub async fn init(app: &AppHandle) {
    load_commands(app).await;
    // Emit an event to notify the frontend that commands are ready
    app.emit("commands_ready", ()).unwrap();
}

pub async fn load_commands(app: &AppHandle) -> Vec<Command> {
    let path = get_commands_file_path(app);
    if !path.exists() {
        return generate_and_save_commands(app).await;
    }

    match fs::read_to_string(&path) {
        Ok(json_str) => {
            let result: Result<Vec<Command>, serde_json::Error> = serde_json::from_str(&json_str);
            match result {
                Ok(commands) => {
                    let installed_plugins = get_initial_plugin_commands(app);
                    let installed_plugins_map: std::collections::HashMap<_, _> = installed_plugins
                        .into_iter()
                        .map(|p| (p.name.clone(), p))
                        .collect();

                    let final_commands: Vec<Command> = commands
                        .iter()
                        .filter(|c| c.source != ItemSource::Plugin)
                        .cloned()
                        .collect();

                    let mut final_plugins: Vec<Command> = Vec::new();
                    for (name, plugin_command) in installed_plugins_map {
                        // Find if this plugin command already exists in the saved commands
                        let existing_command = commands
                            .iter()
                            .find(|c| c.source == ItemSource::Plugin && c.name == name);

                        if let Some(existing) = existing_command {
                            // If it exists, keep the saved version (with user changes)
                            final_plugins.push(existing.clone());
                        } else {
                            // If it's a new plugin, add it
                            final_plugins.push(plugin_command);
                        }
                    }

                    let mut mutable_final_commands = final_commands;
                    mutable_final_commands.extend(final_plugins);
                    save_commands(app, &mutable_final_commands);
                    mutable_final_commands
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse commands.json: {}. Deleting and regenerating.",
                        e
                    );
                    if let Err(err) = fs::remove_file(&path) {
                        eprintln!("Failed to delete corrupted commands.json: {}", err);
                    }
                    generate_and_save_commands(app).await
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read commands.json: {}. Regenerating.", e);
            generate_and_save_commands(app).await
        }
    }
}

fn save_commands(app: &AppHandle, commands: &[Command]) {
    let path = get_commands_file_path(app);
    let json = serde_json::to_string_pretty(commands).unwrap();
    fs::write(path, json).unwrap();
}

#[tauri::command]
pub async fn get_commands(app: AppHandle) -> Vec<Command> {
    load_commands(&app).await
}

#[tauri::command]
pub async fn update_command(app: AppHandle, command_to_update: Command) {
    let mut commands = load_commands(&app).await;
    if let Some(command) = commands
        .iter_mut()
        .find(|cmd| cmd.name == command_to_update.name)
    {
        *command = command_to_update;
        save_commands(&app, &commands);
    }
}

#[tauri::command]
pub async fn refresh_commands(app: AppHandle) {
    let _commands = generate_and_save_commands(&app).await;
    // Emit an event to notify the frontend that commands have been refreshed
    app.emit("commands_refreshed", ()).unwrap();
}

fn get_initial_system_commands() -> Vec<Command> {
    system_commands::SYSTEM_COMMANDS
        .iter()
        .map(|cmd_info| Command {
            name: cmd_info.name.to_string(),
            title: cmd_info.title.to_string(),
            english_name: cmd_info.english_name.to_string(),
            keywords: {
                let mut kws: Vec<_> = cmd_info
                    .keywords
                    .iter()
                    .map(|&alias| CommandKeyword {
                        name: alias.to_string(),
                        disabled: None,
                        is_default: Some(true),
                    })
                    .collect();
                // Ensure title and english_name are also part of keywords so they can be disabled.
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
            },
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Command,
            action: CommandAction::System(cmd_info.name.to_string()),
            origin: None,
        })
        .collect()
}

async fn get_initial_app_commands() -> Vec<Command> {
    if let Ok(apps) = installed_apps::fetch_installed_apps().await {
        apps.into_iter()
            .filter_map(|app_info| {
                app_info.path.map(|path| {
                    let mut final_keywords = app_info.keywords;
                    // Ensure the main name is part of the keywords list
                    if !final_keywords.contains(&app_info.name) {
                        final_keywords.push(app_info.name.clone());
                    }

                    Command {
                        name: format!("app_{}", app_info.name), // Create a unique name
                        title: app_info.name.clone(),
                        english_name: app_info.name.clone(),
                        keywords: final_keywords
                            .into_iter()
                            .map(|kw| CommandKeyword {
                                name: kw,
                                disabled: None,
                                is_default: Some(true),
                            })
                            .collect(),
                        icon: app_info.icon.unwrap_or_default(),
                        source: ItemSource::Application,
                        action: CommandAction::App(path),
                        origin: app_info.origin,
                    }
                })
            })
            .collect()
    } else {
        vec![]
    }
}

async fn get_initial_file_commands(app: &AppHandle) -> Vec<Command> {
    let file_manager = app.state::<file_command_manager::FileCommandManager>();
    let file_items = file_manager.get_items().await;

    file_items
        .into_iter()
        .map(|item| Command {
            name: format!("file_{}", item.path), // Create a unique name
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
        })
        .collect()
}

fn get_initial_plugin_commands(app: &AppHandle) -> Vec<Command> {
    let plugin_store: tauri::State<plugin_manager::PluginStore> = app.state();
    match plugin_manager::load_plugins(app.clone(), plugin_store) {
        Ok(plugins) => plugins
            .into_iter()
            .map(|plugin| Command {
                name: format!("plugin_{}", plugin.manifest.id),
                title: plugin.manifest.name.clone(),
                english_name: plugin.manifest.name.clone(),
                keywords: vec![CommandKeyword {
                    name: plugin.manifest.name,
                    disabled: None,
                    is_default: Some(true),
                }],
                icon: "".to_string(), // Plugins might not have icons in the manifest yet
                source: ItemSource::Plugin,
                action: CommandAction::Plugin(plugin.manifest.id),
                origin: None,
            })
            .collect(),
        Err(e) => {
            eprintln!("Failed to load plugins as commands: {}", e);
            vec![]
        }
    }
}

// 获取插件中定义的指令
pub fn get_plugin_commands(
    app: &AppHandle,
) -> Vec<(String, Vec<plugin_manager::PluginCommandManifest>)> {
    let plugin_store: tauri::State<plugin_manager::PluginStore> = app.state();
    match plugin_manager::load_plugins(app.clone(), plugin_store) {
        Ok(plugins) => plugins
            .into_iter()
            .filter(|plugin| !plugin.manifest.commands.is_empty())
            .map(|plugin| (plugin.manifest.name, plugin.manifest.commands))
            .collect(),
        Err(e) => {
            eprintln!("Failed to load plugin commands: {}", e);
            vec![]
        }
    }
}

// 获取插件ID到名称的映射
pub fn get_plugin_id_name_mapping(
    app: &AppHandle,
) -> Vec<(String, String)> {
    let plugin_store: tauri::State<plugin_manager::PluginStore> = app.state();
    match plugin_manager::load_plugins(app.clone(), plugin_store) {
        Ok(plugins) => plugins
            .into_iter()
            .filter(|plugin| !plugin.manifest.commands.is_empty())
            .map(|plugin| (plugin.manifest.name.clone(), plugin.manifest.id.clone()))
            .collect(),
        Err(e) => {
            eprintln!("Failed to load plugin id mapping: {}", e);
            vec![]
        }
    }
}

#[tauri::command]
pub async fn get_plugin_commands_list(
    app: AppHandle,
) -> Vec<(String, Vec<plugin_manager::PluginCommandManifest>)> {
    get_plugin_commands(&app)
}

#[tauri::command]
pub async fn get_plugin_id_mapping(
    app: AppHandle,
) -> Vec<(String, String)> {
    get_plugin_id_name_mapping(&app)
}
