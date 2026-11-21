//! 应用命令生成器
//!
//! 进程内缓存已安装应用命令：图标提取（Base64）开销大，避免每次 load 都重扫。
//! 进程重启（如用户升级应用后启动）缓存自然为空，会用最新的扫描逻辑重新生成，
//! 这也保证 installed_apps 的修复能被真正应用，而不会被旧的 commands.json 掩盖。

use crate::installed_apps;
use crate::shared_types::{Command, CommandAction, CommandKeyword, ItemSource};
use std::sync::{OnceLock, RwLock};

/// 已安装应用命令的进程内缓存。None 表示尚未扫描。
static APP_COMMANDS_CACHE: OnceLock<RwLock<Option<Vec<Command>>>> = OnceLock::new();

fn cache() -> &'static RwLock<Option<Vec<Command>>> {
    APP_COMMANDS_CACHE.get_or_init(|| RwLock::new(None))
}

/// 使缓存失效，强制下次扫描重新读取本机已安装应用。
pub fn invalidate_app_commands_cache() {
    if let Ok(mut guard) = cache().write() {
        *guard = None;
    }
}

/// 生成已安装应用的命令列表（带进程内缓存）
pub async fn get_initial_app_commands() -> Vec<Command> {
    if let Ok(guard) = cache().read() {
        if let Some(commands) = guard.as_ref() {
            return commands.clone();
        }
    }

    let commands = scan_app_commands().await;

    if let Ok(mut guard) = cache().write() {
        *guard = Some(commands.clone());
    }

    commands
}

/// 真正执行本机已安装应用扫描
async fn scan_app_commands() -> Vec<Command> {
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
                        description: None,
                        keywords: final_keywords
                            .into_iter()
                            .map(|kw| CommandKeyword {
                                name: kw,
                                disabled: None,
                                is_default: Some(true),
                                // TODO: make this configurable if app command supports keyword type in the future
                                keyword_type: None,
                            })
                            .collect(),
                        icon: app_info.icon.unwrap_or_default(),
                        source: ItemSource::Application,
                        action: CommandAction::App(path),
                        origin: app_info.origin,
                        matches: None,
                        requires_confirmation: false,
                        command_type: crate::shared_types::CommandType::Function,
                    }
                })
            })
            .collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_starts_empty_and_can_be_invalidated() {
        // 该测试可能与其它测试共享全局缓存，因此只验证失效接口不会 panic，
        // 且失效后再次读取为空（不依赖是否已被其它测试填充）。
        invalidate_app_commands_cache();
        let guard = cache().read().unwrap();
        assert!(guard.is_none());
    }
}
