//! # 插件窗口管理模块
//!
//! 负责插件窗口的创建、显示、隐藏、销毁，包括：
//! - 窗口基本控制（关闭、聚焦）
//! - 窗口状态持久化
//! - ESC 快捷键管理
//! - 窗口切换防抖

use std::str::FromStr;
use tauri::{Emitter, Manager, State, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use super::state::{load_plugin_window_states, save_plugin_window_state};
use super::types::{
    find_plugin_by_id, ActivePluginWindow, LoadedPlugin, PluginStore,
    PluginWindowCreating, PluginWindowToggleDebounce, WindowBounds,
};

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

/// 聚焦插件窗口
#[tauri::command]
pub fn plugin_set_focus(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|e| e.to_string())
    } else {
        Err(format!("窗口未找到: {}", label))
    }
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

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("插件入口文件未找到: {:?}", entry_path));
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
                    "[plugin/window] 窗口 {} 切换被防抖（距上次切换 {}ms）",
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
                "[plugin/window] 窗口 {} 正在创建中，跳过",
                window_label
            );
            return Ok(());
        }
    }

    // 如果窗口已存在，切换显示状态
    // 核心逻辑：最小化/隐藏的窗口必然没有焦点，所以只需检查 is_focused
    if let Some(window) = app.get_webview_window(&window_label) {
        let is_focused = window.is_focused().unwrap_or(false);

        println!(
            "[plugin/window] 窗口 {} 焦点状态: {}",
            window_label, is_focused
        );

        if !is_focused {
            // 窗口无焦点（可能最小化、隐藏或在后台），恢复并聚焦
            println!("[plugin/window] 显示并聚焦窗口 {}", window_label);
            
            // 依次尝试恢复窗口状态
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();

            trigger_window_visibility_event(&window, true);
        } else {
            // 窗口已聚焦，最小化它
            println!("[plugin/window] 最小化窗口 {}", window_label);
            if let Err(e) = window.minimize() {
                eprintln!("最小化插件窗口失败: {}", e);
            }

            trigger_window_visibility_event(&window, false);
        }
        return Ok(());
    }

    // 构建窗口 URL - 直接指向插件 Entry
    let plugin_url = if plugin.manifest.dev_mode && plugin.manifest.dev_server.is_some() {
        // 开发模式：使用开发服务器
        let mut url = reqwest::Url::parse(plugin.manifest.dev_server.as_ref().unwrap())
            .map_err(|e| format!("无效的开发服务器 URL: {}", e))?;
            
        // 添加必要的查询参数
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("plugin_id", &plugin.manifest.id);
            pairs.append_pair("mode", "window");
        }
        
        println!("[plugin/window] 开发模式: 从 {} 加载", url);
        url.to_string()
    } else {


        // 注意：这里需要确保插件服务器已经启动并且可以被访问
        // 如果我们使用的是自定义协议或者 asset server，URL 格式可能不同
        // 目前假设插件服务器运行在某个端口，或者通过 tauri 协议访问

        // TODO: 这里需要根据实际的插件服务器逻辑来构建 URL
        // 暂时回退到使用 load_plugin_url 类似的逻辑，或者复用之前的 plugin server port
        
        let port = match app.try_state::<crate::plugin::PluginServerPort>() {
            Some(state) => state.0.lock().unwrap().unwrap_or(0),
            None => 0,
        };
        
        if port == 0 {
             return Err("插件服务器未启动".to_string());
        }

        format!(
            "http://127.0.0.1:{}/plugin/{}/{}?mode=window&plugin_id={}",
            port,
            plugin.dir_name,
            plugin.manifest.entry,
            plugin.manifest.id
        )
    };
    
    println!(
        "[plugin/window] 从 {} 加载插件窗口",
        plugin_url
    );

    // 加载保存的窗口状态
    let window_states = load_plugin_window_states(&app);
    let saved_bounds = window_states.get(&plugin.manifest.id).cloned();
    
    // 创建窗口构建器
    let mut builder = WebviewWindowBuilder::new(
        &app,
        window_label.clone(),
        tauri::WebviewUrl::External(plugin_url.parse().unwrap()),
    )
    .title(plugin.manifest.name.clone())
    .resizable(true)
    .decorations(true) // 启用原生装饰
    .devtools(true) // 启用开发者工具
    .transparent(false); // 确保窗口不透明
    
    // 应用保存的窗口位置和大小
    if let Some(ref bounds) = saved_bounds {
        println!(
            "[plugin/window] 恢复窗口边界 {}：x={}, y={}, width={}, height={}, maximized={}",
            plugin.manifest.id, bounds.x, bounds.y, bounds.width, bounds.height, bounds.is_maximized
        );

        // 严格检查保存的边界是否合理
        let is_bounds_valid = bounds.x.abs() < 10000
            && bounds.y.abs() < 10000
            && bounds.width >= 200 && bounds.width <= 3000
            && bounds.height >= 200 && bounds.height <= 2000;

        if is_bounds_valid && !bounds.is_maximized {
            builder = builder
                .position(bounds.x as f64, bounds.y as f64)
                .inner_size(bounds.width as f64, bounds.height as f64);
        } else if bounds.is_maximized {
            builder = builder.inner_size(800.0, 600.0);
        } else {
            println!(
                "[plugin/window] ⚠️ 保存的边界无效（width={}, height={}），使用默认大小",
                bounds.width, bounds.height
            );
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
                    println!("[plugin/window] 恢复 {} 的最大化状态", plugin.manifest.id);
                    if let Err(e) = window.maximize() {
                        eprintln!("最大化窗口失败: {}", e);
                    }
                }
            }

            // 设置 ESC 快捷键和窗口事件
            setup_window_events(
                &app,
                &window,
                plugin.manifest.id.clone(),
            );

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
fn setup_window_events(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    plugin_id: String,
) {
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
                println!("[plugin/window] 设置活跃插件窗口: {}", label_for_immediate);
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
                if let Some(active_window_state) = app_for_window_event.try_state::<ActivePluginWindow>() {
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
                let _ = app_for_window_event.global_shortcut().unregister(esc_shortcut);
                if let Err(e) = app_for_window_event.global_shortcut().register(esc_shortcut) {
                    eprintln!("[plugin/window] 注册 ESC 快捷键失败: {}", e);
                }
            }
            tauri::WindowEvent::Focused(false) => {
                // 清除活跃窗口记录
                if let Some(active_window_state) = app_for_window_event.try_state::<ActivePluginWindow>() {
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
                if let Err(e) = app_for_window_event.global_shortcut().unregister(esc_shortcut) {
                    eprintln!("注销 ESC 快捷键失败: {}", e);
                }
            }
            tauri::WindowEvent::CloseRequested { .. } => {
                // 保存窗口状态
                save_window_state_on_close(
                    &window_for_event,
                    &app_for_save,
                    &plugin_id_for_event,
                );
                
                // 清除活跃窗口记录
                if let Some(active_window_state) = app_for_window_event.try_state::<ActivePluginWindow>() {
                    if let Ok(mut active) = active_window_state.0.lock() {
                        if active.as_ref() == Some(&label_for_event) {
                            *active = None;
                        }
                    }
                }

                let _ = app_for_window_event.global_shortcut().unregister(esc_shortcut);
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
                
                println!(
                    "[plugin/window] 窗口关闭 - 物理: {}x{}，缩放: {}，逻辑: {}x{}",
                    size.width, size.height, scale_factor, logical_width, logical_height
                );
                
                // 边界检查
                let is_bounds_valid = logical_x.abs() < 10000
                    && logical_y.abs() < 10000
                    && logical_width >= 200 && logical_width <= 3000
                    && logical_height >= 200 && logical_height <= 2000;

                if is_bounds_valid || is_maximized {
                    let bounds = WindowBounds {
                        x: logical_x,
                        y: logical_y,
                        width: logical_width,
                        height: logical_height,
                        is_maximized,
                    };
                    
                    println!(
                        "[plugin/window] 保存窗口状态: x={}, y={}, width={}, height={}, maximized={}",
                        bounds.x, bounds.y, bounds.width, bounds.height, bounds.is_maximized
                    );
                    
                    // 异步保存，避免阻塞窗口关闭
                    let app_clone = app.clone();
                    let plugin_id_clone = plugin_id.to_string();
                    std::thread::spawn(move || {
                        if let Err(e) = save_plugin_window_state(&app_clone, &plugin_id_clone, bounds) {
                            eprintln!("[plugin/window] 关闭时保存窗口状态失败: {}", e);
                        }
                    });
                } else {
                    println!(
                        "[plugin/window] ⚠️ 跳过保存无效边界: x={}, y={}, width={}, height={}",
                        logical_x, logical_y, logical_width, logical_height
                    );
                }
            }
        }
    }
}
