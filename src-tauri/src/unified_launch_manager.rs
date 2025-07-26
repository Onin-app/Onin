use crate::app_cache_manager::AppCache;
use crate::installed_apps;
use crate::shared_types::{ItemSource, ItemType, LaunchableItem};
use crate::startup_apps_manager::StartupAppsManager;
use tauri::State;

/// A unified command to get all launchable items from various sources.
/// It combines installed applications from the cache and custom startup items.
#[tauri::command]
pub async fn get_all_launchable_items(
    app_cache: State<'_, AppCache>,
    startup_manager: State<'_, StartupAppsManager>,
) -> Result<Vec<LaunchableItem>, String> {
    // We need a write lock because we might populate the cache.
    let mut apps_guard = app_cache.apps.write().await;

    // 1. If the cache is empty, this is likely the first run.
    //    Fetch the apps now and populate the cache. This ensures the first load is always complete.
    if apps_guard.is_none() {
        tracing::info!("App cache is empty. Performing initial fetch...");
        match installed_apps::fetch_installed_apps().await {
            Ok(new_apps) => {
                *apps_guard = Some(new_apps);
                tracing::info!("App cache populated successfully.");
            }
            Err(e) => {
                // If fetching fails, log the error but continue, so custom items can still be shown.
                // We set the cache to an empty Vec to prevent re-fetching on every call.
                tracing::error!("Initial app fetch failed: {}", e);
                *apps_guard = Some(vec![]);
            }
        }
    }

    let installed_apps = apps_guard.clone().unwrap_or_default();

    // 2. Convert AppInfo into the common LaunchableItem format.
    // We filter out any apps that don't have a valid executable path.
    let mut all_items: Vec<LaunchableItem> = installed_apps
        .into_iter()
        .filter_map(|app_info| {
            app_info.path.map(|path| LaunchableItem {
                name: app_info.name,
                aliases: app_info.aliases,
                path,
                icon: app_info.icon.unwrap_or_default(), // Use default icon if None
                icon_type: crate::shared_types::IconType::Base64,
                item_type: ItemType::App,
                source: ItemSource::Application,
                action: None,
            })
        })
        .collect();

    // 3. Get custom startup items. These are already in the LaunchableItem format.
    let custom_items = startup_manager.get_items().await;

    // 4. Combine the lists.
    all_items.extend(custom_items);

    Ok(all_items)
}
