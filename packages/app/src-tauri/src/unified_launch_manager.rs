use crate::app_config::{AppConfigState, SortMode};
use crate::command_manager;
use crate::shared_types::{IconType, ItemSource, ItemType, LaunchableItem};
use crate::usage_tracker::{UsageTracker, UsageTrackerState};
use tauri::Manager;

/// A unified command to get all launchable items from various sources.
#[tauri::command]
pub async fn get_all_launchable_items(
    app: tauri::AppHandle,
) -> Result<Vec<LaunchableItem>, String> {
    // 1. Load all commands from the central command store.
    let all_commands = command_manager::load_commands(&app).await;

    // 2. Build a map of plugin IDs to plugin names for display.
    let plugin_id_to_name: std::collections::HashMap<String, String> = all_commands
        .iter()
        .filter(|cmd| {
            cmd.source == ItemSource::Plugin
                && cmd.name.starts_with("plugin_")
                && !cmd.name.starts_with("plugin_cmd_")
        })
        .filter_map(|cmd| {
            #[allow(deprecated)]
            match &cmd.action {
                crate::shared_types::CommandAction::PluginEntry { plugin_id } => {
                    Some((plugin_id.clone(), cmd.title.clone()))
                }

                _ => None,
            }
        })
        .collect();

    // 3. Convert Command into the common LaunchableItem format.
    let all_items: Vec<LaunchableItem> = all_commands
        .into_iter()
        .filter_map(|cmd| {
            let enabled_keywords: Vec<_> = cmd
                .keywords
                .into_iter()
                .filter(|keyword| !keyword.disabled.unwrap_or(false))
                .collect();

            let has_matches = cmd.matches.as_ref().is_some_and(|m| !m.is_empty());
            if enabled_keywords.is_empty() && !has_matches {
                return None;
            }

            let source_display = if cmd.source == ItemSource::Plugin {
                if let crate::shared_types::CommandAction::PluginCommand { plugin_id, .. } =
                    &cmd.action
                {
                    plugin_id_to_name.get(plugin_id).cloned()
                } else {
                    None
                }
            } else {
                None
            };

            let icon_type = match cmd.source {
                ItemSource::Application => IconType::Base64,
                ItemSource::FileCommand => IconType::Base64,
                ItemSource::FileSearch => IconType::Iconfont,
                ItemSource::Plugin
                    if cmd.icon.starts_with("http://") || cmd.icon.starts_with("https://") =>
                {
                    IconType::Url
                }
                ItemSource::Plugin
                    if cmd.icon.starts_with("data:")
                        || (!cmd.icon.is_empty() && !cmd.icon.starts_with("icon-")) =>
                {
                    IconType::Base64
                }
                _ => IconType::Iconfont,
            };

            Some(LaunchableItem {
                name: cmd.title,
                description: cmd.description,
                keywords: enabled_keywords,
                path: match &cmd.action {
                    crate::shared_types::CommandAction::App(path) => path.clone(),
                    crate::shared_types::CommandAction::File(path) => path.clone(),
                    _ => String::new(),
                },
                icon: cmd.icon,
                icon_type,
                item_type: match cmd.source {
                    ItemSource::FileCommand => ItemType::File,
                    ItemSource::FileSearch => ItemType::File,
                    _ => ItemType::App,
                },
                source: cmd.source,
                action: Some(cmd.name),
                origin: cmd.origin,
                source_display,
                matches: cmd.matches,
                modified_time: None,
                requires_confirmation: cmd.requires_confirmation,
            })
        })
        .collect();

    // 4. Apply sorting based on usage tracking.
    let sorted_items = apply_usage_sorting(&app, all_items)?;

    Ok(sorted_items)
}

fn apply_usage_sorting(
    app: &tauri::AppHandle,
    mut items: Vec<LaunchableItem>,
) -> Result<Vec<LaunchableItem>, String> {
    let config_state = app.state::<AppConfigState>();
    let config = config_state.0.lock().map_err(|e| e.to_string())?;

    if !config.enable_usage_tracking || config.sort_mode == SortMode::Default {
        return Ok(items);
    }

    let sort_mode = config.sort_mode.clone();
    drop(config);

    let tracker_state = app.state::<UsageTrackerState>();
    let mut tracker_opt = tracker_state.0.lock().map_err(|e| e.to_string())?;

    if tracker_opt.is_none() {
        *tracker_opt = Some(UsageTracker::new(app));
    }

    let tracker = tracker_opt.as_ref().ok_or("Failed to get tracker")?;

    items.sort_by(|a, b| {
        let score_a = a
            .action
            .as_ref()
            .map(|action| tracker.calculate_score(action, &sort_mode))
            .unwrap_or(0.0);
        let score_b = b
            .action
            .as_ref()
            .map(|action| tracker.calculate_score(action, &sort_mode))
            .unwrap_or(0.0);

        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(items)
}
