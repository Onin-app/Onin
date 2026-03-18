//! # 插件加载模块
//!
//! 负责插件的加载和刷新，包括：
//! - 从文件系统扫描和加载插件
//! - 执行插件的初始化脚本
//! - 刷新插件列表

use std::path::Path;
use tauri::{Manager, State};

use super::state::load_plugin_states;
use super::types::{parse_plugin_dir_name, LoadedPlugin, PluginManifest, PluginStore};
use crate::js_runtime;

// ============================================================================
// 内部加载函数
// ============================================================================

/// 插件加载的内部实现
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `store`: 插件存储状态
/// - `clear_existing`: 是否清除现有插件
///
/// # 返回
/// 加载的插件列表
pub fn load_plugins_internal(
    app: &tauri::AppHandle,
    store: &State<PluginStore>,
    clear_existing: bool,
) -> Result<Vec<LoadedPlugin>, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");

    // 如果插件目录不存在，返回空列表
    if !plugins_dir.is_dir() {
        return Ok(Vec::new());
    }

    // 加载持久化的插件状态
    let plugin_states = load_plugin_states(app);

    let mut store_lock = store.0.lock().unwrap();
    if clear_existing {
        store_lock.clear();
    }

    // 收集需要初始化的插件
    let mut plugins_to_init = Vec::new();

    // 遍历插件目录
    for entry in std::fs::read_dir(plugins_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        // 跳过非目录
        if !path.is_dir() {
            continue;
        }

        // 检查 manifest.json 是否存在
        let manifest_path = path.join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }

        // 读取并解析 manifest
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        let manifest: PluginManifest =
            serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;

        let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();

        // 解析目录名，提取插件 ID 和安装来源
        let (_parsed_id, install_source) = parse_plugin_dir_name(&dir_name);

        // 从持久化状态中恢复插件运行相关状态
        let (enabled, auto_detach, terminate_on_bg, run_at_startup) =
            if let Some(state) = plugin_states.get(&manifest.id) {
                (
                    state.enabled,
                    state.auto_detach,
                    state.terminate_on_bg,
                    state.run_at_startup,
                )
            } else {
                (
                    true,
                    manifest.auto_detach,
                    manifest.terminate_on_bg,
                    manifest.run_at_startup,
                )
            };

        let mut manifest_with_state = manifest.clone();
        manifest_with_state.auto_detach = auto_detach;
        manifest_with_state.terminate_on_bg = terminate_on_bg;
        manifest_with_state.run_at_startup = run_at_startup;

        // 自动执行后台初始化入口
        // Headless 插件：执行 index.js (entry)
        // View 插件：执行 manifest.lifecycle 指向的后台入口（默认 lifecycle.js）
        let entry_path = path.join(&manifest.entry);

        if entry_path.is_file() {
            if let Some(extension) = Path::new(&manifest.entry)
                .extension()
                .and_then(|s| s.to_str())
            {
                match extension {
                    "js" => {
                        // Headless 插件：直接执行 index.js
                        plugins_to_init.push((manifest.id.clone(), entry_path, dir_name.clone()));
                    }
                    "html" => {
                        // View 插件：查找并执行后台入口脚本
                        let background_entry_file = manifest
                            .lifecycle
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("lifecycle.js");
                        let background_entry_path = path.join(background_entry_file);

                        if background_entry_path.is_file() {
                            plugins_to_init.push((
                                manifest.id.clone(),
                                background_entry_path,
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

    let plugins = store_lock.values().cloned().collect();
    drop(store_lock);

    // 执行所有插件的后台初始化入口
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

// ============================================================================
// Tauri 命令
// ============================================================================

/// 加载所有插件
///
/// 扫描插件目录并加载所有有效的插件
#[tauri::command]
pub fn load_plugins(
    app: tauri::AppHandle,
    store: State<PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    load_plugins_internal(&app, &store, true)
}

/// 获取已加载的插件列表
///
/// 返回当前内存中缓存的插件列表
#[tauri::command]
pub fn get_loaded_plugins(store: State<PluginStore>) -> Result<Vec<LoadedPlugin>, String> {
    let store_lock = store.0.lock().unwrap();
    let plugins = store_lock.values().cloned().collect();
    Ok(plugins)
}

/// 刷新插件列表
///
/// 清除运行时缓存并重新加载所有插件
#[tauri::command]
pub async fn refresh_plugins(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    // 清除 JavaScript 运行时缓存
    let _ = js_runtime::clear_all_plugin_runtimes().await;

    // 清除插件加载状态缓存
    let loaded_state = app.state::<crate::plugin_api::command::PluginLoadedState>();
    loaded_state.0.lock().unwrap().clear();

    // 重新加载所有插件
    load_plugins_internal(&app, &store, true)
}
