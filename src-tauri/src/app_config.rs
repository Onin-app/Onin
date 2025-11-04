use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 自动粘贴时间限制（秒），0 表示不限制
    #[serde(default = "default_auto_paste_time_limit")]
    pub auto_paste_time_limit: u64,
    
    /// 自动清空剪贴板时间限制（秒），0 表示不自动清空
    #[serde(default = "default_auto_clear_time_limit")]
    pub auto_clear_time_limit: u64,
}

fn default_auto_paste_time_limit() -> u64 {
    5 // 默认 5 秒
}

fn default_auto_clear_time_limit() -> u64 {
    0 // 默认不自动清空
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_paste_time_limit: default_auto_paste_time_limit(),
            auto_clear_time_limit: default_auto_clear_time_limit(),
        }
    }
}

pub struct AppConfigState(pub Mutex<AppConfig>);

/// 获取配置文件路径
fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("app_config.json"))
}

/// 加载配置
pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let config_path = get_config_path(app)?;

    if !config_path.exists() {
        // 如果配置文件不存在，返回默认配置
        return Ok(AppConfig::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// 保存配置
pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app)?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_app_config(_app: AppHandle, state: tauri::State<'_, AppConfigState>) -> Result<AppConfig, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_app_config(
    app: AppHandle,
    state: tauri::State<'_, AppConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    // 更新内存中的配置
    {
        let mut current_config = state.0.lock().map_err(|e| e.to_string())?;
        *current_config = config.clone();
    }

    // 保存到文件
    save_config(&app, &config)?;

    println!("[app_config] Config updated: {:?}", config);
    Ok(())
}
