use crate::command_manager;
use crate::shared_types::{IconType, ItemSource, ItemType, LaunchableItem};

/// A unified command to get all launchable items from various sources.
#[tauri::command]
pub async fn get_all_launchable_items(
    app: tauri::AppHandle,
) -> Result<Vec<LaunchableItem>, String> {
    // 1. Load all commands from the central command store.
    let all_commands = command_manager::load_commands(&app).await;

    // 2. Convert Command into the common LaunchableItem format.
    let all_items: Vec<LaunchableItem> = all_commands
        .into_iter()
        .map(|cmd| LaunchableItem {
            name: cmd.title,
            keywords: cmd.keywords,
            path: match &cmd.action {
                crate::shared_types::CommandAction::App(path) => path.clone(),
                crate::shared_types::CommandAction::File(path) => path.clone(),
                _ => "".to_string(),
            },
            icon: cmd.icon,
            icon_type: match cmd.source {
                ItemSource::Application => IconType::Base64,
                ItemSource::FileCommand => IconType::Base64,
                _ => IconType::Iconfont,
            },
            item_type: match cmd.source {
                ItemSource::FileCommand => ItemType::File, // Or determine from path
                _ => ItemType::App,
            },
            source: cmd.source,
            action: Some(cmd.name), // The unique name is the action identifier
        })
        .collect();

    Ok(all_items)
}
