use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use crate::{installed_apps, system_commands};
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
        Ok(json_str) => match serde_json::from_str(&json_str) {
            Ok(commands) => commands,
            Err(e) => {
                eprintln!("Failed to parse commands.json: {}. Deleting and regenerating.", e);
                if let Err(err) = fs::remove_file(&path) {
                    eprintln!("Failed to delete corrupted commands.json: {}", err);
                }
                generate_and_save_commands(app).await
            }
        },
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
                    })
                    .collect();
                // Ensure title and english_name are also part of keywords so they can be disabled.
                if !kws.iter().any(|kw| kw.name == cmd_info.title) {
                    kws.push(CommandKeyword {
                        name: cmd_info.title.to_string(),
                        disabled: None,
                    });
                }
                if !kws.iter().any(|kw| kw.name == cmd_info.english_name) {
                    kws.push(CommandKeyword {
                        name: cmd_info.english_name.to_string(),
                        disabled: None,
                    });
                }
                kws
            },
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Command,
            action: CommandAction::System(cmd_info.name.to_string()),
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
                            })
                            .collect(),
                        icon: app_info.icon.unwrap_or_default(),
                        source: ItemSource::Application,
                        action: CommandAction::App(path),
                    }
                })
            })
            .collect()
    } else {
        vec![]
    }
}