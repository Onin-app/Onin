//! 应用命令生成器

use crate::installed_apps;
use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};

/// 生成已安装应用的命令列表
pub async fn get_initial_app_commands() -> Vec<Command> {
    if let Ok(apps) = installed_apps::fetch_installed_apps().await {
        apps.into_iter()
            .filter_map(|app_info| {
                app_info.path.map(|path| {
                    let mut final_keywords = app_info.keywords;
                    // 确保主名称在关键词列表中
                    if !final_keywords.contains(&app_info.name) {
                        final_keywords.push(app_info.name.clone());
                    }

                    Command {
                        name: format!("app_{}", app_info.name),
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
                        matches: None,
                    }
                })
            })
            .collect()
    } else {
        vec![]
    }
}
