use crate::js_runtime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::http::{Request, Response};
use tauri::{Emitter, Manager, State, WebviewWindowBuilder};

// 在编译时加载模板文件
const PLUGIN_WINDOW_TOPBAR_TEMPLATE: &str = include_str!("../templates/plugin-window-topbar.html");
const PLUGIN_WINDOW_CONTROLS_SCRIPT: &str = include_str!("../templates/plugin-window-controls.js");

pub struct PluginStore(pub Mutex<HashMap<String, LoadedPlugin>>);

// 用于跟踪当前活跃的插件窗口
pub struct ActivePluginWindow(pub Mutex<Option<String>>);

// 用于跟踪正在创建的插件窗口，防止重复创建
pub struct PluginWindowCreating(pub Mutex<std::collections::HashSet<String>>);

// 用于存储插件服务器端口
pub struct PluginServerPort(pub Mutex<Option<u16>>);

// 用于防止窗口切换的防抖机制
pub struct PluginWindowToggleDebounce(pub Mutex<HashMap<String, std::time::Instant>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandManifest {
    pub code: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<PluginCommandKeyword>,
    #[serde(default)]
    pub matches: Vec<PluginCommandMatch>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandKeyword {
    pub name: String,
    #[serde(rename = "type")]
    pub keyword_type: String,
}

/// 插件命令匹配配置
///
/// 三层优雅降级模型：
/// 1. 开发者层：只需配置 extensions（如 [".png", ".jpg"]）
/// 2. 系统层：自动将 extensions 映射为内部 MIME 类型
/// 3. 运行层：优先使用 MIME 类型判断，fallback 到 extensions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandMatch {
    #[serde(rename = "type")]
    pub match_type: String, // "text" | "image" | "file" | "folder"
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>, // 仅 type=text 时使用，作为额外的匹配条件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>, // text: 字符数, file/image/folder: 文件数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(default)]
    pub extensions: Vec<String>, // 文件扩展名数组（如 [".png", ".jpg"]）
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default, rename = "allowUrls")]
    pub allow_urls: Vec<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default, rename = "maxRetries")]
    pub max_retries: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoragePermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub local: bool,
    #[serde(default)]
    pub session: bool,
    #[serde(default, rename = "maxSize")]
    pub max_size: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub sound: bool,
    #[serde(default)]
    pub badge: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default, rename = "allowCommands")]
    pub allow_commands: Vec<String>,
    #[serde(default, rename = "maxExecutionTime")]
    pub max_execution_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchedulerPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default, rename = "maxTasks")]
    pub max_tasks: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginPermissions {
    #[serde(default)]
    pub http: Option<HttpPermission>,
    #[serde(default)]
    pub storage: Option<StoragePermission>,
    #[serde(default)]
    pub notification: Option<NotificationPermission>,
    #[serde(default)]
    pub command: Option<CommandPermission>,
    #[serde(default)]
    pub scheduler: Option<SchedulerPermission>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
    #[serde(default)]
    pub commands: Vec<PluginCommandManifest>,
    pub permissions: Option<PluginPermissions>,
    /// Display mode for UI plugins: "inline" (default) or "window"
    /// - "inline": Display in main window list area
    /// - "window": Open in new webview window
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    /// Auto detach to separate window when executing
    /// If true, HTML plugins will always open in a separate window
    #[serde(default)]
    pub auto_detach: bool,
    /// Lifecycle file for view plugins (optional)
    /// For view plugins (HTML entry), this file will be executed on load
    /// to register lifecycle hooks, settings, and commands
    /// Default: "lifecycle.js" if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    /// Development mode flag
    /// If true, plugin will be loaded from devServer instead of local files
    #[serde(default, rename = "devMode")]
    pub dev_mode: bool,
    /// Development server URL
    /// Used when devMode is true (e.g., "http://localhost:5172")
    #[serde(skip_serializing_if = "Option::is_none", rename = "devServer")]
    pub dev_server: Option<String>,
}

fn default_display_mode() -> String {
    "inline".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingOption {
    pub label: String,
    pub value: JsonValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "defaultValue")]
    pub default_value: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SettingOption>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxLength")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "minLength")]
    pub min_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "buttonText")]
    pub button_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginSettingsSchema {
    pub fields: Vec<SettingField>,
}

/// 插件安装来源
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstallSource {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "marketplace")]
    Marketplace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadedPlugin {
    #[serde(flatten)]
    pub manifest: PluginManifest,
    pub dir_name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Dynamically registered settings schema (not from manifest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<PluginSettingsSchema>,
    /// Installation source
    #[serde(default = "default_install_source")]
    pub install_source: InstallSource,
}

fn default_enabled() -> bool {
    true
}

fn default_install_source() -> InstallSource {
    InstallSource::Local
}

/// 从目录名解析插件 ID 和安装来源
/// 例如：
/// - "translate@local" -> ("translate", InstallSource::Local)
/// - "translate@market" -> ("translate", InstallSource::Marketplace)
/// - "translate" -> ("translate", InstallSource::Local) // 兼容旧版本
fn parse_plugin_dir_name(dir_name: &str) -> (String, InstallSource) {
    if let Some(at_pos) = dir_name.rfind('@') {
        let plugin_id = dir_name[..at_pos].to_string();
        let suffix = &dir_name[at_pos + 1..];
        
        let source = match suffix {
            "market" | "marketplace" => InstallSource::Marketplace,
            "local" => InstallSource::Local,
            _ => InstallSource::Local, // 未知后缀默认为 Local
        };
        
        (plugin_id, source)
    } else {
        // 没有后缀，默认为 Local（兼容旧版本）
        (dir_name.to_string(), InstallSource::Local)
    }
}

/// 生成带后缀的目录名
fn make_plugin_dir_name(plugin_id: &str, source: InstallSource) -> String {
    match source {
        InstallSource::Local => format!("{}@local", plugin_id),
        InstallSource::Marketplace => format!("{}@market", plugin_id),
    }
}

/// 通过 plugin_id 查找插件（返回第一个匹配的）
fn find_plugin_by_id<'a>(
    store: &'a HashMap<String, LoadedPlugin>,
    plugin_id: &str,
) -> Option<&'a LoadedPlugin> {
    // 先尝试直接匹配 dir_name（兼容旧版本）
    if let Some(plugin) = store.get(plugin_id) {
        return Some(plugin);
    }
    
    // 查找 manifest.id 匹配的插件
    store.values().find(|p| p.manifest.id == plugin_id)
}

/// 通过 plugin_id 查找插件（可变引用）
fn find_plugin_by_id_mut<'a>(
    store: &'a mut HashMap<String, LoadedPlugin>,
    plugin_id: &str,
) -> Option<&'a mut LoadedPlugin> {
    // 先尝试直接匹配 dir_name
    if store.contains_key(plugin_id) {
        return store.get_mut(plugin_id);
    }
    
    // 查找 manifest.id 匹配的插件
    store.values_mut().find(|p| p.manifest.id == plugin_id)
}

/// 获取同一 plugin_id 的所有版本
fn find_all_versions(
    store: &HashMap<String, LoadedPlugin>,
    plugin_id: &str,
) -> Vec<String> {
    store
        .iter()
        .filter(|(_, p)| p.manifest.id == plugin_id)
        .map(|(dir_name, _)| dir_name.clone())
        .collect()
}

// 插件状态持久化结构
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PluginState {
    pub enabled: bool,
    #[serde(default)]
    pub auto_detach: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PluginStates {
    states: HashMap<String, PluginState>, // plugin_id -> state
}

// 窗口位置和大小持久化结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub is_maximized: bool,
}

// 默认窗口尺寸常量
const DEFAULT_WINDOW_WIDTH: u32 = 1000;
const DEFAULT_WINDOW_HEIGHT: u32 = 700;
const MIN_WINDOW_WIDTH: u32 = 400;
const MIN_WINDOW_HEIGHT: u32 = 300;
const MAX_WINDOW_WIDTH: u32 = 4096;  // 支持 4K 显示器
const MAX_WINDOW_HEIGHT: u32 = 2160;

impl WindowBounds {
    /// 验证并修正窗口尺寸，确保在合理范围内
    fn validate_and_fix(&mut self) {
        // 验证宽度
        if self.width < MIN_WINDOW_WIDTH || self.width > MAX_WINDOW_WIDTH {
            eprintln!(
                "[plugin_manager] Invalid window width: {}, using default: {}",
                self.width, DEFAULT_WINDOW_WIDTH
            );
            self.width = DEFAULT_WINDOW_WIDTH;
        }
        
        // 验证高度
        if self.height < MIN_WINDOW_HEIGHT || self.height > MAX_WINDOW_HEIGHT {
            eprintln!(
                "[plugin_manager] Invalid window height: {}, using default: {}",
                self.height, DEFAULT_WINDOW_HEIGHT
            );
            self.height = DEFAULT_WINDOW_HEIGHT;
        }
        
        // 验证位置（确保窗口至少部分可见）
        // 允许负值，但不能太离谱
        const MAX_OFFSET: i32 = 10000;
        if self.x < -MAX_OFFSET || self.x > MAX_OFFSET {
            eprintln!(
                "[plugin_manager] Invalid window x position: {}, resetting to 100",
                self.x
            );
            self.x = 100;
        }
        
        if self.y < -MAX_OFFSET || self.y > MAX_OFFSET {
            eprintln!(
                "[plugin_manager] Invalid window y position: {}, resetting to 100",
                self.y
            );
            self.y = 100;
        }
    }
    
