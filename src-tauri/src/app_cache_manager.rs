use crate::installed_apps::{self, AppInfo};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;

// 用于缓存已安装应用列表的状态
pub struct AppCache {
    pub apps: RwLock<Option<Vec<AppInfo>>>,
}

// 从缓存中获取应用列表的命令
#[tauri::command]
pub async fn get_installed_apps(cache: State<'_, AppCache>) -> Result<Vec<AppInfo>, String> {
    let apps_guard = cache.apps.read().await;
    // 立即从缓存返回数据，如果缓存为空则返回空列表
    Ok(apps_guard.clone().unwrap_or_default())
}

// 触发应用列表后台刷新的辅助函数
pub fn trigger_app_refresh(app: AppHandle) {
    // 在后台任务中执行刷新操作，避免阻塞UI
    tauri::async_runtime::spawn(async move {
        // 从 AppHandle 获取 State，这样可以确保生命周期正确
        let cache: State<AppCache> = app.state();
        println!("Background refreshing installed apps list...");
        // 调用我们重构的、耗时的函数
        match installed_apps::fetch_installed_apps().await {
            Ok(new_apps) => {
                let mut apps_guard = cache.apps.write().await;
                *apps_guard = Some(new_apps);
                println!("App list cache updated successfully.");
                // 通知前端列表已更新，以便前端可以重新获取
                if let Err(e) = app.emit("apps_updated", ()) {
                    eprintln!("[ERROR] Failed to emit apps_updated event: {}", e);
                }
            }
            Err(e) => eprintln!("[ERROR] Failed to refresh app list: {}", e),
        }
    });
}