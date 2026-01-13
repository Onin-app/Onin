use crate::command_manager;
use crate::shared_types::{IconType, ItemSource, ItemType, LaunchableItem};
use crate::app_config::{AppConfigState, SortMode};
use crate::usage_tracker::{UsageTracker, UsageTrackerState};
use tauri::Manager;

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

            // 如果没有启用的关键词，且没有 matches 规则，则不包含此命令
            // 有 matches 规则的命令可以通过内容匹配触发，即使没有关键词
            let has_matches = cmd.matches.as_ref().map_or(false, |m| !m.is_empty());
            if enabled_keywords.is_empty() && !has_matches {
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

            // 先计算 icon_type，避免借用移动后的值
            let icon_type = match cmd.source {
                ItemSource::Application => IconType::Base64,
                ItemSource::FileCommand => IconType::Base64,
                // 插件：根据图标格式选择类型
                ItemSource::Plugin if cmd.icon.starts_with("http://") || cmd.icon.starts_with("https://") => {
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
                    _ => "".to_string(),
                },
                icon: cmd.icon,
                icon_type,
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

    // 4. Apply sorting based on usage tracking
    let sorted_items = apply_usage_sorting(&app, all_items)?;

    Ok(sorted_items)
}

fn apply_usage_sorting(
    app: &tauri::AppHandle,
    mut items: Vec<LaunchableItem>,
) -> Result<Vec<LaunchableItem>, String> {
    // Get config
    let config_state = app.state::<AppConfigState>();
    let config = config_state.0.lock().map_err(|e| e.to_string())?;
    
    // If tracking is disabled or mode is Default, return items as-is
    if !config.enable_usage_tracking || config.sort_mode == SortMode::Default {
        return Ok(items);
    }
    
    let sort_mode = config.sort_mode.clone();
    drop(config); // Release lock
    
    // Get usage tracker
    let tracker_state = app.state::<UsageTrackerState>();
    let mut tracker_opt = tracker_state.0.lock().map_err(|e| e.to_string())?;
    
    if tracker_opt.is_none() {
        *tracker_opt = Some(UsageTracker::new(app));
    }
    
    let tracker = tracker_opt.as_ref().ok_or("Failed to get tracker")?;
    
    // Calculate scores for each item
    items.sort_by(|a, b| {
        let score_a = if let Some(action) = &a.action {
            tracker.calculate_score(action, &sort_mode)
        } else {
            0.0
        };
        
        let score_b = if let Some(action) = &b.action {
            tracker.calculate_score(action, &sort_mode)
        } else {
            0.0
        };
        
        // Sort in descending order (higher score first)
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    Ok(items)
}