    /// 创建默认窗口边界
    fn default() -> Self {
        WindowBounds {
            x: 100,
            y: 100,
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            is_maximized: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PluginWindowStates {
    windows: HashMap<String, WindowBounds>, // plugin_id -> window bounds
}

// 获取插件状态文件路径
fn get_plugin_states_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("plugin_states.json"))
}

// 获取插件窗口状态文件路径
fn get_plugin_window_states_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_data_dir = data_dir.join("plugin_data");
    
    // 确保目录存在
    if !plugin_data_dir.exists() {
        std::fs::create_dir_all(&plugin_data_dir).map_err(|e| e.to_string())?;
    }
    
    Ok(plugin_data_dir.join("window_states.json"))
}

// 获取插件设置文件路径
fn get_plugin_settings_path(
    app: &tauri::AppHandle,
    plugin_id: &str,
) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings_dir = data_dir.join("plugin_settings");

    // 确保目录存在
    if !settings_dir.exists() {
        std::fs::create_dir_all(&settings_dir).map_err(|e| e.to_string())?;
    }

    Ok(settings_dir.join(format!("{}.json", plugin_id)))
}

// 加载插件状态
fn load_plugin_states(app: &tauri::AppHandle) -> HashMap<String, PluginState> {
    match get_plugin_states_path(app) {
        Ok(path) => {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<PluginStates>(&content) {
                        Ok(plugin_states) => {
                            println!(
                                "[plugin_manager] Loaded plugin states: {:?}",
                                plugin_states.states
                            );
                            return plugin_states.states;
                        }
                        Err(e) => {
                            eprintln!("[plugin_manager] Failed to parse plugin states: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[plugin_manager] Failed to read plugin states file: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[plugin_manager] Failed to get plugin states path: {}", e);
        }
    }
    HashMap::new()
}

// 保存插件状态
fn save_plugin_states(
    app: &tauri::AppHandle,
    states: &HashMap<String, PluginState>,
) -> Result<(), String> {
    let path = get_plugin_states_path(app)?;
    let plugin_states = PluginStates {
        states: states.clone(),
    };
    let content = serde_json::to_string_pretty(&plugin_states).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

// 加载插件窗口状态（静默模式，减少日志输出）
fn load_plugin_window_states(app: &tauri::AppHandle) -> HashMap<String, WindowBounds> {
    match get_plugin_window_states_path(app) {
        Ok(path) => {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<PluginWindowStates>(&content) {
                        Ok(mut window_states) => {
                            // 验证并修正所有窗口的尺寸
                            for bounds in window_states.windows.values_mut() {
                                bounds.validate_and_fix();
                            }
                            return window_states.windows;
                        }
                        Err(e) => {
                            eprintln!("[plugin_manager] Failed to parse plugin window states: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[plugin_manager] Failed to read plugin window states file: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[plugin_manager] Failed to get plugin window states path: {}", e);
        }
    }
    HashMap::new()
}

// 保存插件窗口状态（静默模式，减少日志输出）
fn save_plugin_window_states(
    app: &tauri::AppHandle,
    states: &HashMap<String, WindowBounds>,
) -> Result<(), String> {
    let path = get_plugin_window_states_path(app)?;
    
    // 验证所有窗口尺寸
    let mut validated_states = states.clone();
    for bounds in validated_states.values_mut() {
        bounds.validate_and_fix();
    }
    
    let window_states = PluginWindowStates {
        windows: validated_states,
    };
    let content = serde_json::to_string_pretty(&window_states).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

// 用于防抖窗口状态保存
pub struct PluginWindowStateSaveDebounce(pub Mutex<HashMap<String, (std::time::Instant, WindowBounds)>>);

// 保存单个插件窗口状态（带防抖和验证）
fn save_plugin_window_state(
    app: &tauri::AppHandle,
    plugin_id: &str,
    mut bounds: WindowBounds,
) -> Result<(), String> {
    // 验证并修正窗口尺寸
    bounds.validate_and_fix();
    
    // macOS 窗口事件触发频率更高，使用更长的防抖时间
    #[cfg(target_os = "macos")]
    const DEBOUNCE_MS: u64 = 1000; // macOS: 1000ms 防抖时间
    
    #[cfg(not(target_os = "macos"))]
    const DEBOUNCE_MS: u64 = 500; // 其他平台: 500ms 防抖时间
    
    // 使用防抖机制，避免频繁保存
    if let Some(debounce_state) = app.try_state::<PluginWindowStateSaveDebounce>() {
        let mut debounce_map = debounce_state.0.lock().unwrap();
        let now = std::time::Instant::now();
        
        // 更新防抖映射
        debounce_map.insert(plugin_id.to_string(), (now, bounds.clone()));
        drop(debounce_map);
        
        // 启动延迟保存任务
        let app_clone = app.clone();
        let plugin_id_clone = plugin_id.to_string();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(DEBOUNCE_MS)).await;
            
            // 检查是否仍然是最新的状态
            if let Some(debounce_state) = app_clone.try_state::<PluginWindowStateSaveDebounce>() {
                let mut debounce_map = debounce_state.0.lock().unwrap();
                
                if let Some((timestamp, bounds)) = debounce_map.get(&plugin_id_clone) {
                    let elapsed = std::time::Instant::now().duration_since(*timestamp).as_millis() as u64;
                    
                    // 如果已经过了防抖时间，执行保存
                    if elapsed >= DEBOUNCE_MS {
                        let bounds_to_save = bounds.clone();
                        debounce_map.remove(&plugin_id_clone);
                        drop(debounce_map);
                        
                        // 执行实际的保存操作
                        let mut states = load_plugin_window_states(&app_clone);
                        states.insert(plugin_id_clone.clone(), bounds_to_save);
                        if let Err(e) = save_plugin_window_states(&app_clone, &states) {
                            eprintln!("[plugin_manager] Failed to save window state: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    } else {
        // 如果没有防抖状态，直接保存（向后兼容）
        let mut states = load_plugin_window_states(app);
        states.insert(plugin_id.to_string(), bounds);
        save_plugin_window_states(app, &states)
    }
}

// 获取系统保护路径（使用系统 API 动态获取）
fn get_system_protected_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use std::env;

        // 获取 Windows 系统目录
        if let Ok(windows_dir) = env::var("SystemRoot") {
            paths.push(std::path::PathBuf::from(windows_dir));
        }

        // 获取 Program Files 目录
        if let Ok(program_files) = env::var("ProgramFiles") {
            paths.push(std::path::PathBuf::from(program_files));
        }

        // 获取 Program Files (x86) 目录
        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            paths.push(std::path::PathBuf::from(program_files_x86));
        }

        // 获取 System32 目录
        if let Ok(system_root) = env::var("SystemRoot") {
            paths.push(std::path::PathBuf::from(system_root).join("System32"));
        }

        // 获取 Windows 驱动目录
        if let Ok(system_root) = env::var("SystemRoot") {
            paths.push(std::path::PathBuf::from(system_root).join("SysWOW64"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 系统目录
        paths.push(std::path::PathBuf::from("/System"));
        paths.push(std::path::PathBuf::from("/Library"));
        paths.push(std::path::PathBuf::from("/Applications"));
        paths.push(std::path::PathBuf::from("/usr"));
        paths.push(std::path::PathBuf::from("/bin"));
        paths.push(std::path::PathBuf::from("/sbin"));
        paths.push(std::path::PathBuf::from("/var"));
        paths.push(std::path::PathBuf::from("/private"));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 系统目录
        paths.push(std::path::PathBuf::from("/usr"));
        paths.push(std::path::PathBuf::from("/bin"));
        paths.push(std::path::PathBuf::from("/sbin"));
        paths.push(std::path::PathBuf::from("/lib"));
        paths.push(std::path::PathBuf::from("/lib64"));
        paths.push(std::path::PathBuf::from("/boot"));
        paths.push(std::path::PathBuf::from("/sys"));
        paths.push(std::path::PathBuf::from("/proc"));
        paths.push(std::path::PathBuf::from("/dev"));
        paths.push(std::path::PathBuf::from("/etc"));
        paths.push(std::path::PathBuf::from("/var"));
        paths.push(std::path::PathBuf::from("/root"));
    }

    // 规范化所有路径（如果可能）
    paths
        .into_iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect()
}

fn load_plugins_internal(
    app: &tauri::AppHandle,
    store: &State<PluginStore>,
    clear_existing: bool,
) -> Result<Vec<LoadedPlugin>, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");

    if !plugins_dir.is_dir() {
        return Ok(Vec::new());
    }

    let plugin_states = load_plugin_states(app);

    let mut store_lock = store.0.lock().unwrap();
    if clear_existing {
        store_lock.clear();
    }

    let mut plugins_to_init = Vec::new();

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

                let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
                
                // 解析目录名，提取插件 ID 和安装来源
                let (_parsed_id, install_source) = parse_plugin_dir_name(&dir_name);

                // 从持久化状态中获取启用状态和 auto_detach，如果没有则使用默认值
                let (enabled, auto_detach) = if let Some(state) = plugin_states.get(&manifest.id) {
                    (state.enabled, state.auto_detach)
                } else {
                    (true, manifest.auto_detach)
                };

                let mut manifest_with_state = manifest.clone();
                manifest_with_state.auto_detach = auto_detach;

                // 自动执行生命周期文件进行初始化
                // Headless 插件：执行 index.js (entry)
                // View 插件：执行 lifecycle.js（如果存在）
                let entry_path = path.join(&manifest.entry);

                if entry_path.is_file() {
                    if let Some(extension) = Path::new(&manifest.entry)
                        .extension()
                        .and_then(|s| s.to_str())
                    {
                        match extension {
                            "js" => {
                                // Headless 插件：直接执行 index.js
                                plugins_to_init.push((
                                    manifest.id.clone(),
                                    entry_path,
                                    dir_name.clone(),
                                ));
                            }
                            "html" => {
                                // View 插件：查找并执行 lifecycle.js
                                let lifecycle_file = manifest
                                    .lifecycle
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("lifecycle.js");
                                let lifecycle_path = path.join(lifecycle_file);

                                if lifecycle_path.is_file() {
                                    plugins_to_init.push((
                                        manifest.id.clone(),
                                        lifecycle_path,
                                        dir_name.clone(),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                let loaded_plugin = LoadedPlugin {
                    manifest: manifest_with_state,
                    dir_name: dir_name.clone(),
                    enabled,
                    settings: None,
                    install_source: install_source.clone(),
                };

                // 使用 dir_name 作为 key，这样同一插件的不同版本可以共存
                store_lock.insert(dir_name.clone(), loaded_plugin);
            }
        }
    }

    let plugins = store_lock.values().cloned().collect();
    drop(store_lock);

    // 执行所有插件的初始化脚本
    if !plugins_to_init.is_empty() {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                for (plugin_id, entry_path, _dir_name) in plugins_to_init {
                    if let Ok(js_code) = std::fs::read_to_string(&entry_path) {
                        let _ =
                            js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await;
                    }
                }
            });
        });
    }

    Ok(plugins)
}

#[tauri::command]
pub fn load_plugins(
    app: tauri::AppHandle,
    store: State<PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    load_plugins_internal(&app, &store, true)
}

#[tauri::command]
pub fn get_loaded_plugins(store: State<PluginStore>) -> Result<Vec<LoadedPlugin>, String> {
    let store_lock = store.0.lock().unwrap();
    let plugins = store_lock.values().cloned().collect();
    Ok(plugins)
}

#[tauri::command]
pub async fn refresh_plugins(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    // 清除 JavaScript 运行时缓存
    let _ = crate::js_runtime::clear_all_plugin_runtimes().await;

    // 清除插件加载状态缓存
    let loaded_state = app.state::<crate::plugin_api::command::PluginLoadedState>();
    loaded_state.0.lock().unwrap().clear();

    // 重新加载所有插件
    load_plugins_internal(&app, &store, true)
}

#[tauri::command]
pub fn toggle_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,  // 这里是 dir_name（包含后缀）
    enabled: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    // 查找插件
    let plugin = store_lock.get_mut(&plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
    
    let manifest_id = plugin.manifest.id.clone();
    
    // 如果要启用这个插件，需要禁用同一 manifest.id 的其他版本
    if enabled {
        let other_versions: Vec<String> = store_lock
            .iter()
            .filter(|(dir_name, p)| {
                p.manifest.id == manifest_id && *dir_name != &plugin_id && p.enabled
            })
            .map(|(dir_name, _)| dir_name.clone())
            .collect();
        
        for other_dir_name in other_versions {
            if let Some(other_plugin) = store_lock.get_mut(&other_dir_name) {
                other_plugin.enabled = false;
                println!(
                    "[plugin_manager] Auto-disabled {} version of plugin {}",
                    match other_plugin.install_source {
                        InstallSource::Local => "local",
                        InstallSource::Marketplace => "market",
                    },
                    manifest_id
                );
            }
        }
    }
    
    // 启用/禁用当前插件
    let plugin = store_lock.get_mut(&plugin_id).unwrap();
    plugin.enabled = enabled;
    println!(
        "[plugin_manager] Plugin {} ({}) is now {}",
        manifest_id,
        plugin_id,
        if enabled { "enabled" } else { "disabled" }
    );

    // 收集所有插件的状态（使用 manifest.id 作为 key）
    let mut states = HashMap::new();
    for (_, plugin) in store_lock.iter() {
        // 对于同一 manifest.id 的多个版本，只保存启用的那个
        if plugin.enabled || !states.contains_key(&plugin.manifest.id) {
            states.insert(
                plugin.manifest.id.clone(),
                PluginState {
                    enabled: plugin.enabled,
                    auto_detach: plugin.manifest.auto_detach,
                },
            );
        }
    }

    // 释放锁后再保存状态
    drop(store_lock);

    // 持久化保存状态
    if let Err(e) = save_plugin_states(&app, &states) {
        eprintln!("[plugin_manager] Failed to save plugin states: {}", e);
        return Err(format!("Failed to save plugin state: {}", e));
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_plugin_auto_detach(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    auto_detach: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = find_plugin_by_id_mut(&mut store_lock, &plugin_id) {
        plugin.manifest.auto_detach = auto_detach;
        println!(
            "[plugin_manager] Plugin {} auto_detach is now {}",
            plugin_id, auto_detach
        );

        // 收集所有插件的状态（使用 manifest.id 作为 key）
        let mut states = HashMap::new();
        for (_, plugin) in store_lock.iter() {
            // 对于同一 manifest.id 的多个版本，只保存启用的那个
            if plugin.enabled || !states.contains_key(&plugin.manifest.id) {
                states.insert(
                    plugin.manifest.id.clone(),
                    PluginState {
                        enabled: plugin.enabled,
                        auto_detach: plugin.manifest.auto_detach,
                    },
                );
            }
        }

        // 释放锁后再保存状态
        drop(store_lock);

        // 持久化保存状态
        if let Err(e) = save_plugin_states(&app, &states) {
            eprintln!("[plugin_manager] Failed to save plugin states: {}", e);
            return Err(format!("Failed to save plugin state: {}", e));
        }

        Ok(())
    } else {
        Err(format!("Plugin not found: {}", plugin_id))
    }
}

#[tauri::command]
pub fn register_plugin_settings_schema(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    schema: PluginSettingsSchema,
) -> Result<(), String> {
    println!(
        "[plugin_manager] register_plugin_settings_schema called for: {}",
        plugin_id
    );
    println!(
        "[plugin_manager] Schema fields count: {}",
        schema.fields.len()
    );

    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = find_plugin_by_id_mut(&mut store_lock, &plugin_id) {
        // Store settings schema in LoadedPlugin (not in manifest)
        plugin.settings = Some(schema.clone());
        println!(
            "[plugin_manager] ✅ Registered settings schema for plugin {}: {} fields",
            plugin_id,
            schema.fields.len()
        );

        // Emit event to notify frontend that schema has been registered
        if let Err(e) = app.emit("plugin-settings-schema-registered", &plugin_id) {
            eprintln!(
                "[plugin_manager] Failed to emit schema registered event: {}",
                e
            );
            return Err(format!("Failed to emit event: {}", e));
        }

        Ok(())
    } else {
        Err(format!("Plugin not found: {}", plugin_id))
    }
}

#[tauri::command]
pub fn get_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
) -> Result<HashMap<String, JsonValue>, String> {
    let settings_path = get_plugin_settings_path(&app, &plugin_id)?;

    if !settings_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: HashMap<String, JsonValue> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    println!(
        "[plugin_manager] Loaded settings for plugin {}: {:?}",
        plugin_id, settings
    );
    Ok(settings)
}

#[tauri::command]
pub fn save_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
    settings: HashMap<String, JsonValue>,
) -> Result<(), String> {
    let settings_path = get_plugin_settings_path(&app, &plugin_id)?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    println!(
        "[plugin_manager] Saved settings for plugin {}: {:?}",
        plugin_id, settings
    );
    Ok(())
}

#[tauri::command]
pub fn get_plugin_with_schema(
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<LoadedPlugin, String> {
    let store_lock = store.0.lock().unwrap();

    find_plugin_by_id(&store_lock, &plugin_id)
        .cloned()
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))
}

/// 插件详情响应（包含 README）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginDetail {
    #[serde(flatten)]
    pub plugin: LoadedPlugin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
}

#[tauri::command]
pub fn get_plugin_detail(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<PluginDetail, String> {
    let store_lock = store.0.lock().unwrap();

    let plugin = find_plugin_by_id(&store_lock, &plugin_id)
        .cloned()
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    // 读取 README.md
    let plugins_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("plugins");
    
    let plugin_dir = plugins_dir.join(&plugin.dir_name);
    let readme_path = plugin_dir.join("README.md");
    
    let readme = if readme_path.exists() {
        std::fs::read_to_string(&readme_path)
            .ok()
    } else {
        None
    };

    Ok(PluginDetail {
        plugin,
        readme,
    })
}

#[tauri::command]
pub fn get_plugin_server_port(app: tauri::AppHandle) -> Result<u16, String> {
    let server_port_state = app.state::<PluginServerPort>();
    let port = server_port_state
        .0
        .lock()
        .unwrap()
        .ok_or_else(|| "Plugin server not started".to_string())?;
    Ok(port)
}

// 插件窗口控制命令
#[tauri::command]
pub fn plugin_close_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_minimize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.minimize().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_maximize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.maximize().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_unmaximize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_is_maximized(app: tauri::AppHandle, label: String) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.is_maximized().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_show_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_set_focus(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|e| e.to_string())
    } else {
        Err(format!("Window not found: {}", label))
    }
}

#[tauri::command]
pub fn plugin_start_dragging(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_plugin_in_window(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    // Clone plugin data to release the lock ASAP
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        find_plugin_by_id(&store_lock, &plugin_id).cloned()
    }
    .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("Plugin entry file not found: {:?}", entry_path));
    }

    // Force open in window mode
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = create_or_show_plugin_window(app_clone, &plugin).await {
            eprintln!("Failed to create or show plugin window: {}", e);
        }
    });

    Ok(())
}

/// 辅助函数：触发窗口可见性事件
fn trigger_window_visibility_event(window: &tauri::WebviewWindow, is_visible: bool) {
    let eval_script = format!(
        r#"
        if (window.__TAURI__?.event?._trigger) {{
            window.__TAURI__.event._trigger('window_visibility', {});
        }}
        "#,
        is_visible
    );
    if let Err(e) = window.eval(&eval_script) {
        eprintln!(
            "Failed to trigger window_visibility ({}): {}",
            is_visible, e
        );
    }
}

/// 创建或显示插件窗口的共享函数
async fn create_or_show_plugin_window(
    app: tauri::AppHandle,
    plugin: &LoadedPlugin,
) -> Result<(), String> {
    let window_label = format!("plugin_{}", plugin.manifest.id.replace('.', "_"));

    // 防抖检查：防止短时间内重复触发
    const DEBOUNCE_MS: u64 = 100; // 100ms 防抖时间
    if let Some(debounce_state) = app.try_state::<PluginWindowToggleDebounce>() {
        let mut debounce_map = debounce_state.0.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some(last_toggle) = debounce_map.get(&window_label) {
            let elapsed = now.duration_since(*last_toggle).as_millis() as u64;
            if elapsed < DEBOUNCE_MS {
                println!(
                    "[plugin_manager] Window {} toggle debounced ({}ms since last toggle)",
                    window_label, elapsed
                );
                return Ok(());
            }
        }

        // 更新最后切换时间
        debounce_map.insert(window_label.clone(), now);
    }

    // 检查窗口是否正在创建中
    if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
        let creating = creating_state.0.lock().unwrap();
        if creating.contains(&window_label) {
            println!(
                "[plugin_manager] Window {} is already being created, skipping",
                window_label
            );
            return Ok(());
        }
    }

    // 如果窗口已存在，切换显示状态
    if let Some(window) = app.get_webview_window(&window_label) {
        // 检查窗口是否被最小化
        let is_minimized = window.is_minimized().unwrap_or(false);
        // 检查窗口是否可见
        let is_visible = window.is_visible().unwrap_or(false);

        println!(
            "[plugin_manager] Window {} state - minimized: {}, visible: {}",
            window_label, is_minimized, is_visible
        );

        if is_minimized || !is_visible {
            // 窗口被最小化或隐藏，显示并聚焦
            println!("[plugin_manager] Showing window {}", window_label);
            if is_minimized {
                if let Err(e) = window.unminimize() {
                    eprintln!("Failed to unminimize plugin window: {}", e);
                }
            }
            if let Err(e) = window.show() {
                eprintln!("Failed to show plugin window: {}", e);
            }
            if let Err(e) = window.set_focus() {
                eprintln!("Failed to focus plugin window: {}", e);
            }

            // 发送窗口可见性事件到前端
            trigger_window_visibility_event(&window, true);
        } else {
            // 窗口已显示，最小化它
            println!("[plugin_manager] Minimizing window {}", window_label);
            if let Err(e) = window.minimize() {
                eprintln!("Failed to minimize plugin window: {}", e);
            }

            // 发送窗口可见性事件到前端
            trigger_window_visibility_event(&window, false);
        }
        return Ok(());
    }

    // 使用 Svelte 页面加载插件窗口，页面内部会用 iframe 加载插件
    let plugin_url = format!(
        "http://localhost:1420/plugin-window?plugin_id={}",
        plugin.manifest.id
    );
    println!(
        "[plugin_manager] Loading plugin window from: {}",
        plugin_url
    );

    // 如果是开发模式，打印开发服务器信息
    if plugin.manifest.dev_mode {
        if let Some(dev_server) = &plugin.manifest.dev_server {
            println!(
                "[plugin_manager] Plugin {} is in dev mode, will load from: {}",
                plugin.manifest.id, dev_server
            );
        } else {
            eprintln!(
                "[plugin_manager] Warning: Plugin {} has devMode=true but no devServer specified",
                plugin.manifest.id
            );
        }
    }

    // 加载保存的窗口状态
    let window_states = load_plugin_window_states(&app);
    let saved_bounds = window_states.get(&plugin.manifest.id).cloned();
    
    // 获取窗口边界（使用保存的值或默认值）
    let bounds = saved_bounds.unwrap_or_else(WindowBounds::default);
    
    println!(
        "[plugin_manager] Creating window for {}: x={}, y={}, width={}, height={}, maximized={}",
        plugin.manifest.id, bounds.x, bounds.y, bounds.width, bounds.height, bounds.is_maximized
    );
    
    // 创建窗口构建器
    let mut builder = WebviewWindowBuilder::new(
        &app,
        window_label.clone(),
        tauri::WebviewUrl::External(plugin_url.parse().unwrap()),
    )
    .title(plugin.manifest.name.clone())
    .resizable(true)
    .decorations(false) // 所有平台都隐藏系统装饰
    .transparent(false) // 确保窗口不透明
    .position(bounds.x as f64, bounds.y as f64)
    .inner_size(bounds.width as f64, bounds.height as f64);

    // 标记窗口正在创建
    if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
        let mut creating = creating_state.0.lock().unwrap();
        creating.insert(window_label.clone());
    }

    match builder.build() {
        Ok(window) => {
            // 窗口创建成功，移除创建标记
            if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
                let mut creating = creating_state.0.lock().unwrap();
                creating.remove(&window_label);
            }
            
            // 如果之前窗口是最大化的，恢复最大化状态
            if bounds.is_maximized {
                println!("[plugin_manager] Restoring maximized state for {}", plugin.manifest.id);
                if let Err(e) = window.maximize() {
                    eprintln!("Failed to maximize window: {}", e);
                }
            }

            // 监听窗口事件，用于注册/注销 ESC 快捷键
            use std::str::FromStr;
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

            let esc_shortcut = Shortcut::from_str("escape").unwrap();
            let app_for_window_event = app.clone();
            let window_label_for_tracking = window.label().to_string();

            // 设置窗口焦点
            if let Err(e) = window.set_focus() {
                eprintln!("Failed to set focus on plugin window: {}", e);
            }

            // 立即记录活跃窗口并注册 ESC 快捷键
            let app_for_immediate = app.clone();
            let label_for_immediate = window_label_for_tracking.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                // 记录活跃窗口
                if let Some(active_window_state) =
                    app_for_immediate.try_state::<ActivePluginWindow>()
                {
                    if let Ok(mut active) = active_window_state.0.lock() {
                        *active = Some(label_for_immediate.clone());
                        println!(
                            "[plugin_manager] Set active plugin window: {}",
                            label_for_immediate
                        );
                    }
                }

                // 注册 ESC 快捷键
                let _ = app_for_immediate.global_shortcut().unregister(esc_shortcut);
                if let Err(e) = app_for_immediate.global_shortcut().register(esc_shortcut) {
                    eprintln!("[plugin_manager] Failed to register ESC shortcut: {}", e);
                }
            });

            let label_for_event = window_label_for_tracking.clone();
            let window_for_event = window.clone();
            let plugin_id_for_event = plugin.manifest.id.clone();
            let app_for_save = app.clone();

            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        // 记录活跃窗口
                        if let Some(active_window_state) =
                            app_for_window_event.try_state::<ActivePluginWindow>()
                        {
                            if let Ok(mut active) = active_window_state.0.lock() {
                                *active = Some(label_for_event.clone());
                            }
                        }

                        // 发送窗口可见性事件到前端
                        if let Err(e) = window_for_event.emit("window_visibility", true) {
                            eprintln!(
                                "[plugin_manager] Failed to emit window_visibility event: {}",
                                e
                            );
                        }

                        // 重新注册 ESC 快捷键
                        let _ = app_for_window_event
                            .global_shortcut()
                            .unregister(esc_shortcut);
                        if let Err(e) = app_for_window_event
                            .global_shortcut()
                            .register(esc_shortcut)
                        {
                            eprintln!("[plugin_manager] Failed to register ESC shortcut: {}", e);
                        }
                    }
                    tauri::WindowEvent::Focused(false) => {
                        // 清除活跃窗口记录
                        if let Some(active_window_state) =
                            app_for_window_event.try_state::<ActivePluginWindow>()
                        {
                            if let Ok(mut active) = active_window_state.0.lock() {
                                if active.as_ref() == Some(&label_for_event) {
                                    *active = None;
                                }
                            }
                        }

                        // 发送窗口可见性事件到前端
                        if let Err(e) = window_for_event.emit("window_visibility", false) {
                            eprintln!(
                                "[plugin_manager] Failed to emit window_visibility event: {}",
                                e
                            );
                        }

                        // 注销 ESC 快捷键
                        if let Err(e) = app_for_window_event
                            .global_shortcut()
                            .unregister(esc_shortcut)
                        {
                            eprintln!("Failed to unregister ESC shortcut: {}", e);
                        }
                    }
                    tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                        // 统一处理窗口移动和调整大小事件
                        // 使用一致的方式获取位置和尺寸
                        if let Ok(position) = window_for_event.outer_position() {
                            if let Ok(size) = window_for_event.inner_size() {
                                if let Ok(is_maximized) = window_for_event.is_maximized() {
                                    let bounds = WindowBounds {
                                        x: position.x,
                                        y: position.y,
                                        width: size.width,
                                        height: size.height,
                                        is_maximized,
                                    };
                                    // 使用防抖保存，避免频繁 I/O
                                    let _ = save_plugin_window_state(&app_for_save, &plugin_id_for_event, bounds);
                                }
                            }
                        }
                    }
                    tauri::WindowEvent::CloseRequested { .. } => {
                        // 在关闭前保存最终的窗口状态
                        if let Ok(position) = window_for_event.outer_position() {
                            if let Ok(size) = window_for_event.inner_size() {
                                if let Ok(is_maximized) = window_for_event.is_maximized() {
                                    let bounds = WindowBounds {
                                        x: position.x,
                                        y: position.y,
                                        width: size.width,
                                        height: size.height,
                                        is_maximized,
                                    };
                                    if let Err(e) = save_plugin_window_state(&app_for_save, &plugin_id_for_event, bounds) {
                                        eprintln!("[plugin_manager] Failed to save window state on close: {}", e);
                                    }
                                }
                            }
                        }
                        
                        // 清除活跃窗口记录
                        if let Some(active_window_state) =
                            app_for_window_event.try_state::<ActivePluginWindow>()
                        {
                            if let Ok(mut active) = active_window_state.0.lock() {
                                if active.as_ref() == Some(&label_for_event) {
                                    *active = None;
                                }
                            }
                        }

                        let _ = app_for_window_event
                            .global_shortcut()
                            .unregister(esc_shortcut);
                    }
                    _ => {}
                }
            });

            Ok(())
        }
        Err(e) => {
            // 窗口创建失败，移除创建标记
            if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
                let mut creating = creating_state.0.lock().unwrap();
                creating.remove(&window_label);
            }
            eprintln!("Failed to build plugin window: {}", e);
            Err(format!("Failed to build plugin window: {}", e))
        }
    }
}

