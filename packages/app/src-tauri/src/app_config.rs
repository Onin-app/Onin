use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortMode {
    #[serde(rename = "smart")]
    Smart, // 智能排序：综合频率和最近使用
    #[serde(rename = "frequency")]
    Frequency, // 纯频率排序
    #[serde(rename = "recent")]
    Recent, // 最近使用排序
    #[serde(rename = "default")]
    Default, // 默认排序（不使用频率）
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::Smart
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 自动粘贴时间限制（秒），0 表示不限制
    #[serde(default = "default_auto_paste_time_limit")]
    pub auto_paste_time_limit: u64,

    /// 自动清空剪贴板时间限制（秒），0 表示不自动清空
    #[serde(default = "default_auto_clear_time_limit")]
    pub auto_clear_time_limit: u64,

    /// 指令排序模式
    #[serde(default)]
    pub sort_mode: SortMode,

    /// 是否启用使用频率追踪
    #[serde(default = "default_enable_usage_tracking")]
    pub enable_usage_tracking: bool,

    /// 插件市场 API 地址（可选）
    #[serde(default = "default_marketplace_api_url")]
    pub marketplace_api_url: Option<String>,

    /// 已禁用的内置扩展 ID
    #[serde(default)]
    pub disabled_extension_ids: Vec<String>,

    /// 文件搜索索引根目录
    #[serde(default = "default_file_search_roots")]
    pub file_search_roots: Vec<String>,

    /// 文件搜索排除路径
    #[serde(default)]
    pub file_search_excluded_paths: Vec<String>,

    /// 文件搜索是否包含隐藏文件
    #[serde(default)]
    pub file_search_include_hidden: bool,
}

fn default_auto_paste_time_limit() -> u64 {
    5 // 默认 5 秒
}

fn default_auto_clear_time_limit() -> u64 {
    0 // 默认不自动清空
}

fn default_enable_usage_tracking() -> bool {
    true // 默认启用
}

fn default_marketplace_api_url() -> Option<String> {
    Some("https://onin.baiyapeng.cc".to_string())
}

pub fn default_file_search_roots() -> Vec<String> {
    #[cfg(target_os = "windows")]
    let home = std::env::var_os("USERPROFILE");

    #[cfg(not(target_os = "windows"))]
    let home = std::env::var_os("HOME");

    home.map(PathBuf::from)
        .map(|path| vec![path.to_string_lossy().to_string()])
        .unwrap_or_default()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_paste_time_limit: default_auto_paste_time_limit(),
            auto_clear_time_limit: default_auto_clear_time_limit(),
            sort_mode: SortMode::default(),
            enable_usage_tracking: default_enable_usage_tracking(),
            marketplace_api_url: default_marketplace_api_url(),
            disabled_extension_ids: Vec::new(),
            file_search_roots: default_file_search_roots(),
            file_search_excluded_paths: Vec::new(),
            file_search_include_hidden: false,
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

    let config: AppConfig =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

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

pub fn is_extension_enabled(app: &AppHandle, extension_id: &str) -> bool {
    let config_state = app.state::<AppConfigState>();
    let Ok(config) = config_state.0.lock() else {
        return true;
    };

    !config
        .disabled_extension_ids
        .iter()
        .any(|id| id == extension_id)
}

#[tauri::command]
pub fn get_app_config(
    _app: AppHandle,
    state: tauri::State<'_, AppConfigState>,
) -> Result<AppConfig, String> {
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

    Ok(())
}
