use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAlias {
    pub id: String,
    pub alias: String,
    pub command: String,
    pub enabled: bool,
}

const ALIASES_STORE_KEY: &str = "command_aliases";

// 获取指令别名
#[tauri::command]
pub async fn get_command_aliases(app: AppHandle) -> Result<Vec<CommandAlias>, String> {
    let store = app.store("shortcuts.json").map_err(|e| e.to_string())?;

    match store.get(ALIASES_STORE_KEY) {
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|e| format!("Failed to deserialize aliases: {}", e)),
        None => Ok(Vec::new()),
    }
}

// 添加指令别名
#[tauri::command]
pub async fn add_command_alias(app: AppHandle, alias: CommandAlias) -> Result<(), String> {
    let store = app.store("shortcuts.json").map_err(|e| e.to_string())?;

    let mut aliases = get_command_aliases(app.clone()).await?;
    aliases.push(alias);

    store.set(ALIASES_STORE_KEY, serde_json::to_value(&aliases).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// 更新指令别名
#[tauri::command]
pub async fn update_command_alias(app: AppHandle, alias: CommandAlias) -> Result<(), String> {
    let store = app.store("shortcuts.json").map_err(|e| e.to_string())?;

    let mut aliases = get_command_aliases(app.clone()).await?;
    if let Some(pos) = aliases.iter().position(|a| a.id == alias.id) {
        aliases[pos] = alias;

        store.set(ALIASES_STORE_KEY, serde_json::to_value(&aliases).unwrap());
        store.save().map_err(|e| e.to_string())?;
    }

    Ok(())
}

// 删除指令别名
#[tauri::command]
pub async fn remove_command_alias(app: AppHandle, id: String) -> Result<(), String> {
    let store = app.store("shortcuts.json").map_err(|e| e.to_string())?;

    let mut aliases = get_command_aliases(app.clone()).await?;
    aliases.retain(|a| a.id != id);

    store.set(ALIASES_STORE_KEY, serde_json::to_value(&aliases).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
