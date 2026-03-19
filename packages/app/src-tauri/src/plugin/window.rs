//! # 插件窗口管理模块
//!
//! 负责插件窗口的创建、显示、隐藏、销毁，包括：
//! - 窗口基本控制（最小化、最大化、关闭等）
//! - 窗口状态持久化
//! - ESC 快捷键管理
//! - 窗口切换防抖

use std::str::FromStr;
use tauri::{Emitter, Manager, State, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use super::state::{load_plugin_window_states, save_plugin_window_state};
use super::types::{
    find_plugin_by_id, ActivePluginWindow, LoadedPlugin, PluginStore, PluginWindowCreating,
    PluginWindowToggleDebounce, WindowBounds,
};

/// 编译期嵌入的 FAB 悬浮菜单脚本（CSS + HTML + JS）
/// 源文件：src-tauri/templates/plugin-window-fab.js
const PLUGIN_WINDOW_FAB_SCRIPT: &str = include_str!("../../templates/plugin-window-fab.js");

// ============================================================================
// 窗口基本控制命令
// ============================================================================

/// 关闭插件窗口
#[tauri::command]
pub fn plugin_close_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 最小化插件窗口
#[tauri::command]
pub fn plugin_minimize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.minimize().map_err(|e| e.to_string())?;
        // 触发窗口隐藏事件
        trigger_window_visibility_event(&window, false);
        Ok(())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 最大化插件窗口
#[tauri::command]
pub fn plugin_maximize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.maximize().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 取消最大化插件窗口
#[tauri::command]
pub fn plugin_unmaximize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 检查插件窗口是否最大化
#[tauri::command]
pub fn plugin_is_maximized(app: tauri::AppHandle, label: String) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.is_maximized().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 显示插件窗口
#[tauri::command]
pub fn plugin_show_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        // 触发窗口显示事件
        trigger_window_visibility_event(&window, true);
        Ok(())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 取消最小化插件窗口（恢复窗口）
#[tauri::command]
pub fn plugin_unminimize_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.unminimize().map_err(|e| e.to_string())?;
        // 触发窗口显示事件
        trigger_window_visibility_event(&window, true);
        Ok(())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

/// 聚焦插件窗口
#[tauri::command]
pub fn plugin_set_focus(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
}

#[tauri::command]
pub fn plugin_open_devtools(window: tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(debug_assertions)]
    window.open_devtools();
    Ok(())
}

// ============================================================================
// 窗口创建和管理
// ============================================================================

/// 在独立窗口中打开插件
#[tauri::command]
pub fn open_plugin_in_window(
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

    // 仅在非 dev_mode（无 dev_server）时校验本地入口文件是否存在
    // dev_mode 插件直接从 dev_server 加载，不需要本地文件
    let is_dev_mode = plugin.manifest.dev_mode && plugin.manifest.dev_server.is_some();
    if !is_dev_mode {
        let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
        let entry_path = plugin_dir.join(&plugin.manifest.entry);

        if !entry_path.is_file() {
            return Err(format!("插件入口文件未找到: {:?}", entry_path));
        }
    }

    // 强制在窗口模式中打开
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = create_or_show_plugin_window(app_clone, &plugin).await {
            eprintln!("创建或显示插件窗口失败: {}", e);
        }
    });

    Ok(())
}

// ============================================================================
// 窗口相关操作（例如：回到内联模式等）
// ============================================================================

/// 从窗口模式恢复到内联模式
#[tauri::command]
pub fn return_to_inline_from_window(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    // 1. 获取插件信息
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        find_plugin_by_id(&store_lock, &plugin_id).cloned()
    }
    .ok_or_else(|| format!("插件未找到: {}", plugin_id))?;

    // 2. 构建插件 URL
    let plugin_url = if plugin.manifest.dev_mode && plugin.manifest.dev_server.is_some() {
        plugin.manifest.dev_server.as_ref().unwrap().clone()
    } else {
        let port = {
            let port_state = app.state::<super::types::PluginServerPort>();
            let guard = port_state.0.lock().unwrap();
            *guard
        }
        .ok_or_else(|| "插件服务器未启动".to_string())?;

        let entry = plugin.manifest.entry.trim_start_matches('/');
        format!(
            "http://127.0.0.1:{}/plugin/{}/{}",
            port, plugin.dir_name, entry
        )
    };

    // 3. 关闭弹出的大窗口
    let window_label = crate::plugin::context::make_label_from_plugin_id(&plugin.manifest.id);
    let _ = plugin_close_window(app.clone(), window_label);

    // 4. 确保主窗口重新显示并获得焦点，否则用户会感觉“没有切回来”
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }

    // 5. 通知主窗口显示内联插件
    super::executor::show_plugin_inline(&app, &plugin, plugin_url)
}