#[tauri::command]
pub fn execute_plugin_entry(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    println!(
        "[plugin_manager] execute_plugin_entry called for: {}",
        plugin_id
    );

    // Clone plugin data to release the lock ASAP
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        find_plugin_by_id(&store_lock, &plugin_id).cloned()
    }
    .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

    println!(
        "[plugin_manager] Plugin found: {}, enabled: {}",
        plugin.manifest.name, plugin.enabled
    );

    // 调试：打印开发模式信息
    println!(
        "[plugin_manager] Plugin dev_mode: {}, dev_server: {:?}",
        plugin.manifest.dev_mode, plugin.manifest.dev_server
    );

    // 检查插件是否启用
    if !plugin.enabled {
        return Err("Plugin is disabled".to_string());
    }

    // 开发模式下，直接使用开发服务器，不需要检查本地文件
    if plugin.manifest.dev_mode {
        if let Some(dev_server) = &plugin.manifest.dev_server {
            println!(
                "[plugin_manager] Plugin {} is in dev mode, loading from: {}",
                plugin.manifest.id, dev_server
            );

            // 开发模式下，假设是 HTML 插件（webview）
            // 检查是否应该在窗口中打开
            let should_open_in_window =
                plugin.manifest.auto_detach || plugin.manifest.display_mode.as_str() == "window";

            if should_open_in_window {
                // 在独立窗口中打开
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = create_or_show_plugin_window(app_clone, &plugin).await {
                        eprintln!("Failed to create or show plugin window: {}", e);
                    }
                });
                return Ok(());
            } else {
                // 内联模式
                let main_window = app
                    .get_webview_window("main")
                    .ok_or_else(|| "Main window not found".to_string())?;

                // 显示主窗口
                if let Ok(false) = main_window.is_visible() {
                    if let Err(e) = main_window.show() {
                        eprintln!(
                            "[plugin_manager] Warning: Failed to show main window: {}",
                            e
                        );
                    }
                    if let Err(e) = main_window.set_focus() {
                        eprintln!(
                            "[plugin_manager] Warning: Failed to focus main window: {}",
                            e
                        );
                    }
                }

                #[derive(Serialize, Clone)]
                struct PluginInlinePayload {
                    plugin_id: String,
                    plugin_name: String,
                    plugin_url: String,
                }

                let payload = PluginInlinePayload {
                    plugin_id: plugin.manifest.id.clone(),
                    plugin_name: plugin.manifest.name.clone(),
                    plugin_url: dev_server.clone(),
                };

                main_window
                    .emit("show_plugin_inline", payload)
                    .map_err(|e| format!("Failed to emit show_plugin_inline event: {}", e))?;

                return Ok(());
            }
        } else {
            return Err("Plugin has devMode=true but no devServer specified".to_string());
        }
    }

    // 生产模式：检查本地文件
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("Plugin entry file not found: {:?}", entry_path));
    }

    if let Some(extension) = Path::new(&plugin.manifest.entry)
        .extension()
        .and_then(|s| s.to_str())
    {
        match extension {
            "js" => {
                println!(
                    "[plugin_manager] Executing JS plugin: {}",
                    plugin.manifest.id
                );
                // Headless plugin, execute in the background
                let js_code = std::fs::read_to_string(entry_path).map_err(|e| e.to_string())?;
                let app_clone = app.clone();
                let plugin_id = plugin.manifest.id.clone();
                println!(
                    "[plugin_manager] Spawning JS execution thread for plugin: {}",
                    plugin_id
                );
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        if let Err(e) =
                            js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await
                        {
                            eprintln!("Failed to execute headless plugin: {}", e);
                        }
                    });
                });
                Ok(())
            }
            "html" => {
                // UI plugin - check auto_detach first, then display mode
                let should_open_in_window = plugin.manifest.auto_detach
                    || plugin.manifest.display_mode.as_str() == "window";

                println!(
                    "[plugin_manager] Plugin {} - auto_detach: {}, display_mode: {}, should_open_in_window: {}",
                    plugin.manifest.id, plugin.manifest.auto_detach, plugin.manifest.display_mode, should_open_in_window
                );

                if should_open_in_window {
                    // Open in new webview window
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = create_or_show_plugin_window(app_clone, &plugin).await {
                            eprintln!("Failed to create or show plugin window: {}", e);
                        }
                    });
                    Ok(())
                } else {
                    // Display inline - use HTTP server (生产模式)
                    let main_window = app
                        .get_webview_window("main")
                        .ok_or_else(|| "Main window not found".to_string())?;

                    // 特殊处理：当通过快捷键触发内联插件时，需要显示主窗口
                    if let Ok(false) = main_window.is_visible() {
                        if let Err(e) = main_window.show() {
                            eprintln!("[plugin_manager] Warning: Failed to show main window for inline plugin: {}", e);
                        }
                        if let Err(e) = main_window.set_focus() {
                            eprintln!("[plugin_manager] Warning: Failed to focus main window for inline plugin: {}", e);
                        }
                    }

                    #[derive(Serialize, Clone)]
                    struct PluginInlinePayload {
                        plugin_id: String,
                        plugin_name: String,
                        plugin_url: String,
                    }

                    // 获取插件服务器端口
                    let server_port_state = app.state::<PluginServerPort>();
                    let port = server_port_state
                        .0
                        .lock()
                        .unwrap()
                        .ok_or_else(|| "Plugin server not started".to_string())?;

                    let plugin_url = format!(
                        "http://127.0.0.1:{}/plugin/{}/{}",
                        port, plugin.dir_name, plugin.manifest.entry
                    );

                    let payload = PluginInlinePayload {
                        plugin_id: plugin.manifest.id.clone(),
                        plugin_name: plugin.manifest.name.clone(),
                        plugin_url,
                    };

                    main_window
                        .emit("show_plugin_inline", payload)
                        .map_err(|e| format!("Failed to emit show_plugin_inline event: {}", e))?;

                    Ok(())
                }
            }
            _ => Err(format!("Unsupported plugin entry type: {}", extension)),
        }
    } else {
        Err("Plugin entry file has no extension".to_string())
    }
}

