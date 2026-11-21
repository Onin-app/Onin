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
        if let Err(e) = fs::create_dir_all(&path) {
            tracing::error!("Failed to create app_data_dir for commands: {:?}", e);
        }
    }
    path.join("commands.json")
}

/// 生成并保存所有命令
pub async fn generate_and_save_commands(app: &AppHandle) -> Vec<Command> {
    // 强制重新扫描本机已安装应用，使手动刷新 / 首次生成总是拿到最新状态
    generators::invalidate_app_commands_cache();
    let mut initial_commands = generators::get_initial_system_commands();
    let extension_commands = generators::get_initial_extension_commands(app);
    initial_commands.extend(extension_commands);
    let internal_commands = generators::get_initial_internal_commands();
    initial_commands.extend(internal_commands);
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

/// 归一化 saved 命令的 app_ 名称键，使其与本机扫描结果的名称对齐。
///
/// Application 命令的 name 形如 `app_<显示名>`；扫描器会归一化显示名，
/// 但旧版持久化的名称可能未归一化（如 `app_钉钉（桌面版）`）。
/// 这里剥掉 `app_` 前缀后复用 installed_apps 的归一化规则。
fn normalize_saved_app_name(command_name: &str) -> String {
    let display = command_name.strip_prefix("app_").unwrap_or(command_name);
    format!(
        "app_{}",
        crate::installed_apps::normalize_app_name_for_match(display)
    )
}

/// 合并一条机器特定命令：以当前扫描结果为权威，继承用户对关键词的自定义。
///
/// `current` 来自本机权威源（installed_apps / file_commands.json），其 name/path/icon/origin
/// 一律采用；`saved` 仅用于保留用户改动：
/// - 用户新增的关键词（不在当前默认关键词中）保留；
/// - 默认关键词的 disabled 状态沿用用户的设置。
fn merge_machine_specific_command(current: Command, saved: Option<&Command>) -> Command {
    let Some(saved) = saved else {
        return current;
    };

    let mut merged = current;
    for saved_kw in &saved.keywords {
        if let Some(default_kw) = merged
            .keywords
            .iter_mut()
            .find(|kw| kw.name == saved_kw.name)
        {
            // 默认关键词：沿用用户的禁用状态
            if let Some(disabled) = saved_kw.disabled {
                default_kw.disabled = Some(disabled);
            }
        } else {
            // 用户新增的关键词：保留
            let mut kw = saved_kw.clone();
            kw.is_default = Some(false);
            merged.keywords.push(kw);
        }
    }
    merged
}

/// 一条命令是否包含机器特定数据（本机路径），必须从本机权威源重建而不能信任持久化值。
fn is_machine_specific_source(source: &ItemSource) -> bool {
    matches!(source, ItemSource::Application | ItemSource::FileCommand)
}

/// 用本机的权威扫描结果重建机器特定命令（Application / FileCommand）。
///
/// 这是 load 流程的核心：saved 里的路径绝不信任，仅继承用户对关键词的自定义。
/// 抽成纯函数便于端到端测试（给定旧 commands.json + 本机权威源，验证旧路径被替换）。
fn rebuild_machine_specific_commands(
    saved_commands: &[Command],
    current_apps: Vec<Command>,
    current_files: Vec<Command>,
) -> (Vec<Command>, Vec<Command>) {
    // saved 侧按归一化名建索引：旧版本持久化的名称可能带括号/版本号，
    // 本机扫描结果用的是归一化名（见 normalize_app_name_for_match），
    // 直接按原始 name 匹配会在"重命名"的条目上丢失用户自定义关键词。
    // 文件指令不经过应用名归一化，它们的 name 是稳定的 `file_<path>`。
    let saved_app_map: HashMap<String, &Command> = saved_commands
        .iter()
        .filter(|c| c.source == ItemSource::Application)
        .map(|c| (normalize_saved_app_name(&c.name), c))
        .collect();
    let saved_file_map: HashMap<&str, &Command> = saved_commands
        .iter()
        .filter(|c| c.source == ItemSource::FileCommand)
        .map(|c| (c.name.as_str(), c))
        .collect();

    let rebuilt_apps = current_apps
        .into_iter()
        .map(|c| {
            let saved = saved_app_map
                .get(&normalize_saved_app_name(&c.name))
                .copied();
            merge_machine_specific_command(c, saved)
        })
        .collect();
    let rebuilt_files = current_files
        .into_iter()
        .map(|c| {
            let saved = saved_file_map.get(c.name.as_str()).copied();
            merge_machine_specific_command(c, saved)
        })
        .collect();

    (rebuilt_apps, rebuilt_files)
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

    // 机器特定的命令（Application / FileCommand）不能信任持久化数据：
    // 路径是本机绝对路径 / shell:AppsFolder AUMID，升级后扫描逻辑可能已修复，
    // 跨机备份恢复后更是来自另一台电脑。因此总是以当前权威源重新生成，
    // 仅从 saved 中继承用户对关键词的自定义（disabled/用户新增别名）。
    let current_app_commands = generators::get_initial_app_commands().await;
    let current_file_commands = generators::get_initial_file_commands(app).await;

    let (refreshed_app_commands, refreshed_file_commands) = rebuild_machine_specific_commands(
        &saved_commands,
        current_app_commands,
        current_file_commands,
    );

    // 其余来源（如 FileSearch 等）没有机器特定路径，原样保留即可。
    // 被排除的来源一次性列清：系统/插件/扩展/内部命令由下方各自合并，
    // 机器特定命令（Application/FileCommand）由 rebuild_machine_specific_commands 重建。
    let merged_from_current_sources = [
        ItemSource::Plugin,
        ItemSource::Command,
        ItemSource::Extension,
        ItemSource::Internal,
    ];
    let other_commands: Vec<Command> = saved_commands
        .iter()
        .filter(|c| {
            !merged_from_current_sources.contains(&c.source)
                && !is_machine_specific_source(&c.source)
        })
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
    let current_extension_commands = generators::get_initial_extension_commands(app);
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
            merged.command_type = ext_command.command_type.clone();
            final_extensions.push(merged);
        } else {
            final_extensions.push(ext_command.clone());
        }
    }

    // 合并 Internal 命令 (类似系统命令)
    let current_internal_commands = generators::get_initial_internal_commands();
    let current_internal_map: HashMap<_, _> = current_internal_commands
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    let mut final_internal_commands: Vec<Command> = Vec::new();
    for (name, internal_command) in &current_internal_map {
        let existing = saved_commands
            .iter()
            .find(|c| c.source == ItemSource::Internal && &c.name == name);

        if let Some(saved) = existing {
            let mut merged = saved.clone();
            // 更新元数据，保留用户配置（如 keywords）
            merged.title = internal_command.title.clone();
            merged.description = internal_command.description.clone();
            merged.icon = internal_command.icon.clone();
            merged.action = internal_command.action.clone();
            merged.requires_confirmation = internal_command.requires_confirmation;
            final_internal_commands.push(merged);
        } else {
            final_internal_commands.push(internal_command.clone());
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
    final_commands.extend(refreshed_app_commands);
    final_commands.extend(refreshed_file_commands);
    final_commands.extend(final_plugins);
    final_commands.extend(final_extensions);
    final_commands.extend(final_internal_commands);
    save_commands(app, &final_commands);
    final_commands
}

/// 保存命令到文件
pub fn save_commands(app: &AppHandle, commands: &[Command]) {
    let path = get_commands_file_path(app);
    let json = serde_json::to_string_pretty(commands).unwrap();
    if let Err(e) = fs::write(path, json) {
        tracing::error!("Failed to write commands.json: {:?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_types::{CommandAction, CommandKeyword, CommandType};

    fn kw(name: &str, disabled: Option<bool>, is_default: Option<bool>) -> CommandKeyword {
        CommandKeyword {
            name: name.to_string(),
            disabled,
            is_default,
            keyword_type: None,
        }
    }

    fn app_command(keywords: Vec<CommandKeyword>, path: &str) -> Command {
        Command {
            name: "app_小米电脑管家".to_string(),
            title: "小米电脑管家".to_string(),
            description: None,
            keywords,
            icon: "new-icon".to_string(),
            source: ItemSource::Application,
            action: CommandAction::App(path.to_string()),
            origin: None,
            matches: None,
            requires_confirmation: false,
            command_type: CommandType::Function,
        }
    }

    #[test]
    fn machine_specific_source_classification() {
        assert!(is_machine_specific_source(&ItemSource::Application));
        assert!(is_machine_specific_source(&ItemSource::FileCommand));
        assert!(!is_machine_specific_source(&ItemSource::Command));
        assert!(!is_machine_specific_source(&ItemSource::Plugin));
        assert!(!is_machine_specific_source(&ItemSource::Extension));
        assert!(!is_machine_specific_source(&ItemSource::Internal));
        assert!(!is_machine_specific_source(&ItemSource::FileSearch));
    }

    #[test]
    fn machine_specific_command_uses_current_path_not_saved() {
        // 当前扫描结果是权威：即使 saved 里是旧的 Uninstall.exe，也要用当前路径
        let current = app_command(
            vec![kw("小米电脑管家", None, Some(true))],
            r"C:\Program Files\MI\XiaomiPCManager\XiaomiPcManager.exe",
        );
        let saved = app_command(
            vec![kw("小米电脑管家", None, Some(true))],
            r"C:\...\Uninstall.exe",
        );

        let merged = merge_machine_specific_command(current, Some(&saved));

        match merged.action {
            CommandAction::App(p) => assert_eq!(
                p,
                r"C:\Program Files\MI\XiaomiPCManager\XiaomiPcManager.exe"
            ),
            _ => panic!("expected App action"),
        }
        assert_eq!(merged.icon, "new-icon");
    }

    #[test]
    fn machine_specific_command_keeps_user_disabled_state() {
        let current = app_command(
            vec![
                kw("小米电脑管家", None, Some(true)),
                kw("默认别名", None, Some(true)),
            ],
            "current.exe",
        );
        // 用户禁用了默认别名
        let saved = app_command(
            vec![
                kw("小米电脑管家", None, Some(true)),
                kw("默认别名", Some(true), Some(true)),
            ],
            "old.exe",
        );

        let merged = merge_machine_specific_command(current, Some(&saved));

        let disabled = merged
            .keywords
            .iter()
            .find(|k| k.name == "默认别名")
            .unwrap()
            .disabled;
        assert_eq!(disabled, Some(true));
    }

    #[test]
    fn machine_specific_command_keeps_user_added_keyword() {
        let current = app_command(vec![kw("钉钉", None, Some(true))], "current.exe");
        // 用户新增了自定义别名
        let saved = app_command(
            vec![
                kw("钉钉", None, Some(true)),
                kw("我的钉钉", None, Some(false)),
            ],
            "old.exe",
        );

        let merged = merge_machine_specific_command(current, Some(&saved));

        let added = merged
            .keywords
            .iter()
            .find(|k| k.name == "我的钉钉")
            .unwrap();
        assert_eq!(added.is_default, Some(false));
    }

    #[test]
    fn machine_specific_command_without_saved_returns_current() {
        let current = app_command(vec![kw("钉钉", None, Some(true))], "current.exe");
        let merged = merge_machine_specific_command(current, None);
        match merged.action {
            CommandAction::App(p) => assert_eq!(p, "current.exe"),
            _ => panic!("expected App action"),
        }
        assert_eq!(merged.keywords.len(), 1);
    }

    /// 端到端（不含 AppHandle 的纯逻辑部分）：模拟旧 commands.json 中残留
    /// 小米电脑管家的 Uninstall.exe 路径 + 用户自定义关键词，验证重建后
    /// 路径被本机扫描结果替换、且用户关键词被保留。对应 FIX_PLAN 第二轮验证项 3。
    /// 该用例不依赖平台特定的名称归一化（saved/current 名称一致），全平台可运行。
    #[test]
    fn rebuild_replaces_stale_uninstaller_path_and_keeps_user_keywords() {
        // saved（旧 commands.json）：路径是卸载器，含用户新增别名
        let saved_stale = app_command(
            vec![
                kw("小米电脑管家", None, Some(true)),
                kw("我的管家", None, Some(false)), // 用户新增别名
            ],
            r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\Uninstall.exe",
        );

        // current（本机权威源）：正确路径
        let current = app_command(
            vec![kw("小米电脑管家", None, Some(true))],
            r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\XiaomiPcManager.exe",
        );

        let (apps, files) =
            rebuild_machine_specific_commands(&[saved_stale], vec![current], vec![]);
        assert!(files.is_empty());
        assert_eq!(apps.len(), 1);

        // 路径必须是本机权威源的正确路径，绝不能是卸载器
        match &apps[0].action {
            CommandAction::App(p) => {
                assert!(p.ends_with("XiaomiPcManager.exe"), "unexpected path: {}", p);
                assert!(!p.to_lowercase().contains("uninstall"));
            }
            _ => panic!("expected App action"),
        }
        // 名称沿用当前扫描结果
        assert_eq!(apps[0].name, "app_小米电脑管家");
        // 用户新增的关键词被保留
        assert!(
            apps[0].keywords.iter().any(|k| k.name == "我的管家"),
            "user keyword should be preserved"
        );
    }

    /// 名称归一化后仍能匹配 saved 命令并保留用户关键词。
    /// 全角括号剥离仅 Windows 扫描器具备（其它平台扫描器不产生带括号名称），
    /// 因此该用例限定 Windows，避免在非 Windows 上因归一化规则不同而失败。
    #[cfg(target_os = "windows")]
    #[test]
    fn rebuild_matches_renamed_app_and_keeps_user_keyword_windows() {
        let mut saved = app_command(
            vec![
                kw("钉钉", None, Some(true)),
                kw("我的钉钉", None, Some(false)),
            ],
            "old.exe",
        );
        // 旧版本持久化的名称带全角括号（Windows 扫描器会归一化为无括号）
        saved.name = "app_钉钉（桌面版）".to_string();

        let mut current = app_command(vec![kw("钉钉", None, Some(true))], "current.exe");
        current.name = "app_钉钉".to_string(); // app_command 帮助函数默认名是小米，这里改成钉钉
        current.title = "钉钉".to_string();

        let (apps, _) = rebuild_machine_specific_commands(&[saved], vec![current], vec![]);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "app_钉钉");
        assert!(
            apps[0].keywords.iter().any(|k| k.name == "我的钉钉"),
            "user keyword should survive app-name normalization"
        );
    }
}
