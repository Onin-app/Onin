//! # 插件设置模块
//!
//! 负责插件的配置和设置管理，包括：
//! - 插件启用/禁用切换
//! - 插件自动分离设置
//! - 插件设置模式注册
//! - 插件设置的读写

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tauri::{Emitter, Manager, State};

use super::state::{
    collect_plugin_states, get_plugin_settings_path, load_plugin_settings,
    save_plugin_settings_to_file, save_plugin_states,
};
use super::types::{
    find_plugin_by_id, find_plugin_by_id_mut, InstallSource, LoadedPlugin, PluginDetail,
    PluginServerPort, PluginSettingsSchema, PluginStore,
};

// ============================================================================
// Tauri 命令 - 插件启用/禁用
// ============================================================================

/// 切换插件启用状态
///
/// 启用插件时，会自动禁用同一 manifest.id 的其他版本
///
/// # 参数
/// - `plugin_id`: 插件目录名（包含后缀，如 "translate@local"）
/// - `enabled`: 启用或禁用
#[tauri::command]
pub fn toggle_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    // 查找插件
    let plugin = store_lock
        .get_mut(&plugin_id)
        .ok_or_else(|| format!("插件未找到: {}", plugin_id))?;

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
                    "[plugin/settings] 自动禁用插件 {} 的 {} 版本",
                    manifest_id,
                    match other_plugin.install_source {
                        InstallSource::Local => "本地",
                        InstallSource::Marketplace => "市场",
                    }
                );
            }
        }
    }

    // 启用/禁用当前插件
    let plugin = store_lock.get_mut(&plugin_id).unwrap();
    plugin.enabled = enabled;
    println!(
        "[plugin/settings] 插件 {} ({}) 已{}",
        manifest_id,
        plugin_id,
        if enabled { "启用" } else { "禁用" }
    );

    // 收集所有插件的状态
    let states = collect_plugin_states(&store_lock);

    // 释放锁后再保存状态
    drop(store_lock);

    // 持久化保存状态
    if let Err(e) = save_plugin_states(&app, &states) {
        eprintln!("[plugin/settings] 保存插件状态失败: {}", e);
        return Err(format!("保存插件状态失败: {}", e));
    }

    Ok(())
}

/// 切换插件自动分离设置
///
/// 设置插件是否自动在独立窗口中打开
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
            "[plugin/settings] 插件 {} 的 auto_detach 已设为 {}",
            plugin_id, auto_detach
        );

        // 收集所有插件的状态
        let states = collect_plugin_states(&store_lock);

        // 释放锁后再保存状态
        drop(store_lock);

        // 持久化保存状态
        if let Err(e) = save_plugin_states(&app, &states) {
            eprintln!("[plugin/settings] 保存插件状态失败: {}", e);
            return Err(format!("保存插件状态失败: {}", e));
        }

        Ok(())
    } else {
        Err(format!("插件未找到: {}", plugin_id))
    }
}

/// 切换插件退出到后台立即结束运行设置
#[tauri::command]
pub fn toggle_plugin_terminate_on_bg(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    terminate_on_bg: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = find_plugin_by_id_mut(&mut store_lock, &plugin_id) {
        plugin.manifest.terminate_on_bg = terminate_on_bg;
        println!(
            "[plugin/settings] 插件 {} 的 terminate_on_bg 已设为 {}",
            plugin_id, terminate_on_bg
        );

        // 收集所有插件的状态
        let states = collect_plugin_states(&store_lock);

        // 释放锁后再保存状态
        drop(store_lock);

        // 持久化保存状态
        if let Err(e) = save_plugin_states(&app, &states) {
            eprintln!("[plugin/settings] 保存插件状态失败: {}", e);
            return Err(format!("保存插件状态失败: {}", e));
        }

        Ok(())
    } else {
        Err(format!("插件未找到: {}", plugin_id))
    }
}