/// Inline CSS and JS resources into HTML content
fn inline_resources(html_content: &str, html_dir: &std::path::Path) -> String {
    let mut modified_html = html_content.to_string();

    // Inline CSS files
    let css_regex =
        regex::Regex::new(r#"<link[^>]+href\s*=\s*["']([^"']+\.css)["'][^>]*>"#).unwrap();
    let css_matches: Vec<_> = css_regex.captures_iter(html_content).collect();

    for cap in css_matches {
        if let Some(css_path_match) = cap.get(1) {
            let css_path = css_path_match.as_str();
            let normalized_path = css_path.replace("/", std::path::MAIN_SEPARATOR_STR);
            let css_file_path = html_dir.join(
                normalized_path
                    .trim_start_matches("./")
                    .trim_start_matches(std::path::MAIN_SEPARATOR),
            );

            if let Ok(css_content) = std::fs::read_to_string(&css_file_path) {
                let inline_style = format!("<style>{}</style>", css_content);
                let original_tag = cap.get(0).unwrap().as_str();
                modified_html = modified_html.replace(original_tag, &inline_style);
            } else {
                eprintln!(
                    "[plugin_manager] Warning: Failed to read CSS file: {:?}",
                    css_file_path
                );
            }
        }
    }

    // Inline JS files
    let js_regex =
        regex::Regex::new(r#"<script[^>]+src\s*=\s*["']([^"']+\.js)["'][^>]*></script>"#).unwrap();
    let js_matches: Vec<_> = js_regex.captures_iter(html_content).collect();

    for cap in js_matches {
        if let Some(js_path_match) = cap.get(1) {
            let js_path = js_path_match.as_str();
            let normalized_path = js_path.replace("/", std::path::MAIN_SEPARATOR_STR);
            let js_file_path = html_dir.join(
                normalized_path
                    .trim_start_matches("./")
                    .trim_start_matches(std::path::MAIN_SEPARATOR),
            );

            if let Ok(js_content) = std::fs::read_to_string(&js_file_path) {
                let original_tag = cap.get(0).unwrap().as_str();
                let is_module = original_tag.contains("type=\"module\"")
                    || original_tag.contains("type='module'");

                let inline_script = if is_module {
                    format!("<script type=\"module\">{}</script>", js_content)
                } else {
                    format!("<script>{}</script>", js_content)
                };

                modified_html = modified_html.replace(original_tag, &inline_script);
            } else {
                eprintln!(
                    "[plugin_manager] Warning: Failed to read JS file: {:?}",
                    js_file_path
                );
            }
        }
    }

    modified_html
}

/// Inject Tauri API bridge script into HTML
fn inject_tauri_bridge(html: &str, plugin_id: &str) -> String {
    let tauri_init_script = format!(
        r#"
<script>
(function() {{
  console.log('[Plugin Inline] Initializing Tauri API bridge');
  
  // Set plugin ID in global context
  window.__PLUGIN_ID__ = '{}';
  
  const createProxy = (command) => {{
    return (...args) => {{
      return new Promise((resolve, reject) => {{
        const messageId = 'tauri_' + Math.random().toString(36).substring(7) + '_' + Date.now();
        
        const handleResponse = (event) => {{
          if (event.data && event.data.messageId === messageId) {{
            window.removeEventListener('message', handleResponse);
            if (event.data.error) {{
              reject(new Error(event.data.error));
            }} else {{
              resolve(event.data.result);
            }}
          }}
        }};
        
        window.addEventListener('message', handleResponse);
        
        window.parent.postMessage({{
          type: 'plugin-tauri-call',
          messageId,
          command,
          args
        }}, '*');
        
        setTimeout(() => {{
          window.removeEventListener('message', handleResponse);
          reject(new Error('Tauri call timeout'));
        }}, 30000);
      }});
    }};
  }};
  
  const invokeProxy = createProxy('invoke');
  
  window.__TAURI__ = {{
    core: {{ invoke: invokeProxy }},
    event: {{
      emit: createProxy('emit'),
      listen: createProxy('listen')
    }},
    invoke: invokeProxy
  }};
  
  window.__TAURI_INVOKE__ = invokeProxy;
  
  console.log('[Plugin Inline] Tauri API bridge ready');
}})();
</script>
"#,
        plugin_id
    );

    if html.contains("<head>") {
        html.replace("<head>", &format!("<head>{}", tauri_init_script))
    } else if html.contains("<html>") {
        html.replace(
            "<html>",
            &format!("<html><head>{}</head>", tauri_init_script),
        )
    } else {
        format!("{}{}", tauri_init_script, html)
    }
}

#[tauri::command]
pub fn import_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    source_path: String,
) -> Result<LoadedPlugin, String> {
    println!("[plugin_manager] Importing plugin from: {}", source_path);

    // 1. 验证源路径
    let source = std::path::PathBuf::from(&source_path);
    if !source.exists() || !source.is_dir() {
        return Err("Invalid plugin directory".to_string());
    }

    // 安全检查：规范化路径，防止路径遍历攻击
    let source = source
        .canonicalize()
        .map_err(|e| format!("Failed to resolve plugin path: {}", e))?;

    // 安全检查：禁止导入系统敏感目录
    let forbidden_paths = get_system_protected_paths();

    for forbidden in &forbidden_paths {
        if source.starts_with(forbidden) {
            return Err(format!(
                "Cannot import plugins from system directory: {:?}",
                forbidden
            ));
        }
    }

    // 2. 验证 manifest.json 是否存在
    let manifest_path = source.join("manifest.json");
    if !manifest_path.exists() {
        return Err("manifest.json not found in plugin directory".to_string());
    }

    // 3. 读取并解析 manifest
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    let manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Invalid manifest format: {}", e))?;

    println!(
        "[plugin_manager] Plugin manifest loaded: {} ({})",
        manifest.name, manifest.id
    );

    // 安全检查：验证插件 ID 格式（防止特殊字符导致路径问题）
    if manifest.id.contains("..") || manifest.id.contains("/") || manifest.id.contains("\\") {
        return Err("Invalid plugin ID: contains illegal characters".to_string());
    }

    // 检查插件是否已存在
    let (plugin_exists, existing_plugin_name) = {
        let store_lock = store.0.lock().unwrap();
        if let Some(existing) = find_plugin_by_id(&store_lock, &manifest.id) {
            (true, Some(existing.manifest.name.clone()))
        } else {
            (false, None)
        }
    };

    if plugin_exists {
        let plugin_name = existing_plugin_name.unwrap_or_else(|| manifest.id.clone());
        return Err(format!(
            "插件 '{}' (ID: {}) 已存在。\n请先卸载现有插件，然后再导入新版本。",
            plugin_name, manifest.id
        ));
    }

    // 4. 获取插件目录
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");

    // 确保 plugins 目录存在
    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }

    // 5. 创建符号链接（使用 @local 后缀）
    let dir_name = make_plugin_dir_name(&manifest.id, InstallSource::Local);
    let plugin_link_path = plugins_dir.join(&dir_name);

    // 如果已存在，先删除
    if plugin_link_path.exists() {
        println!(
            "[plugin_manager] Removing existing plugin link: {:?}",
            plugin_link_path
        );
        #[cfg(windows)]
        {
            // Windows 上需要区分目录和文件的符号链接
            let metadata = std::fs::symlink_metadata(&plugin_link_path)
                .map_err(|e| format!("Failed to get symlink metadata: {}", e))?;
            if metadata.is_dir() {
                std::fs::remove_dir(&plugin_link_path)
                    .map_err(|e| format!("Failed to remove existing plugin link: {}", e))?;
            } else {
                std::fs::remove_file(&plugin_link_path)
                    .map_err(|e| format!("Failed to remove existing plugin link: {}", e))?;
            }
        }
        #[cfg(not(windows))]
        {
            std::fs::remove_file(&plugin_link_path)
                .map_err(|e| format!("Failed to remove existing plugin link: {}", e))?;
        }
    }

    // 创建符号链接
    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_dir(&source, &plugin_link_path) {
            Ok(_) => {}
            Err(e) => {
                // Windows 符号链接失败时提供详细的错误信息
                let error_msg = if e.raw_os_error() == Some(1314) {
                    "Failed to create symlink: Insufficient privileges.\n\n\
                     Please enable Developer Mode:\n\
                     1. Open Settings > Update & Security > For developers\n\
                     2. Enable 'Developer Mode'\n\
                     3. Restart the application\n\n\
                     Or run the application as Administrator."
                } else {
                    &format!("Failed to create symlink: {}.\n\
                             On Windows, you may need administrator privileges or Developer Mode enabled.", e)
                };
                return Err(error_msg.to_string());
            }
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&source, &plugin_link_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    println!(
        "[plugin_manager] Created symlink: {:?} -> {:?}",
        plugin_link_path, source
    );

    // 6. 加载插件到 store
    let plugin_states = load_plugin_states(&app);
    let (enabled, auto_detach) = if let Some(state) = plugin_states.get(&manifest.id) {
        (state.enabled, state.auto_detach)
    } else {
        (true, manifest.auto_detach)
    };

    let mut manifest_with_state = manifest.clone();
    manifest_with_state.auto_detach = auto_detach;

    let loaded_plugin = LoadedPlugin {
        manifest: manifest_with_state,
        dir_name: dir_name.clone(),
        enabled,
        settings: None,
        install_source: InstallSource::Local,
    };

    // 7. 添加到 store（使用 dir_name 作为 key）
    {
        let mut store_lock = store.0.lock().unwrap();
        store_lock.insert(dir_name.clone(), loaded_plugin.clone());
    }

    // 8. 初始化插件生命周期
    // Headless 插件：执行 index.js (entry)
    // View 插件：执行 lifecycle.js（如果存在）
    let entry_path = source.join(&manifest.entry);
    if entry_path.is_file() {
        if let Some(extension) = std::path::Path::new(&manifest.entry)
            .extension()
            .and_then(|s| s.to_str())
        {
            let lifecycle_path = match extension {
                "js" => {
                    // Headless 插件：直接使用 index.js
                    println!(
                        "[plugin_manager] Initializing headless plugin: {}",
                        manifest.id
                    );
                    Some(entry_path.clone())
                }
                "html" => {
                    // View 插件：查找 lifecycle.js
                    let lifecycle_file = manifest
                        .lifecycle
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("lifecycle.js");
                    let lc_path = source.join(lifecycle_file);

                    if lc_path.is_file() {
                        println!(
                            "[plugin_manager] Initializing view plugin lifecycle: {} ({})",
                            manifest.id, lifecycle_file
                        );
                        Some(lc_path)
                    } else {
                        println!(
                            "[plugin_manager] No lifecycle file found for view plugin {} (looked for: {})",
                            manifest.id, lifecycle_file
                        );
                        None
                    }
                }
                _ => None,
            };

            if let Some(lc_path) = lifecycle_path {
                let app_clone = app.clone();
                let plugin_id = manifest.id.clone();
                let plugin_name = manifest.name.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        match std::fs::read_to_string(&lc_path) {
                            Ok(js_code) => {
                                match js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id))
                                    .await
                                {
                                    Ok(_) => {
                                        // 通知前端初始化成功
                                        let _ = app_clone.emit("plugin-init-success", &plugin_id);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[plugin_manager] Failed to initialize plugin {}: {}",
                                            plugin_id, e
                                        );
                                        // 通知前端初始化失败
                                        #[derive(serde::Serialize, Clone)]
                                        struct PluginInitError {
                                            plugin_id: String,
                                            plugin_name: String,
                                            error: String,
                                        }
                                        let _ = app_clone.emit(
                                            "plugin-init-error",
                                            PluginInitError {
                                                plugin_id: plugin_id.clone(),
                                                plugin_name: plugin_name.clone(),
                                                error: e.to_string(),
                                            },
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[plugin_manager] Failed to read lifecycle file: {}", e);
                                #[derive(serde::Serialize, Clone)]
                                struct PluginInitError {
                                    plugin_id: String,
                                    plugin_name: String,
                                    error: String,
                                }
                                let _ = app_clone.emit(
                                    "plugin-init-error",
                                    PluginInitError {
                                        plugin_id,
                                        plugin_name,
                                        error: format!("Failed to read lifecycle file: {}", e),
                                    },
                                );
                            }
                        }
                    });
                });
            }
        }
    }

    println!(
        "[plugin_manager] Successfully imported plugin: {}",
        manifest.name
    );
    Ok(loaded_plugin)
}

