use crate::command_manager;
use crate::shared_types::{IconType, ItemSource, ItemType, LaunchableItem};
use crate::startup_apps_manager::StartupAppsManager;
use tauri::State;

/// A unified command to get all launchable items from various sources.
/// It combines commands from commands.json and custom startup items.
#[tauri::command]
pub async fn get_all_launchable_items(
    startup_manager: State<'_, StartupAppsManager>,
    app: tauri::AppHandle,
) -> Result<Vec<LaunchableItem>, String> {
    // 1. Load all commands from the central command store.
    let all_commands = command_manager::load_commands(&app).await;

    // 2. Convert Command into the common LaunchableItem format.
    let mut all_items: Vec<LaunchableItem> = all_commands
        .into_iter()
        .map(|cmd| LaunchableItem {
            name: cmd.title,
            keywords: cmd
                .keywords
                .into_iter()
                .filter(|kw| kw.disabled.is_none() || !kw.disabled.unwrap())
                .map(|kw| kw.name)
                .collect(),
            path: match &cmd.action {
                crate::shared_types::CommandAction::App(path) => path.clone(),
                _ => "".to_string(),
            },
            icon: cmd.icon,
            icon_type: match cmd.source {
                ItemSource::Application => IconType::Base64,
                _ => IconType::Iconfont,
            },
            item_type: ItemType::App, // All commands are treated as apps for launching
            source: cmd.source,
            action: Some(cmd.name), // The unique name is the action identifier
        })
        .collect();

    // 3. Get custom startup items. These are already in the LaunchableItem format.
    let custom_items = startup_manager.get_items().await;

    // 4. Combine the lists.
    all_items.extend(custom_items);

    Ok(all_items)
}