/// 切换插件跟随主程序同时启动运行设置
#[tauri::command]
pub fn toggle_plugin_run_at_startup(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    run_at_startup: bool,
) -> Result<(), String> {
    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = find_plugin_by_id_mut(&mut store_lock, &plugin_id) {
        plugin.manifest.run_at_startup = run_at_startup;
        println!(
            "[plugin/settings] 插件 {} 的 run_at_startup 已设为 {}",
            plugin_id, run_at_startup
        );

        // 收集所有插件的状态
        let states = collect_plugin_states(&store_lock);

        // 释放锁后再保存状态
        drop(store_lock);

        // 持久化保存状态
        if let Err(e) = save_plugin_states(&app, &states) {
            eprintln!("[plugin/settings] 保存插件状态失败: {}", e);
            return Err(format!("保存插件状态失败: {}", e));
        }

        Ok(())
    } else {
        Err(format!("插件未找到: {}", plugin_id))
    }
}

// ============================================================================
// Tauri 命令 - 设置模式管理
// ============================================================================

/// 注册插件设置模式
///
/// 由插件生命周期脚本调用，注册可配置的设置项
#[tauri::command]
pub fn register_plugin_settings_schema(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
    schema: PluginSettingsSchema,
) -> Result<(), String> {
    println!(
        "[plugin/settings] 注册插件设置模式: {}，字段数: {}",
        plugin_id,
        schema.fields.len()
    );

    let mut store_lock = store.0.lock().unwrap();

    if let Some(plugin) = find_plugin_by_id_mut(&mut store_lock, &plugin_id) {
        // 将设置模式存储在 LoadedPlugin 中（而非 manifest）
        plugin.settings = Some(schema.clone());
        println!(
            "[plugin/settings] ✅ 已注册插件 {} 的设置模式: {} 个字段",
            plugin_id,
            schema.fields.len()
        );

        // 发送事件通知前端设置模式已注册
        if let Err(e) = app.emit("plugin-settings-schema-registered", &plugin_id) {
            eprintln!("[plugin/settings] 发送设置模式注册事件失败: {}", e);
            return Err(format!("发送事件失败: {}", e));
        }

        Ok(())
    } else {
        Err(format!("插件未找到: {}", plugin_id))
    }
}

// ============================================================================
// Tauri 命令 - 设置读写
// ============================================================================

/// 获取插件设置
///
/// 从文件中读取指定插件的用户设置
#[tauri::command]
pub fn get_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
) -> Result<HashMap<String, JsonValue>, String> {
    load_plugin_settings(&app, &plugin_id)
}

/// 保存插件设置
///
/// 将用户设置写入文件
#[tauri::command]
pub fn save_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
    settings: HashMap<String, JsonValue>,
) -> Result<(), String> {
    save_plugin_settings_to_file(&app, &plugin_id, &settings)
}

// ============================================================================
// Tauri 命令 - 插件信息查询
// ============================================================================

/// 获取带设置模式的插件信息
#[tauri::command]
pub fn get_plugin_with_schema(
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<LoadedPlugin, String> {
    let store_lock = store.0.lock().unwrap();

    find_plugin_by_id(&store_lock, &plugin_id)
        .cloned()
        .ok_or_else(|| format!("插件未找到: {}", plugin_id))
}

/// 获取插件详情
///
/// 返回插件的完整信息，包括 README 内容
#[tauri::command]
pub fn get_plugin_detail(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<PluginDetail, String> {
    let store_lock = store.0.lock().unwrap();

    let plugin = find_plugin_by_id(&store_lock, &plugin_id)
        .cloned()
        .ok_or_else(|| format!("插件未找到: {}", plugin_id))?;

    // 读取 README.md
    let plugins_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?
        .join("plugins");

    let plugin_dir = plugins_dir.join(&plugin.dir_name);
    let readme_path = plugin_dir.join("README.md");

    let readme = if readme_path.exists() {
        std::fs::read_to_string(&readme_path).ok()
    } else {
        None
    };

    Ok(PluginDetail { plugin, readme })
}

/// 获取插件服务器端口
#[tauri::command]
pub fn get_plugin_server_port(app: tauri::AppHandle) -> Result<u16, String> {
    let server_port_state = app.state::<PluginServerPort>();
    let port = server_port_state
        .0
        .lock()
        .unwrap()
        .ok_or_else(|| "插件服务器未启动".to_string())?;
    Ok(port)
}