#[tauri::command]
pub fn uninstall_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    // 1. 从 store 中获取插件信息（需要 dir_name）
    let (dir_name, actual_key) = {
        let store_lock = store.0.lock().unwrap();
        if let Some(plugin) = find_plugin_by_id(&store_lock, &plugin_id) {
            // 找到插件，获取其 dir_name 和实际的 key
            let key = store_lock
                .iter()
                .find(|(_, p)| p.manifest.id == plugin_id)
                .map(|(k, _)| k.clone());
            (Some(plugin.dir_name.clone()), key)
        } else {
            (None, None)
        }
    };

    // 2. 从 store 中移除（使用实际的 key）
    let plugin_was_in_store = {
        let mut store_lock = store.0.lock().unwrap();
        if let Some(key) = actual_key {
            store_lock.remove(&key).is_some()
        } else {
            false
        }
    };

    // 3. 删除插件目录（使用 dir_name 而不是 plugin_id）
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");

    // 如果有 dir_name，使用它；否则尝试使用 plugin_id
    let dir_to_remove = if let Some(ref dn) = dir_name {
        dn.clone()
    } else {
        plugin_id.clone()
    };

    let plugin_link_path = plugins_dir.join(&dir_to_remove);

    println!(
        "[plugin_manager] Attempting to remove plugin directory: {:?} (dir_name: {:?})",
        plugin_link_path, dir_name
    );

    if plugin_link_path.exists() {
        #[cfg(windows)]
        {
            // 在 Windows 上，使用 symlink_metadata 检查是否为符号链接
            let metadata = std::fs::symlink_metadata(&plugin_link_path)
                .map_err(|e| format!("Failed to get symlink metadata: {}", e))?;

            let file_type = metadata.file_type();
            let is_symlink = file_type.is_symlink();

            println!(
                "[plugin_manager] Symlink metadata - is_dir: {}, is_file: {}, is_symlink: {}",
                metadata.is_dir(),
                file_type.is_file(),
                is_symlink
            );

            // 对于符号链接，Windows 需要根据链接类型使用不同的删除方法
            if is_symlink {
                // 在 Windows 上，目录符号链接的 is_dir() 可能返回 false（如果目标不存在）
                // 我们先尝试作为目录删除，如果失败再尝试作为文件删除
                println!(
                    "[plugin_manager] Attempting to remove symlink (trying directory first)..."
                );
                match std::fs::remove_dir(&plugin_link_path) {
                    Ok(_) => {}
                    Err(_) => {
                        std::fs::remove_file(&plugin_link_path)
                            .map_err(|e| format!("Failed to remove plugin link: {}", e))?;
                    }
                }
            } else {
                // 不是符号链接，可能是实际的目录或文件
                if metadata.is_dir() {
                    std::fs::remove_dir_all(&plugin_link_path)
                        .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
                } else {
                    std::fs::remove_file(&plugin_link_path)
                        .map_err(|e| format!("Failed to remove plugin file: {}", e))?;
                }
            }
        }
        #[cfg(not(windows))]
        {
            std::fs::remove_file(&plugin_link_path)
                .map_err(|e| format!("Failed to remove plugin link: {}", e))?;
        }
    } else {
        println!(
            "[plugin_manager] Plugin directory does not exist: {:?}",
            plugin_link_path
        );

        // 如果使用 plugin_id 找不到，尝试扫描 plugins 目录查找匹配的插件
        if dir_name.is_none() {
            println!(
                "[plugin_manager] Scanning plugins directory to find plugin with ID: {}",
                plugin_id
            );
            if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let manifest_path = path.join("manifest.json");
                        if manifest_path.is_file() {
                            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                                if let Ok(manifest) =
                                    serde_json::from_str::<PluginManifest>(&content)
                                {
                                    if manifest.id == plugin_id {
                                        println!(
                                            "[plugin_manager] Found plugin directory: {:?}",
                                            path
                                        );
                                        // 尝试删除找到的目录
                                        #[cfg(windows)]
                                        {
                                            let metadata = std::fs::symlink_metadata(&path)
                                                .map_err(|e| {
                                                    format!("Failed to get metadata: {}", e)
                                                })?;
                                            if metadata.is_dir() {
                                                std::fs::remove_dir_all(&path).map_err(|e| {
                                                    format!("Failed to remove directory: {}", e)
                                                })?;
                                                println!("[plugin_manager] Successfully removed directory");
                                            }
                                        }
                                        #[cfg(not(windows))]
                                        {
                                            std::fs::remove_dir_all(&path).map_err(|e| {
                                                format!("Failed to remove directory: {}", e)
                                            })?;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. 清理插件状态
    let plugin_states = load_plugin_states(&app);
    let mut new_states = plugin_states.clone();
    new_states.remove(&plugin_id);
    save_plugin_states(&app, &new_states)?;

    // 5. 清理插件设置
    if let Ok(settings_path) = get_plugin_settings_path(&app, &plugin_id) {
        if settings_path.exists() {
            if let Err(e) = std::fs::remove_file(&settings_path) {
                eprintln!("[plugin_manager] Failed to remove settings file: {}", e);
            }
        }
    }

    // 6. 清理 JavaScript 运行时
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async {
        if let Err(e) = crate::js_runtime::clear_plugin_runtime(&plugin_id).await {
            eprintln!("[plugin_manager] Failed to clear JS runtime: {}", e);
        }
    });

    if !plugin_was_in_store && !plugin_link_path.exists() {
        return Err(format!("Plugin not found: {}", plugin_id));
    }

    Ok(())
}

pub fn handle_plugin_protocol<R: tauri::Runtime>(
    context: tauri::UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<std::borrow::Cow<'static, [u8]>> {
    let uri = request.uri();
    let path = uri.path();

    println!("[plugin_protocol] Request URI: {}", uri);
    println!("[plugin_protocol] Request path: {}", path);

    // 解析路径，格式为 /plugin_dir_name/file_path 或者 /plugin_dir_name/assets/file_path
    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    println!("[plugin_protocol] Path parts: {:?}", path_parts);

    if path_parts.is_empty() || path_parts[0].is_empty() {
        println!("[plugin_protocol] Empty path");
        return Response::builder()
            .status(404)
            .body("Not Found".as_bytes().to_vec().into())
            .unwrap();
    }

    let plugin_dir_name = path_parts[0];
    let file_path = if path_parts.len() > 1 {
        path_parts[1..].join("/")
    } else {
        // 如果只有插件目录名，默认加载 index.html
        "index.html".to_string()
    };

    // 获取插件目录
    let data_dir = match context.app_handle().path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("[plugin_protocol] Failed to get app data dir: {}", e);
            return Response::builder()
                .status(500)
                .body("Internal Server Error".as_bytes().to_vec().into())
                .unwrap();
        }
    };

    let plugin_file_path = data_dir
        .join("plugins")
        .join(plugin_dir_name)
        .join(&file_path);

    println!("[plugin_protocol] Requesting file: {:?}", plugin_file_path);

    // 检查文件是否存在
    if !plugin_file_path.exists() {
        println!(
            "[plugin_protocol] File does not exist: {:?}",
            plugin_file_path
        );

        // 尝试列出插件目录的内容以便调试
        let plugin_dir = data_dir.join("plugins").join(plugin_dir_name);
        if plugin_dir.exists() {
            println!(
                "[plugin_protocol] Plugin directory exists: {:?}",
                plugin_dir
            );
            if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
                println!("[plugin_protocol] Directory contents:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {:?}", entry.file_name());
                    }
                }
            }
        } else {
            println!(
                "[plugin_protocol] Plugin directory does not exist: {:?}",
                plugin_dir
            );
        }

        return Response::builder()
            .status(404)
            .body(
                format!("File Not Found: {}", file_path)
                    .as_bytes()
                    .to_vec()
                    .into(),
            )
            .unwrap();
    }

    // 读取文件内容
    let mut content = match std::fs::read(&plugin_file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("[plugin_protocol] Failed to read file: {}", e);
            return Response::builder()
                .status(500)
                .body("Failed to read file".as_bytes().to_vec().into())
                .unwrap();
        }
    };

    // 如果是 HTML 文件，需要修改其中的资源路径并注入内容
    if plugin_file_path.extension().and_then(|s| s.to_str()) == Some("html") {
        if let Ok(html_content) = String::from_utf8(content.clone()) {
            // 将绝对路径转换为相对路径
            let mut modified_html = html_content
                .replace("src=\"/assets/", "src=\"./assets/")
                .replace("href=\"/assets/", "href=\"./assets/")
                .replace("src='/assets/", "src='./assets/")
                .replace("href='/assets/", "href='./assets/")
                .replace("href=\"/vite.svg\"", "href=\"./vite.svg\"");

            // 获取插件信息以注入 plugin ID
            let store = context.app_handle().state::<PluginStore>();
            let store_lock = store.0.lock().unwrap();
            let plugin_opt = store_lock.values().find(|p| p.dir_name == plugin_dir_name);

            if let Some(plugin) = plugin_opt {
                // 注入 plugin ID
                let plugin_id_script = format!(
                    r#"<script>window.__PLUGIN_ID__ = '{}';</script>"#,
                    plugin.manifest.id
                );

                // 注入自定义顶栏
                let topbar_html = format!(
                    "{}{}\n<script>\n{}\n</script>",
                    plugin_id_script, PLUGIN_WINDOW_TOPBAR_TEMPLATE, PLUGIN_WINDOW_CONTROLS_SCRIPT
                );

                // 在 </head> 之前或 <body> 之后注入
                if let Some(head_pos) = modified_html.find("</head>") {
                    modified_html.insert_str(head_pos, &topbar_html);
                } else if let Some(body_pos) = modified_html.find("<body") {
                    if let Some(body_end) = modified_html[body_pos..].find('>') {
                        let insert_pos = body_pos + body_end + 1;
                        modified_html.insert_str(insert_pos, &topbar_html);
                    }
                }
            }

            content = modified_html.into_bytes();
        }
    }

    // 根据文件扩展名设置Content-Type
    let content_type = match plugin_file_path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    };

    println!(
        "[plugin_protocol] Serving file with content-type: {}",
        content_type
    );

    Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Cache-Control", "no-cache")
        .body(content.into())
        .unwrap()
}

