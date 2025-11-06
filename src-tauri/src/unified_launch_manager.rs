use crate::command_manager;
use crate::shared_types::{IconType, ItemSource, ItemType, LaunchableItem};

/// A unified command to get all launchable items from various sources.
#[tauri::command]
pub async fn get_all_launchable_items(
    app: tauri::AppHandle,
) -> Result<Vec<LaunchableItem>, String> {
    // 1. Load all commands from the central command store.
    let all_commands = command_manager::load_commands(&app).await;

    // 2. Build a map of plugin IDs to plugin names for display
    let plugin_id_to_name: std::collections::HashMap<String, String> = all_commands
        .iter()
        .filter(|cmd| {
            cmd.source == ItemSource::Plugin
                && cmd.name.starts_with("plugin_")
                && !cmd.name.starts_with("plugin_cmd_")
        })
        .filter_map(|cmd| {
            if let crate::shared_types::CommandAction::Plugin(plugin_id) = &cmd.action {
                Some((plugin_id.clone(), cmd.title.clone()))
            } else {
                None
            }
        })
        .collect();

    // 3. Convert Command into the common LaunchableItem format.
    let all_items: Vec<LaunchableItem> = all_commands
        .into_iter()
        .filter_map(|cmd| {
            // Filter out disabled keywords
            let enabled_keywords: Vec<_> = cmd
                .keywords
                .into_iter()
                .filter(|keyword| !keyword.disabled.unwrap_or(false))
                .collect();

            // If all keywords are disabled, don't include this command at all
            if enabled_keywords.is_empty() {
                return None;
            }

            // Determine source_display for plugin commands
            // Plugin itself (for opening): show "Plugin"
            // Plugin command (for executing): show plugin name
            let source_display = if cmd.source == ItemSource::Plugin {
                if let crate::shared_types::CommandAction::PluginCommand { plugin_id, .. } =
                    &cmd.action
                {
                    // For plugin commands, show the plugin name
                    plugin_id_to_name.get(plugin_id).cloned()
                } else {
                    // For plugin itself, don't set source_display (will show "Plugin")
                    None
                }
            } else {
                None
            };

            Some(LaunchableItem {
                name: cmd.title,
                keywords: enabled_keywords,
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
                    ItemSource::FileCommand => ItemType::File,
                    _ => ItemType::App,
                },
                source: cmd.source,
                action: Some(cmd.name),
                origin: cmd.origin,
                source_display,
                matches: cmd.matches,
            })
        })
        .collect();

    Ok(all_items)
}