/// 切换插件窗口置顶状态
#[tauri::command]
pub fn plugin_toggle_window_pin(
    app: tauri::AppHandle,
    plugin_id: String,
    pin: bool,
) -> Result<(), String> {
    let window_label = crate::plugin::context::make_label_from_plugin_id(&plugin_id);
    if let Some(window) = app.get_webview_window(&window_label) {
        window.set_always_on_top(pin).map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", window_label))
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 触发窗口可见性事件
///
/// 向插件窗口发送可见性变化事件
pub fn trigger_window_visibility_event(window: &tauri::WebviewWindow, is_visible: bool) {
    use tauri::Emitter;

    // 发送具体的 show/hide 事件
    if is_visible {
        if let Err(e) = window.emit("window_show", ()) {
            eprintln!("[plugin/window] 发送 window_show 事件失败: {}", e);
        }
    } else {
        if let Err(e) = window.emit("window_hide", ()) {
            eprintln!("[plugin/window] 发送 window_hide 事件失败: {}", e);
        }
    }

    // 同时发送通用的 visibility 事件（带 payload）
    if let Err(e) = window.emit("window_visibility", is_visible) {
        eprintln!("[plugin/window] 发送 window_visibility 事件失败: {}", e);
    }
}

/// 创建或显示插件窗口
///
/// 如果窗口已存在，切换其显示状态；否则创建新窗口
pub async fn create_or_show_plugin_window(
    app: tauri::AppHandle,
    plugin: &LoadedPlugin,
) -> Result<(), String> {
    let window_label = crate::plugin::context::make_label_from_plugin_id(&plugin.manifest.id);

    // 防抖检查：防止短时间内重复触发
    const DEBOUNCE_MS: u64 = 100; // 100ms 防抖时间
    if let Some(debounce_state) = app.try_state::<PluginWindowToggleDebounce>() {
        let mut debounce_map = debounce_state.0.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some(last_toggle) = debounce_map.get(&window_label) {
            let elapsed = now.duration_since(*last_toggle).as_millis() as u64;
            if elapsed < DEBOUNCE_MS {
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
            return Ok(());
        }
    }

    // 如果窗口已存在，切换显示状态
    // 语义要求：
    // - focus != visibility：可见窗口失焦时，重新激活只应触发 focus，不应额外触发 show
    // - show/hide 只在真实可见性变化时触发
    if let Some(window) = app.get_webview_window(&window_label) {
        let is_visible = window.is_visible().unwrap_or(true);
        let is_minimized = window.is_minimized().unwrap_or(false);
        let is_focused = window.is_focused().unwrap_or(false);

        if !is_visible || is_minimized {
            crate::focus_manager::capture_previous_foreground(&app);
            let _ = window.show();
            let _ = window.unminimize();
            crate::focus_manager::focus_webview_window(&window);

            trigger_window_visibility_event(&window, true);
        } else if !is_focused {
            crate::focus_manager::capture_previous_foreground(&app);
            crate::focus_manager::focus_webview_window(&window);
        } else {
            crate::focus_manager::restore_previous_foreground(&app);

            if let Err(e) = window.minimize() {
                eprintln!("最小化插件窗口失败: {}", e);
            }

            trigger_window_visibility_event(&window, false);
        }
        return Ok(());
    }

    // 构建窗口 URL
    let plugin_url = if plugin.manifest.dev_mode && plugin.manifest.dev_server.is_some() {
        let dev_server = plugin.manifest.dev_server.as_ref().unwrap();
        // 如果 devServer 已经有 query params，使用 & 连接
        let separator = if dev_server.contains('?') { '&' } else { '?' };
        let url = format!(
            "{}{}mode=window&plugin_id={}",
            dev_server, separator, plugin.manifest.id
        );
        url
    } else {
        // 获取插件服务器端口 (仅在非 Dev Mode 或 Dev Mode 无 Server 时需要)
        let port = {
            let port_state = app.state::<super::types::PluginServerPort>();
            let guard = port_state.0.lock().unwrap();
            *guard
        }
        .ok_or_else(|| "插件服务器未启动".to_string())?;

        // 直接加载插件文件，绕过 plugin-window 包装页
        let entry = plugin.manifest.entry.trim_start_matches('/');
        let url = format!(
            "http://127.0.0.1:{}/plugin/{}/{}?mode=window&plugin_id={}",
            port,
            plugin.dir_name, // Use dir_name to match actual directory on disk (e.g. with @market suffix)
            entry,
            plugin.manifest.id
        );
        url
    };

    // 注入运行时信息及 FAB 悬浮菜单
    // FAB 脚本从外部模板文件编译期嵌入，详见 templates/plugin-window-fab.js
    let runtime_script = format!(
        r#"
        window.__ONIN_RUNTIME__ = {{
            mode: "window",
            pluginId: "{id}",
            version: "{ver}",
            mainWindowLabel: "main"
        }};
        window.__PLUGIN_ID__ = "{id}";        {fab}
        "#,
        id = plugin.manifest.id,
        ver = plugin.manifest.version,
        fab = PLUGIN_WINDOW_FAB_SCRIPT,
    );

    // 加载保存的窗口状态
    let window_states = load_plugin_window_states(&app);
    let saved_bounds = window_states.get(&plugin.manifest.id).cloned();

    // 创建窗口构建器
    let mut builder = WebviewWindowBuilder::new(
        &app,
        window_label.clone(),
        tauri::WebviewUrl::External(
            plugin_url
                .parse()
                .map_err(|e: url::ParseError| e.to_string())?,
        ),
    )
    .title(plugin.manifest.name.clone())
    .resizable(true)
    .decorations(true) // 使用系统原生装饰
    .transparent(false) // 确保窗口不透明
    .initialization_script(&runtime_script);

    // 应用保存的窗口位置和大小
    if let Some(ref bounds) = saved_bounds {
        // 严格检查保存的边界是否合理
        let is_bounds_valid = bounds.x.abs() < 10000
            && bounds.y.abs() < 10000
            && bounds.width >= 200
            && bounds.width <= 3000
            && bounds.height >= 200
            && bounds.height <= 2000;

        if is_bounds_valid && !bounds.is_maximized {
            builder = builder
                .position(bounds.x as f64, bounds.y as f64)
                .inner_size(bounds.width as f64, bounds.height as f64);
        } else if bounds.is_maximized {
            builder = builder.inner_size(800.0, 600.0);
        } else {
            builder = builder.inner_size(800.0, 600.0);
        }
    } else {
        builder = builder.inner_size(800.0, 600.0);
    }

    // 标记窗口正在创建
    if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
        let mut creating = creating_state.0.lock().unwrap();
        creating.insert(window_label.clone());
    }

    match builder.build() {
        Ok(window) => {
            // 移除创建标记
            if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
                let mut creating = creating_state.0.lock().unwrap();
                creating.remove(&window_label);
            }

            // 如果之前是最大化的，恢复最大化状态
            if let Some(ref bounds) = saved_bounds {
                if bounds.is_maximized {
                    if let Err(e) = window.maximize() {
                        eprintln!("最大化窗口失败: {}", e);
                    }
                }
            }

            // 设置 ESC 快捷键和窗口事件
            setup_window_events(&app, &window, plugin.manifest.id.clone());

            Ok(())
        }
        Err(e) => {
            // 移除创建标记
            if let Some(creating_state) = app.try_state::<PluginWindowCreating>() {
                let mut creating = creating_state.0.lock().unwrap();
                creating.remove(&window_label);
            }
            eprintln!("构建插件窗口失败: {}", e);
            Err(format!("构建插件窗口失败: {}", e))
        }
    }
}

/// 设置窗口事件监听
///
/// 包括焦点变化、ESC 快捷键注册/注销、窗口关闭时保存状态
fn setup_window_events(app: &tauri::AppHandle, window: &tauri::WebviewWindow, plugin_id: String) {
    let esc_shortcut = Shortcut::from_str("escape").unwrap();
    let app_for_window_event = app.clone();
    let window_label_for_tracking = window.label().to_string();

    // 设置窗口焦点
    if let Err(e) = window.set_focus() {
        eprintln!("设置插件窗口焦点失败: {}", e);
    }

    // 立即记录活跃窗口并注册 ESC 快捷键
    let app_for_immediate = app.clone();
    let label_for_immediate = window_label_for_tracking.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if let Some(active_window_state) = app_for_immediate.try_state::<ActivePluginWindow>() {
            if let Ok(mut active) = active_window_state.0.lock() {
                *active = Some(label_for_immediate.clone());
            }
        }

        let _ = app_for_immediate.global_shortcut().unregister(esc_shortcut);
        if let Err(e) = app_for_immediate.global_shortcut().register(esc_shortcut) {
            eprintln!("[plugin/window] 注册 ESC 快捷键失败: {}", e);
        }
    });

    let label_for_event = window_label_for_tracking.clone();
    let window_for_event = window.clone();
    let plugin_id_for_event = plugin_id.clone();
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

                // 只发送焦点事件，不发送可见性事件
                // 焦点变化不等于可见性变化：窗口可以失去焦点但仍然可见
                if let Err(e) = window_for_event.emit("window_focus", ()) {
                    eprintln!("[plugin/window] 发送 window_focus 事件失败: {}", e);
                }

                // 重新注册 ESC 快捷键
                let _ = app_for_window_event
                    .global_shortcut()
                    .unregister(esc_shortcut);
                if let Err(e) = app_for_window_event
                    .global_shortcut()
                    .register(esc_shortcut)
                {
                    eprintln!("[plugin/window] 注册 ESC 快捷键失败: {}", e);
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

                // 只发送失焦事件，不发送可见性事件
                // 焦点变化不等于可见性变化：窗口可以失去焦点但仍然可见
                if let Err(e) = window_for_event.emit("window_blur", ()) {
                    eprintln!("[plugin/window] 发送 window_blur 事件失败: {}", e);
                }

                // 注销 ESC 快捷键
                if let Err(e) = app_for_window_event
                    .global_shortcut()
                    .unregister(esc_shortcut)
                {
                    eprintln!("注销 ESC 快捷键失败: {}", e);
                }
            }
            tauri::WindowEvent::CloseRequested { .. } => {
                // 保存窗口状态
                save_window_state_on_close(&window_for_event, &app_for_save, &plugin_id_for_event);

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
}

