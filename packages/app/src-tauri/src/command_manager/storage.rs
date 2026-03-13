//! 命令存储模块
//!
//! 负责命令的持久化存储和加载

use crate::shared_types::{Command, ItemSource};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use super::generators;

/// 获取命令存储文件路径
pub fn get_commands_file_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("commands.json")
}

/// 生成并保存所有命令
pub async fn generate_and_save_commands(app: &AppHandle) -> Vec<Command> {
    let mut initial_commands = generators::get_initial_system_commands();
    let extension_commands = generators::get_initial_extension_commands();
    initial_commands.extend(extension_commands);
    let app_commands = generators::get_initial_app_commands().await;
    initial_commands.extend(app_commands);
    let file_commands = generators::get_initial_file_commands(app).await;
    initial_commands.extend(file_commands);
    let plugin_commands = generators::get_initial_plugin_commands(app);
    initial_commands.extend(plugin_commands);
    let dynamic_commands = generators::get_initial_dynamic_commands(app);
    initial_commands.extend(dynamic_commands);
    save_commands(app, &initial_commands);
    initial_commands
}

/// 加载命令
///
/// 从文件加载命令，并与当前系统命令和插件命令合并
pub async fn load_commands(app: &AppHandle) -> Vec<Command> {
    let path = get_commands_file_path(app);
    if !path.exists() {
        return generate_and_save_commands(app).await;
    }

    match fs::read_to_string(&path) {
        Ok(json_str) => {
            let result: Result<Vec<Command>, serde_json::Error> = serde_json::from_str(&json_str);
            match result {
                Ok(commands) => merge_commands(app, commands).await,
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

/// 合并已保存的命令与当前系统/插件命令
async fn merge_commands(app: &AppHandle, saved_commands: Vec<Command>) -> Vec<Command> {
    // 获取当前系统命令
    let current_system_commands = generators::get_initial_system_commands();
    let current_system_map: HashMap<_, _> = current_system_commands
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    // 获取已安装的插件命令
    let installed_plugins = generators::get_initial_plugin_commands(app);
    let installed_plugins_map: HashMap<_, _> = installed_plugins
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    // 过滤出非系统/非插件命令（保留应用/文件命令）
    // 应用命令的图标已经在生成时通过透明度检测修复，无需每次加载都重新扫描
    let other_commands: Vec<Command> = saved_commands
        .iter()
        .filter(|c| c.source != ItemSource::Plugin && c.source != ItemSource::Command && c.source != ItemSource::Extension)
        .cloned()
        .collect();

    // 合并系统命令（保留用户自定义，添加新命令）
    let mut final_system_commands: Vec<Command> = Vec::new();
    for (name, system_command) in &current_system_map {
        let existing_command = saved_commands
            .iter()
            .find(|c| c.source == ItemSource::Command && &c.name == name);

        if let Some(existing) = existing_command {
            // 保留已保存的版本（含用户修改），但更新系统定义的字段
            let mut merged = existing.clone();
            merged.title = system_command.title.clone();
            merged.description = system_command.description.clone();
            merged.english_name = system_command.english_name.clone();
            merged.icon = system_command.icon.clone();
            merged.action = system_command.action.clone();
            // keywords 保持用户的配置（已保存的）
            
            final_system_commands.push(merged);
        } else {
            // 新系统命令，添加
            final_system_commands.push(system_command.clone());
        }
    }

    // 合并 Extension 命令 (类似系统命令)
    let current_extension_commands = generators::get_initial_extension_commands();
    let current_extension_map: HashMap<_, _> = current_extension_commands
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    let mut final_extensions: Vec<Command> = Vec::new();
    for (name, ext_command) in &current_extension_map {
        let existing = saved_commands
            .iter()
            .find(|c| c.source == ItemSource::Extension && &c.name == name);

        if let Some(saved) = existing {
            let mut merged = saved.clone();
            // 更新元数据，保留用户配置（如 keywords）
            merged.title = ext_command.title.clone();
            merged.description = ext_command.description.clone();
            merged.icon = ext_command.icon.clone();
            merged.action = ext_command.action.clone();
            merged.matches = ext_command.matches.clone();
            final_extensions.push(merged);
        } else {
            final_extensions.push(ext_command.clone());
        }
    }

    // 合并插件命令
    let mut final_plugins: Vec<Command> = Vec::new();
    for (name, plugin_command) in installed_plugins_map {
        let existing_command = saved_commands
            .iter()
            .find(|c| c.source == ItemSource::Plugin && c.name == name);

        if let Some(existing) = existing_command {
            final_plugins.push(existing.clone());
        } else {
            final_plugins.push(plugin_command);
        }
    }

    // 合并动态命令
    let current_dynamic_commands = generators::get_initial_dynamic_commands(app);
    for dynamic_command in current_dynamic_commands {
        let existing_command = saved_commands
            .iter()
            .find(|c| c.source == ItemSource::Plugin && c.name == dynamic_command.name);

        if let Some(existing) = existing_command {
            final_plugins.push(existing.clone());
        } else {
            final_plugins.push(dynamic_command);
        }
    }

    let mut final_commands = final_system_commands;
    final_commands.extend(other_commands);
    final_commands.extend(final_plugins);
    final_commands.extend(final_extensions);
    save_commands(app, &final_commands);
    final_commands
}

/// 保存命令到文件
pub fn save_commands(app: &AppHandle, commands: &[Command]) {
    let path = get_commands_file_path(app);
    let json = serde_json::to_string_pretty(commands).unwrap();
    fs::write(path, json).unwrap();
}

