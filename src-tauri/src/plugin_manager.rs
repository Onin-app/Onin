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
}

fn default_enabled() -> bool {
    true
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

// 获取插件状态文件路径
fn get_plugin_states_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("plugin_states.json"))
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
    println!("[plugin_manager] Saved plugin states: {:?}", states);
    Ok(())
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
    println!("[plugin_manager] Loading plugins...");
    let data_dir = app.path().app_data_dir().map_err(|e| {
        println!("[plugin_manager] Error getting data dir: {}", e);
        e.to_string()
    })?;

    let plugins_dir = data_dir.join("plugins");
    println!("[plugin_manager] Plugins dir: {:?}", plugins_dir);

    if !plugins_dir.is_dir() {
        println!("[plugin_manager] Plugins dir not found.");
        return Ok(Vec::new());
    }

    // 加载插件状态
    let plugin_states = load_plugin_states(app);

    let mut store_lock = store.0.lock().unwrap();
    if clear_existing {
        store_lock.clear(); // Clear old plugins
        println!("[plugin_manager] Cleared existing plugins from store.");
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

                // 从持久化状态中获取启用状态和 auto_detach，如果没有则使用默认值
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
                    settings: None, // Settings will be registered dynamically via JavaScript
                };

                println!(
                    "[plugin_manager] Loaded plugin: {} from {} (enabled: {})",
                    manifest.name, loaded_plugin.dir_name, enabled
                );

                // 自动执行 entry 文件进行初始化（如果是 .js 文件）
                let entry_path = path.join(&manifest.entry);
                if entry_path.is_file() {
                    if let Some(extension) = Path::new(&manifest.entry)
                        .extension()
                        .and_then(|s| s.to_str())
                    {
                        if extension == "js" {
                            // JS 文件自动执行以注册设置和命令
                            plugins_to_init.push((
                                manifest.id.clone(),
                                entry_path,
                                dir_name.clone(),
                            ));
                        }
                    }
                }

                store_lock.insert(manifest.id.clone(), loaded_plugin);
            }
        }
    }

    let plugins = store_lock.values().cloned().collect();
    println!("[plugin_manager] Loaded {} plugins.", store_lock.len());

    // 释放锁后执行初始化脚本
    drop(store_lock);

    // 执行所有插件的初始化脚本
    if !plugins_to_init.is_empty() {
        println!(
            "[plugin_manager] Initializing {} plugins",
            plugins_to_init.len()
        );
        let app_clone = app.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                for (plugin_id, entry_path, _dir_name) in plugins_to_init {
                    println!("[plugin_manager] Initializing plugin: {}", plugin_id);
                    match std::fs::read_to_string(&entry_path) {
                        Ok(js_code) => {
                            if let Err(e) =
                                js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await
                            {
                                eprintln!(
                                    "[plugin_manager] Failed to initialize plugin {}: {}",
                                    plugin_id, e
                                );
                            } else {
                                println!(
                                    "[plugin_manager] Successfully initialized plugin {}",
                                    plugin_id
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[plugin_manager] Failed to read entry file for {}: {}",
                                plugin_id, e
                            );
                        }
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
    println!("[plugin_manager] Refreshing plugins...");

    // 清除 JavaScript 运行时缓存
    if let Err(e) = crate::js_runtime::clear_all_plugin_runtimes().await {
        println!(
            "[plugin_manager] Warning: Failed to clear JS runtimes: {}",
            e
        );
        // 不要因为这个错误而失败，继续刷新插件
    } else {
        println!("[plugin_manager] Successfully cleared all plugin runtimes");
    }

    // 清除插件加载状态缓存
    let loaded_state = app.state::<crate::plugin_api::command::PluginLoadedState>();
    {
        let mut state = loaded_state.0.lock().unwrap();
        let count = state.len();
        state.clear();
        println!("[plugin_manager] Cleared {} plugin loaded states", count);
    }

    // 重新加载所有插件
    let result = load_plugins_internal(&app, &store, true);

    match &result {
        Ok(plugins) => {
            println!(
                "[plugin_manager] Successfully refreshed {} plugins.",
                plugins.len()
            );
        }
        Err(e) => {
            println!("[plugin_manager] Failed to refresh plugins: {}", e);
        }
    }

    result
}

#[tauri::command]
pub fn toggle_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = store_lock.get_mut(&plugin_id) {
        plugin.enabled = enabled;
        println!(
            "[plugin_manager] Plugin {} is now {}",
            plugin_id,
            if enabled { "enabled" } else { "disabled" }
        );

        // 收集所有插件的状态
        let mut states = HashMap::new();
        for (id, plugin) in store_lock.iter() {
            states.insert(
                id.clone(),
                PluginState {
                    enabled: plugin.enabled,
                    auto_detach: plugin.manifest.auto_detach,
                },
            );
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
pub fn toggle_plugin_auto_detach(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    auto_detach: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = store_lock.get_mut(&plugin_id) {
        plugin.manifest.auto_detach = auto_detach;
        println!(
            "[plugin_manager] Plugin {} auto_detach is now {}",
            plugin_id, auto_detach
        );

        // 收集所有插件的状态
        let mut states = HashMap::new();
        for (id, plugin) in store_lock.iter() {
            states.insert(
                id.clone(),
                PluginState {
                    enabled: plugin.enabled,
                    auto_detach: plugin.manifest.auto_detach,
                },
            );
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

    if let Some(plugin) = store_lock.get_mut(&plugin_id) {
        // Store settings schema in LoadedPlugin (not in manifest)
        plugin.settings = Some(schema.clone());
        println!(
            "[plugin_manager] ✅ Registered settings schema for plugin {}: {} fields",
            plugin_id,
            schema.fields.len()
        );

        // Emit event to notify frontend that schema has been registered
        println!("[plugin_manager] Emitting plugin-settings-schema-registered event");
        if let Err(e) = app.emit("plugin-settings-schema-registered", &plugin_id) {
            eprintln!(
                "[plugin_manager] ❌ Failed to emit schema registered event: {}",
                e
            );
            return Err(format!("Failed to emit event: {}", e));
        }
        println!("[plugin_manager] ✅ Event emitted successfully");

        Ok(())
    } else {
        let error_msg = format!("Plugin not found: {}", plugin_id);
        eprintln!("[plugin_manager] ❌ {}", error_msg);
        Err(error_msg)
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

    store_lock
        .get(&plugin_id)
        .cloned()
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))
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
pub fn open_plugin_in_window(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    // Clone plugin data to release the lock ASAP
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        store_lock.get(&plugin_id).cloned()
    }
    .ok_or_else(|| "Plugin not found".to_string())?;

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

/// 创建或显示插件窗口的共享函数
async fn create_or_show_plugin_window(
    app: tauri::AppHandle,
    plugin: &LoadedPlugin,
) -> Result<(), String> {
    let window_label = format!("plugin_{}", plugin.manifest.id.replace('.', "_"));

    // 如果窗口已存在，切换显示状态
    if let Some(window) = app.get_webview_window(&window_label) {
        // 检查窗口是否被最小化
        let is_minimized = window.is_minimized().unwrap_or(false);
        // 检查窗口是否可见
        let is_visible = window.is_visible().unwrap_or(false);
        
        if is_minimized || !is_visible {
            // 窗口被最小化或隐藏，显示并聚焦
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
        } else {
            // 窗口已显示，最小化它
            if let Err(e) = window.minimize() {
                eprintln!("Failed to minimize plugin window: {}", e);
            }
        }
        return Ok(());
    }

    // 使用自定义协议来加载插件文件
    let plugin_url = format!(
        "plugin://localhost/{}/{}",
        plugin.dir_name, plugin.manifest.entry
    );
    println!(
        "[plugin_manager] Loading plugin in window from: {}",
        plugin_url
    );

    // 创建窗口菜单
    use tauri::menu::{Menu, MenuItemBuilder};
    let menu_result = (|| -> Result<Menu<tauri::Wry>, tauri::Error> {
        let back_to_inline =
            MenuItemBuilder::with_id("back_to_inline", "切换到主窗口模式").build(&app)?;
        Menu::with_items(&app, &[&back_to_inline])
    })();

    let mut builder = WebviewWindowBuilder::new(
        &app,
        window_label.clone(),
        tauri::WebviewUrl::External(plugin_url.parse().unwrap()),
    )
    .title(plugin.manifest.name.clone())
    .inner_size(800.0, 600.0)
    .resizable(true)
    .decorations(false); // 隐藏系统标题栏

    // 如果菜单创建成功，添加到窗口
    if let Ok(menu) = menu_result {
        builder = builder.menu(menu);
    }

    match builder.build() {
        Ok(window) => {
            let plugin_id_for_menu = plugin.manifest.id.clone();
            let app_for_menu = app.clone();

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
                println!("[plugin_manager] Registering ESC shortcut for plugin window");
                let _ = app_for_immediate.global_shortcut().unregister(esc_shortcut);
                if let Err(e) = app_for_immediate.global_shortcut().register(esc_shortcut) {
                    eprintln!("[plugin_manager] Failed to register ESC shortcut: {}", e);
                } else {
                    println!("[plugin_manager] ESC shortcut registered successfully");
                }
            });

            let label_for_event = window_label_for_tracking.clone();
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        println!(
                            "[plugin_manager] Plugin window focused: {}",
                            label_for_event
                        );

                        // 记录活跃窗口
                        if let Some(active_window_state) =
                            app_for_window_event.try_state::<ActivePluginWindow>()
                        {
                            if let Ok(mut active) = active_window_state.0.lock() {
                                *active = Some(label_for_event.clone());
                                println!(
                                    "[plugin_manager] Updated active plugin window: {}",
                                    label_for_event
                                );
                            }
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
                        } else {
                            println!("[plugin_manager] ESC shortcut registered successfully");
                        }
                    }
                    tauri::WindowEvent::Focused(false) => {
                        println!(
                            "[plugin_manager] Plugin window unfocused: {}",
                            label_for_event
                        );

                        // 清除活跃窗口记录
                        if let Some(active_window_state) =
                            app_for_window_event.try_state::<ActivePluginWindow>()
                        {
                            if let Ok(mut active) = active_window_state.0.lock() {
                                if active.as_ref() == Some(&label_for_event) {
                                    *active = None;
                                    println!("[plugin_manager] Cleared active plugin window");
                                }
                            }
                        }

                        // 注销 ESC 快捷键
                        if let Err(e) = app_for_window_event
                            .global_shortcut()
                            .unregister(esc_shortcut)
                        {
                            eprintln!("Failed to unregister ESC shortcut: {}", e);
                        }
                    }
                    tauri::WindowEvent::CloseRequested { .. } => {
                        println!(
                            "[plugin_manager] Plugin window closing: {}",
                            label_for_event
                        );

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

            // 监听菜单事件
            window.on_menu_event(move |window, event| {
                if event.id().as_ref() == "back_to_inline" {
                    println!(
                        "[plugin_manager] Switching plugin {} back to inline mode",
                        plugin_id_for_menu
                    );

                    // 关闭当前窗口
                    if let Err(e) = window.close() {
                        eprintln!("Failed to close plugin window: {}", e);
                    }

                    // 切换插件的 auto_detach 为 false
                    if let Err(e) = toggle_plugin_auto_detach(
                        app_for_menu.clone(),
                        app_for_menu.state::<PluginStore>(),
                        plugin_id_for_menu.clone(),
                        false, // 设置为 false，切换回主窗口模式
                    ) {
                        eprintln!("Failed to toggle plugin auto_detach: {}", e);
                    }

                    // 显示主窗口
                    if let Some(main_window) = app_for_menu.get_webview_window("main") {
                        if let Err(e) = main_window.show() {
                            eprintln!("Failed to show main window: {}", e);
                        }
                        if let Err(e) = main_window.set_focus() {
                            eprintln!("Failed to focus main window: {}", e);
                        }
                    }
                }
            });

            Ok(())
        }
        Err(e) => {
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
        store_lock.get(&plugin_id).cloned()
    }
    .ok_or_else(|| "Plugin not found".to_string())?;

    println!(
        "[plugin_manager] Plugin found: {}, enabled: {}",
        plugin.manifest.name, plugin.enabled
    );

    // 检查插件是否启用
    if !plugin.enabled {
        return Err("Plugin is disabled".to_string());
    }

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("Plugin entry file not found: {:?}", entry_path));
    }

    println!("[plugin_manager] Plugin entry: {}", plugin.manifest.entry);

    if let Some(extension) = Path::new(&plugin.manifest.entry)
        .extension()
        .and_then(|s| s.to_str())
    {
        println!("[plugin_manager] Plugin extension: {}", extension);
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
                    // Display inline - read and inline all resources (CSS, JS) into HTML
                    let html_content = std::fs::read_to_string(&entry_path)
                        .map_err(|e| format!("Failed to read plugin HTML: {}", e))?;

                    let html_dir = entry_path.parent().unwrap_or(&plugin_dir);

                    // Inline all CSS and JS resources
                    let mut modified_html = inline_resources(&html_content, html_dir);

                    // Inject Tauri API bridge and plugin ID
                    modified_html = inject_tauri_bridge(&modified_html, &plugin.manifest.id);

                    // Send to frontend
                    let main_window = app
                        .get_webview_window("main")
                        .ok_or_else(|| "Main window not found".to_string())?;

                    // 特殊处理：当通过快捷键触发内联插件时，需要显示主窗口
                    // 这是因为内联插件默认在主窗口中显示，如果主窗口隐藏，用户将看不到任何反馈
                    // 即使用户为主窗口显示/隐藏绑定了其他快捷键，这里也需要确保窗口可见
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
                        html_content: String,
                    }

                    let payload = PluginInlinePayload {
                        plugin_id: plugin.manifest.id.clone(),
                        plugin_name: plugin.manifest.name.clone(),
                        html_content: modified_html,
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
        if let Some(existing) = store_lock.get(&manifest.id) {
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

    // 5. 创建符号链接
    let plugin_link_path = plugins_dir.join(&manifest.id);

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
        dir_name: manifest.id.clone(),
        enabled,
        settings: None,
    };

    // 7. 添加到 store
    {
        let mut store_lock = store.0.lock().unwrap();
        store_lock.insert(manifest.id.clone(), loaded_plugin.clone());
    }

    println!("[plugin_manager] Plugin added to store: {}", manifest.id);

    // 8. 初始化插件（如果是 JS 文件）
    let entry_path = source.join(&manifest.entry);
    if entry_path.is_file() {
        if let Some(extension) = std::path::Path::new(&manifest.entry)
            .extension()
            .and_then(|s| s.to_str())
        {
            if extension == "js" {
                println!("[plugin_manager] Initializing JS plugin: {}", manifest.id);
                let app_clone = app.clone();
                let plugin_id = manifest.id.clone();
                let plugin_name = manifest.name.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        match std::fs::read_to_string(&entry_path) {
                            Ok(js_code) => {
                                match js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await {
                                    Ok(_) => {
                                        println!("[plugin_manager] Successfully initialized imported plugin {}", plugin_id);
                                        // 通知前端初始化成功
                                        let _ = app_clone.emit("plugin-init-success", &plugin_id);
                                    }
                                    Err(e) => {
                                        eprintln!("[plugin_manager] Failed to initialize imported plugin {}: {}", plugin_id, e);
                                        // 通知前端初始化失败
                                        #[derive(serde::Serialize, Clone)]
                                        struct PluginInitError {
                                            plugin_id: String,
                                            plugin_name: String,
                                            error: String,
                                        }
                                        let _ = app_clone.emit("plugin-init-error", PluginInitError {
                                            plugin_id: plugin_id.clone(),
                                            plugin_name: plugin_name.clone(),
                                            error: e.to_string(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[plugin_manager] Failed to read entry file for {}: {}", plugin_id, e);
                                #[derive(serde::Serialize, Clone)]
                                struct PluginInitError {
                                    plugin_id: String,
                                    plugin_name: String,
                                    error: String,
                                }
                                let _ = app_clone.emit("plugin-init-error", PluginInitError {
                                    plugin_id,
                                    plugin_name,
                                    error: format!("Failed to read entry file: {}", e),
                                });
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
    println!("[plugin_manager] Uninstalling plugin: {}", plugin_id);

    // 1. 从 store 中移除（如果存在）
    let plugin_was_in_store = {
        let mut store_lock = store.0.lock().unwrap();
        store_lock.remove(&plugin_id).is_some()
    };

    if plugin_was_in_store {
        println!("[plugin_manager] Plugin removed from store: {}", plugin_id);
    } else {
        println!("[plugin_manager] Plugin not found in store, will try to clean up files: {}", plugin_id);
    }

    // 2. 删除符号链接（即使插件不在 store 中也尝试删除）
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_link_path = data_dir.join("plugins").join(&plugin_id);

    if plugin_link_path.exists() {
        println!(
            "[plugin_manager] Removing plugin link: {:?}",
            plugin_link_path
        );
        #[cfg(windows)]
        {
            // 在 Windows 上，使用 symlink_metadata 检查是否为符号链接
            let metadata = std::fs::symlink_metadata(&plugin_link_path)
                .map_err(|e| format!("Failed to get symlink metadata: {}", e))?;
            
            let file_type = metadata.file_type();
            let is_symlink = file_type.is_symlink();
            
            println!("[plugin_manager] Symlink metadata - is_dir: {}, is_file: {}, is_symlink: {}", 
                metadata.is_dir(), file_type.is_file(), is_symlink);
            
            // 对于符号链接，Windows 需要根据链接类型使用不同的删除方法
            if is_symlink {
                // 在 Windows 上，目录符号链接的 is_dir() 可能返回 false（如果目标不存在）
                // 我们先尝试作为目录删除，如果失败再尝试作为文件删除
                println!("[plugin_manager] Attempting to remove symlink (trying directory first)...");
                match std::fs::remove_dir(&plugin_link_path) {
                    Ok(_) => {
                        println!("[plugin_manager] Successfully removed directory symlink");
                    }
                    Err(e) => {
                        println!("[plugin_manager] Failed to remove as directory ({}), trying as file...", e);
                        std::fs::remove_file(&plugin_link_path)
                            .map_err(|e2| format!("Failed to remove plugin link as both directory and file. Dir error: {}, File error: {}", e, e2))?;
                        println!("[plugin_manager] Successfully removed file symlink");
                    }
                }
            } else {
                // 不是符号链接，可能是实际的目录或文件
                if metadata.is_dir() {
                    println!("[plugin_manager] Removing actual directory (not a symlink)...");
                    std::fs::remove_dir_all(&plugin_link_path)
                        .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
                    println!("[plugin_manager] Successfully removed directory");
                } else {
                    println!("[plugin_manager] Removing actual file (not a symlink)...");
                    std::fs::remove_file(&plugin_link_path)
                        .map_err(|e| format!("Failed to remove plugin file: {}", e))?;
                    println!("[plugin_manager] Successfully removed file");
                }
            }
        }
        #[cfg(not(windows))]
        {
            std::fs::remove_file(&plugin_link_path)
                .map_err(|e| format!("Failed to remove plugin link: {}", e))?;
        }
    } else {
        println!("[plugin_manager] Plugin link does not exist: {:?}", plugin_link_path);
    }

    // 3. 清理插件状态
    println!("[plugin_manager] Cleaning plugin states...");
    let plugin_states = load_plugin_states(&app);
    let mut new_states = plugin_states.clone();
    new_states.remove(&plugin_id);
    save_plugin_states(&app, &new_states)?;
    println!("[plugin_manager] Plugin states cleaned");

    // 4. 清理插件设置
    println!("[plugin_manager] Cleaning plugin settings...");
    if let Ok(settings_path) = get_plugin_settings_path(&app, &plugin_id) {
        if settings_path.exists() {
            match std::fs::remove_file(&settings_path) {
                Ok(_) => println!("[plugin_manager] Plugin settings removed: {:?}", settings_path),
                Err(e) => eprintln!("[plugin_manager] Warning: Failed to remove settings file: {}", e),
            }
        } else {
            println!("[plugin_manager] No settings file to remove");
        }
    }

    // 5. 清理 JavaScript 运行时
    println!("[plugin_manager] Cleaning JavaScript runtime...");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    rt.block_on(async {
        if let Err(e) = crate::js_runtime::clear_plugin_runtime(&plugin_id).await {
            eprintln!("[plugin_manager] Warning: Failed to clear JS runtime for {}: {}", plugin_id, e);
        } else {
            println!("[plugin_manager] JavaScript runtime cleaned");
        }
    });

    if !plugin_was_in_store && !plugin_link_path.exists() {
        return Err(format!("Plugin not found: {}", plugin_id));
    }

    println!(
        "[plugin_manager] Successfully uninstalled plugin: {}",
        plugin_id
    );
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

    // 如果是 HTML 文件，需要修改其中的资源路径并注入顶栏
    if plugin_file_path.extension().and_then(|s| s.to_str()) == Some("html") {
        if let Ok(html_content) = String::from_utf8(content.clone()) {
            // 将绝对路径转换为相对路径，这样浏览器会通过同一个协议请求资源
            let mut modified_html = html_content
                .replace("src=\"/assets/", "src=\"./assets/")
                .replace("href=\"/assets/", "href=\"./assets/")
                .replace("src='/assets/", "src='./assets/")
                .replace("href='/assets/", "href='./assets/")
                .replace("href=\"/vite.svg\"", "href=\"./vite.svg\"");

            // 注入自定义顶栏（从外部模板加载）
            let topbar_html = format!(
                "{}\n<script>\n{}\n</script>",
                PLUGIN_WINDOW_TOPBAR_TEMPLATE,
                PLUGIN_WINDOW_CONTROLS_SCRIPT
            );

            // 在 </head> 之前或 <body> 之后注入顶栏
            if let Some(head_pos) = modified_html.find("</head>") {
                modified_html.insert_str(head_pos, &topbar_html);
            } else if let Some(body_pos) = modified_html.find("<body") {
                if let Some(body_end) = modified_html[body_pos..].find('>') {
                    let insert_pos = body_pos + body_end + 1;
                    modified_html.insert_str(insert_pos, &topbar_html);
                }
            }

            content = modified_html.into_bytes();
            println!(
                "[plugin_protocol] Modified HTML content and injected topbar for plugin: {}",
                plugin_dir_name
            );
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
