use crate::shared_types::{Command, CommandKeyword, ItemSource};
use crate::system_commands;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn get_commands_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("commands.json")
}

pub fn init(app: &AppHandle) {
    let path = get_commands_file_path(app);
    if !path.exists() {
        let initial_commands = get_initial_system_commands();
        let json = serde_json::to_string_pretty(&initial_commands).unwrap();
        fs::write(path, json).unwrap();
    }
}

pub fn load_commands(app: &AppHandle) -> Vec<Command> {
    let path = get_commands_file_path(app);
    if !path.exists() {
        return vec![];
    }
    let json_str = fs::read_to_string(path).unwrap();
    serde_json::from_str(&json_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse commands.json: {}", e);
        vec![]
    })
}

fn save_commands(app: &AppHandle, commands: &[Command]) {
    let path = get_commands_file_path(app);
    let json = serde_json::to_string_pretty(commands).unwrap();
    fs::write(path, json).unwrap();
}

#[tauri::command]
pub fn update_command(app: AppHandle, command_to_update: Command) {
    let mut commands = load_commands(&app);
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
            keywords: cmd_info
                .aliases
                .iter()
                .map(|&alias| CommandKeyword {
                    name: alias.to_string(),
                    disabled: None,
                })
                .collect(),
            icon: cmd_info.icon.to_string(),
            source: ItemSource::Command,
        })
        .collect()
}