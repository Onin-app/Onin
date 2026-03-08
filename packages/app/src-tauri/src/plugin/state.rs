//! # 插件状态持久化模块
//!
//! 负责插件状态的持久化存储，包括：
//! - 插件启用/禁用状态
//! - 插件窗口位置和大小
//! - 插件用户设置

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;

use super::types::{PluginState, PluginStates, PluginWindowStates, WindowBounds};

// ============================================================================
// 路径获取函数
// ============================================================================

/// 获取插件状态文件路径
///
/// 返回 `app_data_dir/plugin_states.json`
pub fn get_plugin_states_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("plugin_states.json"))
}

/// 获取插件窗口状态文件路径
///
/// 返回 `app_data_dir/plugin_data/window_states.json`
pub fn get_plugin_window_states_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_data_dir = data_dir.join("plugin_data");

    // 确保目录存在
    if !plugin_data_dir.exists() {
        std::fs::create_dir_all(&plugin_data_dir).map_err(|e| e.to_string())?;
    }

    Ok(plugin_data_dir.join("window_states.json"))
}

/// 获取插件设置文件路径
///
/// 返回 `app_data_dir/plugin_settings/{plugin_id}.json`
pub fn get_plugin_settings_path(
    app: &tauri::AppHandle,
    plugin_id: &str,
) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings_dir = data_dir.join("plugin_settings");

    // 确保目录存在
    if !settings_dir.exists() {
        std::fs::create_dir_all(&settings_dir).map_err(|e| e.to_string())?;
    }

    Ok(settings_dir.join(format!("{}.json", plugin_id)))
}

// ============================================================================
// 插件状态操作
// ============================================================================

/// 加载插件状态
///
/// 从文件中读取所有插件的启用/禁用状态
pub fn load_plugin_states(app: &tauri::AppHandle) -> HashMap<String, PluginState> {
    match get_plugin_states_path(app) {
        Ok(path) => {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<PluginStates>(&content) {
                        Ok(plugin_states) => {
                            println!("[plugin/state] 已加载插件状态: {:?}", plugin_states.states);
                            return plugin_states.states;
                        }
                        Err(e) => {
                            eprintln!("[plugin/state] 解析插件状态失败: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[plugin/state] 读取插件状态文件失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[plugin/state] 获取插件状态路径失败: {}", e);
        }
    }
    HashMap::new()
}

/// 保存插件状态
///
/// 将所有插件的启用/禁用状态写入文件
pub fn save_plugin_states(
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

// ============================================================================
// 窗口状态操作
// ============================================================================

/// 加载插件窗口状态
///
/// 从文件中读取所有插件窗口的位置和大小信息
/// 采用静默模式，减少日志输出
pub fn load_plugin_window_states(app: &tauri::AppHandle) -> HashMap<String, WindowBounds> {
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
                            eprintln!("[plugin/state] 解析插件窗口状态失败: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[plugin/state] 读取插件窗口状态文件失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("[plugin/state] 获取插件窗口状态路径失败: {}", e);
        }
    }
    HashMap::new()
}

/// 保存单个插件窗口状态
///
/// 只在窗口关闭时调用，更新指定插件的窗口位置和大小
pub fn save_plugin_window_state(
    app: &tauri::AppHandle,
    plugin_id: &str,
    mut bounds: WindowBounds,
) -> Result<(), String> {
    // 验证并修正窗口尺寸
    bounds.validate_and_fix();

    let path = get_plugin_window_states_path(app)?;

    // 加载现有状态
    let mut all_states = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<PluginWindowStates>(&content) {
                Ok(states) => states.windows,
                Err(_) => HashMap::new(),
            },
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    // 更新当前插件的状态
    all_states.insert(plugin_id.to_string(), bounds);

    // 保存到文件
    let window_states = PluginWindowStates {
        windows: all_states,
    };
    let content = serde_json::to_string_pretty(&window_states).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;

    println!("[plugin/state] 已保存插件 {} 的窗口状态", plugin_id);
    Ok(())
}

// ============================================================================
// 插件设置操作
// ============================================================================

/// 加载插件设置
///
/// 从文件中读取指定插件的用户设置
pub fn load_plugin_settings(
    app: &tauri::AppHandle,
    plugin_id: &str,
) -> Result<HashMap<String, JsonValue>, String> {
    let settings_path = get_plugin_settings_path(app, plugin_id)?;

    if !settings_path.exists() {
        return Ok(HashMap::new());
    }

    let content =
        std::fs::read_to_string(&settings_path).map_err(|e| format!("读取设置文件失败: {}", e))?;

    let settings: HashMap<String, JsonValue> =
        serde_json::from_str(&content).map_err(|e| format!("解析设置失败: {}", e))?;

    println!(
        "[plugin/state] 已加载插件 {} 的设置: {:?}",
        plugin_id, settings
    );
    Ok(settings)
}

/// 保存插件设置
///
/// 将指定插件的用户设置写入文件
pub fn save_plugin_settings_to_file(
    app: &tauri::AppHandle,
    plugin_id: &str,
    settings: &HashMap<String, JsonValue>,
) -> Result<(), String> {
    let settings_path = get_plugin_settings_path(app, plugin_id)?;

    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化设置失败: {}", e))?;

    std::fs::write(&settings_path, content).map_err(|e| format!("写入设置文件失败: {}", e))?;

    println!(
        "[plugin/state] 已保存插件 {} 的设置: {:?}",
        plugin_id, settings
    );
    Ok(())
}

// ============================================================================
// 安全辅助函数
// ============================================================================

/// 获取系统保护路径
///
/// 返回不允许导入插件的系统敏感目录列表
/// 使用系统 API 动态获取，确保跨平台兼容
pub fn get_system_protected_paths() -> Vec<std::path::PathBuf> {
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
            paths.push(std::path::PathBuf::from(&system_root).join("System32"));
            paths.push(std::path::PathBuf::from(&system_root).join("SysWOW64"));
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

/// 收集所有插件的状态
///
/// 遍历插件存储，生成状态映射表
pub fn collect_plugin_states(
    store: &HashMap<String, crate::plugin::types::LoadedPlugin>,
) -> HashMap<String, PluginState> {
    let mut states = HashMap::new();
    for (_, plugin) in store.iter() {
        // 对于同一 manifest.id 的多个版本，只保存启用的那个
        if plugin.enabled || !states.contains_key(&plugin.manifest.id) {
            states.insert(
                plugin.manifest.id.clone(),
                PluginState {
                    enabled: plugin.enabled,
                    auto_detach: plugin.manifest.auto_detach,
                    terminate_on_bg: plugin.manifest.terminate_on_bg,
                    run_at_startup: plugin.manifest.run_at_startup,
                },
            );
        }
    }
    states
}
