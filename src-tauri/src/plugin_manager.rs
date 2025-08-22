use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry: String,
}

#[tauri::command]
pub fn load_plugins(app: tauri::AppHandle) -> Result<Vec<PluginManifest>, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // 插件目录现在位于应用数据目录中
    let plugins_dir = data_dir.join("plugins");

    if !plugins_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(plugins_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.is_file() {
                let manifest_content =
                    std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
                let manifest: PluginManifest =
                    serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;
                plugins.push(manifest);
            }
        }
    }
    Ok(plugins)
}
