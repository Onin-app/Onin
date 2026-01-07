//! 全局实例管理模块
//!
//! 管理全局唯一的 PluginRuntimeManager 实例

use once_cell::sync::Lazy;
use std::sync::Arc;
use tauri::AppHandle;

use super::manager::PluginRuntimeManager;

/// 全局插件运行时管理器实例
static PLUGIN_RUNTIME_MANAGER: Lazy<Arc<tokio::sync::Mutex<Option<PluginRuntimeManager>>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

/// 初始化全局插件运行时管理器
///
/// 应该在应用启动时调用一次
pub async fn init_plugin_runtime_manager(app_handle: AppHandle) {
    let mut manager = PLUGIN_RUNTIME_MANAGER.lock().await;
    if manager.is_none() {
        *manager = Some(PluginRuntimeManager::new(app_handle));
    }
}

/// 获取全局插件运行时管理器
///
/// # 返回
/// - `Ok(PluginRuntimeManager)`: 管理器的克隆实例
/// - `Err(String)`: 如果管理器未初始化
pub async fn get_plugin_runtime_manager() -> Result<PluginRuntimeManager, String> {
    let manager = PLUGIN_RUNTIME_MANAGER.lock().await;
    manager
        .as_ref()
        .ok_or_else(|| "Plugin runtime manager not initialized".to_string())
        .map(|m| PluginRuntimeManager {
            sender: m.sender.clone(),
        })
}

/// 清除所有插件运行时
pub async fn clear_all_plugin_runtimes() -> Result<(), String> {
    let manager = get_plugin_runtime_manager().await?;
    manager.clear_all_plugins().await
}

/// 清除指定插件的运行时
pub async fn clear_plugin_runtime(plugin_id: &str) -> Result<(), String> {
    let manager = get_plugin_runtime_manager().await?;
    manager.clear_plugin(plugin_id.to_string()).await
}