/// 下载并安装插件（使用 @market 后缀）
#[tauri::command]
pub async fn download_and_install_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    download_url: String,
    _plugin_id: String,  // 市场 ID，仅用于前端传递，不用于验证
    icon_url: Option<String>,  // 市场提供的图标 URL
) -> Result<LoadedPlugin, String> {
    println!("[plugin_manager] Downloading plugin from: {}", download_url);

    // 1. 下载 ZIP 文件到临时目录
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let zip_path = temp_dir.path().join("plugin.zip");

    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("Failed to download plugin: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    std::fs::write(&zip_path, &bytes)
        .map_err(|e| format!("Failed to write zip file: {}", e))?;

    println!("[plugin_manager] Downloaded {} bytes", bytes.len());

    // 2. 解压到临时目录
    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    let file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    // 3. 查找插件根目录
    let plugin_root = find_plugin_root(&extract_dir)?;

    // 4. 读取 manifest.json（使用真实的插件 ID）
    let manifest_path = plugin_root.join("manifest.json");
    if !manifest_path.exists() {
        return Err("manifest.json not found in extracted files".to_string());
    }

    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    let mut manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Invalid manifest format: {}", e))?;

    // 如果市场提供了 icon URL，使用它覆盖 manifest 中的 icon
    if let Some(icon) = icon_url {
        manifest.icon = Some(icon);
    }

    println!(
        "[plugin_manager] Plugin manifest loaded: {} ({})",
        manifest.name, manifest.id
    );

    // 5. 生成带 @market 后缀的目录名
    let dir_name = make_plugin_dir_name(&manifest.id, InstallSource::Marketplace);
    
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");
    let target_dir = plugins_dir.join(&dir_name);

    // 检查市场版本是否已存在
    if target_dir.exists() {
        return Err(format!(
            "插件市场版本 '{}' 已存在。\n请先卸载现有版本，然后再安装。",
            manifest.name
        ));
    }

    // 6. 确保 plugins 目录存在并复制文件
    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }

    copy_dir_all(&plugin_root, &target_dir)?;

    // 将更新后的 manifest（包含 icon URL）写回到目标目录
    let target_manifest_path = target_dir.join("manifest.json");
    let updated_manifest_content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(&target_manifest_path, updated_manifest_content)
        .map_err(|e| format!("Failed to write updated manifest: {}", e))?;

    println!(
        "[plugin_manager] Plugin files copied to: {:?}",
        target_dir
    );

    // 7. 加载插件到 store
    let plugin_states = load_plugin_states(&app);
    let auto_detach = if let Some(state) = plugin_states.get(&manifest.id) {
        state.auto_detach
    } else {
        manifest.auto_detach
    };

    let mut manifest_with_state = manifest.clone();
    manifest_with_state.auto_detach = auto_detach;

    let loaded_plugin = LoadedPlugin {
        manifest: manifest_with_state,
        dir_name: dir_name.clone(),
        enabled: true,  // 新安装的插件默认启用
        settings: None,
        install_source: InstallSource::Marketplace,
    };

    // 添加到 store（使用 dir_name 作为 key）
    // 并禁用同一 manifest.id 的其他版本
    {
        let mut store_lock = store.0.lock().unwrap();
        
        // 禁用同一 manifest.id 的其他版本
        let other_versions: Vec<String> = store_lock
            .iter()
            .filter(|(_, p)| p.manifest.id == manifest.id && p.enabled)
            .map(|(dir_name, _)| dir_name.clone())
            .collect();
        
        for other_dir_name in other_versions {
            if let Some(other_plugin) = store_lock.get_mut(&other_dir_name) {
                other_plugin.enabled = false;
                println!(
                    "[plugin_manager] Auto-disabled {} version when installing market version",
                    match other_plugin.install_source {
                        InstallSource::Local => "local",
                        InstallSource::Marketplace => "market",
                    }
                );
            }
        }
        
        store_lock.insert(dir_name.clone(), loaded_plugin.clone());
    }

    // 8. 初始化插件生命周期
    let entry_path = target_dir.join(&manifest.entry);
    if entry_path.is_file() {
        if let Some(extension) = std::path::Path::new(&manifest.entry)
            .extension()
            .and_then(|s| s.to_str())
        {
            let lifecycle_path = match extension {
                "js" => Some(entry_path.clone()),
                "html" => {
                    let lifecycle_file = manifest
                        .lifecycle
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("lifecycle.js");
                    let lc_path = target_dir.join(lifecycle_file);
                    if lc_path.is_file() {
                        Some(lc_path)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(lc_path) = lifecycle_path {
                let app_clone = app.clone();
                let plugin_id = manifest.id.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        if let Ok(js_code) = std::fs::read_to_string(&lc_path) {
                            let _ = js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await;
                        }
                    });
                });
            }
        }
    }

    println!(
        "[plugin_manager] Successfully installed plugin: {}",
        manifest.name
    );
    
    // 发送插件安装成功事件，通知前端刷新列表
    let _ = app.emit("plugin-installed", &manifest.id);
    
    Ok(loaded_plugin)
}

/// 查找包含 manifest.json 的插件根目录
fn find_plugin_root(extract_dir: &Path) -> Result<std::path::PathBuf, String> {
    if extract_dir.join("manifest.json").exists() {
        return Ok(extract_dir.to_path_buf());
    }

    let entries = std::fs::read_dir(extract_dir)
        .map_err(|e| format!("Failed to read extract directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() && path.join("manifest.json").exists() {
            return Ok(path);
        }
    }

    Err("manifest.json not found in extracted files".to_string())
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory {:?}: {}", dst, e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {:?}: {}", src, e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let ty = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file {:?}: {}", src_path, e))?;
        }
    }

    Ok(())
}