/// 窗口关闭时保存状态
fn save_window_state_on_close(
    window: &tauri::WebviewWindow,
    app: &tauri::AppHandle,
    plugin_id: &str,
) {
    if let Ok(position) = window.outer_position() {
        if let Ok(size) = window.inner_size() {
            if let Ok(is_maximized) = window.is_maximized() {
                // 获取缩放因子，将物理像素转换为逻辑像素
                let scale_factor = window.scale_factor().unwrap_or(1.0);
                let logical_width = (size.width as f64 / scale_factor) as u32;
                let logical_height = (size.height as f64 / scale_factor) as u32;
                let logical_x = (position.x as f64 / scale_factor) as i32;
                let logical_y = (position.y as f64 / scale_factor) as i32;

                // 边界检查
                let is_bounds_valid = logical_x.abs() < 10000
                    && logical_y.abs() < 10000
                    && logical_width >= 200
                    && logical_width <= 3000
                    && logical_height >= 200
                    && logical_height <= 2000;

                if is_bounds_valid || is_maximized {
                    let bounds = WindowBounds {
                        x: logical_x,
                        y: logical_y,
                        width: logical_width,
                        height: logical_height,
                        is_maximized,
                    };

                    // 异步保存，避免阻塞窗口关闭
                    let app_clone = app.clone();
                    let plugin_id_clone = plugin_id.to_string();
                    std::thread::spawn(move || {
                        if let Err(e) =
                            save_plugin_window_state(&app_clone, &plugin_id_clone, bounds)
                        {
                            eprintln!("[plugin/window] 关闭时保存窗口状态失败: {}", e);
                        }
                    });
                } else {
                }
            }
        }
    }
}
