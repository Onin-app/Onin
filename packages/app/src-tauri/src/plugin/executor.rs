//! # 插件执行器模块
//!
//! 负责插件入口的执行，包括：
//! - 判断插件类型（JS/HTML）
//! - 决定显示模式（内联/窗口）
//! - 执行 JS 插件或加载 HTML 插件

use std::path::Path;
use serde::Serialize;
use tauri::{Emitter, Manager, State};

use super::types::{find_plugin_by_id, PluginServerPort, PluginStore};
use super::window::create_or_show_plugin_window;
use crate::js_runtime;

// ============================================================================
// Tauri 命令
// ============================================================================

/// 执行插件入口
///
/// 根据插件类型和配置，决定如何执行：
/// - JS 插件：在后台执行 JavaScript 代码
/// - HTML 插件：在窗口或内联模式中显示
///
/// # 参数
/// - `plugin_id`: 插件唯一标识符
#[tauri::command]
pub fn execute_plugin_entry(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {

    // 尽快释放锁，克隆插件数据
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        find_plugin_by_id(&store_lock, &plugin_id).cloned()
    }
    .ok_or_else(|| format!("插件未找到: {}", plugin_id))?;

    // 调试日志

    // 检查插件是否启用
    if !plugin.enabled {
        return Err("插件已禁用".to_string());
    }

    // 开发模式处理
    if plugin.manifest.dev_mode {
        return execute_dev_mode_plugin(&app, &plugin);
    }

    // 生产模式：检查本地文件
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("插件入口文件未找到: {:?}", entry_path));
    }

    // 根据文件扩展名决定执行方式
    if let Some(extension) = Path::new(&plugin.manifest.entry)
        .extension()
        .and_then(|s| s.to_str())
    {
        match extension {
            "js" => execute_js_plugin(&app, &plugin, entry_path),
            "html" => execute_html_plugin(&app, &plugin),
            _ => Err(format!("不支持的插件入口类型: {}", extension)),
        }
    } else {
        Err("插件入口文件没有扩展名".to_string())
    }
}

// ============================================================================
// 执行辅助函数
// ============================================================================

/// 开发模式插件执行
fn execute_dev_mode_plugin(
    app: &tauri::AppHandle,
    plugin: &super::types::LoadedPlugin,
) -> Result<(), String> {
    if let Some(dev_server) = &plugin.manifest.dev_server {

        // 判断是否在窗口中打开
        let should_open_in_window =
            plugin.manifest.auto_detach || plugin.manifest.display_mode.as_str() == "window";

        if should_open_in_window {
            let app_clone = app.clone();
            let plugin_clone = plugin.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = create_or_show_plugin_window(app_clone, &plugin_clone).await {
                    eprintln!("创建或显示插件窗口失败: {}", e);
                }
            });
            Ok(())
        } else {
            // 内联模式
            show_plugin_inline(app, plugin, dev_server.clone())
        }
    } else {
        Err("插件设置了 devMode=true 但未指定 devServer".to_string())
    }
}

/// 执行 JS 插件
fn execute_js_plugin(
    app: &tauri::AppHandle,
    plugin: &super::types::LoadedPlugin,
    entry_path: std::path::PathBuf,
) -> Result<(), String> {

    // 读取 JS 代码
    let js_code = std::fs::read_to_string(&entry_path).map_err(|e| e.to_string())?;
    let app_clone = app.clone();
    let plugin_id = plugin.manifest.id.clone();

    // 在后台线程中执行
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            if let Err(e) = js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await {
                eprintln!("执行 Headless 插件失败: {}", e);
            }
        });
    });

    Ok(())
}

/// 执行 HTML 插件
fn execute_html_plugin(
    app: &tauri::AppHandle,
    plugin: &super::types::LoadedPlugin,
) -> Result<(), String> {
    // 判断是否在窗口中打开
    let should_open_in_window =
        plugin.manifest.auto_detach || plugin.manifest.display_mode.as_str() == "window";

    if should_open_in_window {
        // 在新窗口中打开
        let app_clone = app.clone();
        let plugin_clone = plugin.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = create_or_show_plugin_window(app_clone, &plugin_clone).await {
                eprintln!("创建或显示插件窗口失败: {}", e);
            }
        });
        Ok(())
    } else {
        // 内联显示 - 使用 HTTP 服务器
        let server_port_state = app.state::<PluginServerPort>();
        let port = server_port_state
            .0
            .lock()
            .unwrap()
            .ok_or_else(|| "插件服务器未启动".to_string())?;

        let plugin_url = format!(
            "http://127.0.0.1:{}/plugin/{}/{}",
            port, plugin.dir_name, plugin.manifest.entry
        );

        show_plugin_inline(app, plugin, plugin_url)
    }
}

/// 在主窗口内联显示插件
pub fn show_plugin_inline(
    app: &tauri::AppHandle,
    plugin: &super::types::LoadedPlugin,
    plugin_url: String,
) -> Result<(), String> {
    // 尝试恢复主窗口可见性；失败不应阻塞插件打开流程。
    if let Some(main_window) = app
        .get_webview_window("main")
        .or_else(|| app.webview_windows().values().next().cloned())
    {
        if let Ok(false) = main_window.is_visible() {
            if let Err(e) = main_window.show() {
                eprintln!("[plugin/executor] 警告: 显示主窗口失败: {}", e);
            }
            if let Err(e) = main_window.set_focus() {
                eprintln!("[plugin/executor] 警告: 聚焦主窗口失败: {}", e);
            }
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
        plugin_url,
    };

    app
        .emit("show_plugin_inline", payload)
        .map_err(|e| format!("发送 show_plugin_inline 事件失败: {}", e))?;

    Ok(())
}

