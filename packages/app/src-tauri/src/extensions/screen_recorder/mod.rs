use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub mod commands;
pub mod engine;

#[cfg(target_os = "windows")]
pub mod windows;

// 跨平台工厂函数，编译时选择具体引擎
pub fn create_platform_engine() -> Arc<dyn engine::ScreenRecordEngine> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(windows::WindowsRecordEngine::new())
    }
    #[cfg(not(target_os = "windows"))]
    {
        // 非 Windows 平台暂时使用 Dummy Engine 作为占位符，后续可逐步实现
        struct DummyEngine;
        impl engine::ScreenRecordEngine for DummyEngine {
            fn start(
                &self,
                _cfg: &engine::RecordConfig,
                _path: &std::path::Path,
                _app: &AppHandle,
            ) -> Result<(), String> {
                Err("当前平台录屏功能暂未支持".into())
            }
            fn pause(&self) -> Result<(), String> {
                Err("不支持".into())
            }
            fn resume(&self) -> Result<(), String> {
                Err("不支持".into())
            }
            fn stop(&self) -> Result<(), String> {
                Err("不支持".into())
            }
            fn get_state(&self) -> engine::RecordStateSnapshot {
                engine::RecordStateSnapshot {
                    state: engine::RecordState::Idle,
                    duration_secs: 0,
                }
            }
        }
        Arc::new(DummyEngine)
    }
}

pub static SCREEN_RECORDER_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "screen_recorder",
    name: "屏幕录制",
    description: "录制当前桌面屏幕和声音",
    icon: "video", // 使用现有的 video 图标
    commands: &[ExtensionCommand {
        code: "record",
        name: "启动录屏",
        description: Some("开启屏幕录像工具栏"),
        icon: Some("video"),
        keywords: &["rec", "record", "录像", "录屏"],
        matches: None,
    }],
};

pub struct ScreenRecorderExtension;

pub static SCREEN_RECORDER_EXTENSION: ScreenRecorderExtension = ScreenRecorderExtension;

impl Extension for ScreenRecorderExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &SCREEN_RECORDER_MANIFEST
    }

    fn execute(&self, _input: &str) -> ExtensionResult {
        // 在 CommandManager 执行内置扩展命令时，我们没有 AppHandle，
        // 但可以通过系统在其他地方异步拉起，或者直接返回成功指示，
        // 在这里由于没有 AppHandle，前端如果调用了 execute_extension，
        // 我们会通过主应用广播或命令来进行窗口创建。
        // 不过，在 execute_extension 中其实可以通过前端在匹配后，
        // 直接调用我们的 Tauri commands (例如 start_color_picker 那样) 来拉起。
        ExtensionResult {
            success: true,
            value: Some("start_toolbar".into()), // 告知前端，我们可以启动工具栏
            result_type: ExtensionResultType::Conversion,
            copyable: None,
            subtitle: None,
            error: None,
        }
    }

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}

/// 动态创建录屏悬浮工具栏窗口（专属子窗口）
pub fn show_screen_recorder_bar(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("screen-recorder-bar") {
        let _ = win.show();
        let _ = win.set_focus();
        let _ = try_show_area_indicator(app);
        return Ok(());
    }

    let builder = WebviewWindowBuilder::new(
        app,
        "screen-recorder-bar",
        WebviewUrl::App("/screen-recorder-bar".into()),
    )
    .title("录屏控制")
    .inner_size(380.0, 92.0) // 比控制条 360x72 稍大，提供透明边缘缓冲，消除系统投影白边
    .decorations(false) // 无边框
    .transparent(true) // 透明背景
    .visible(false) // 初始隐藏，等前端 onMount 就绪后主动唤醒显示，防白屏闪烁
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true) // 隐藏任务栏图标
    .shadow(false); // 消除矩形白色投影框

    let window = builder.build().map_err(|e| e.to_string())?;

    // 居中定位在当前主窗口所在的显示器上
    let mut target_monitor = None;
    if let Some(main_win) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = main_win.current_monitor() {
            target_monitor = Some(monitor);
        }
    }

    let monitor = match target_monitor {
        Some(m) => m,
        None => {
            if let Ok(Some(m)) = app.primary_monitor() {
                m
            } else {
                return Ok(());
            }
        }
    };

    let size = monitor.size();
    let scale = monitor.scale_factor();
    let position = monitor.position();

    // 转换成全局逻辑坐标
    let monitor_logical_x = position.x as f64 / scale;
    let monitor_logical_y = position.y as f64 / scale;

    let x = monitor_logical_x + (size.width as f64 / scale - 380.0) / 2.0;
    let y = monitor_logical_y + 76.0;
    let _ = window.set_position(tauri::LogicalPosition::new(x, y));

    // 同时开启区域红色指示边界框
    let _ = try_show_area_indicator(app);

    Ok(())
}

/// 开启录制区域红色指示边界框 (不阻挡鼠标操作，开启鼠标穿透)
pub fn try_show_area_indicator(app: &AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let state = app.state::<crate::extensions::screen_recorder::commands::RecorderAppState>();
    let config_guard = state.config.lock().map_err(|e| e.to_string())?;
    let config = config_guard.clone();

    if config.record_target_type.as_deref() != Some("area") {
        return Ok(());
    }

    let area = match config.area_rect {
        Some(a) => a,
        None => return Ok(()),
    };

    let monitor_index = config.monitor_index.unwrap_or(0);
    let monitors = app.available_monitors().unwrap_or_default();
    let monitor = if monitor_index >= 0 && (monitor_index as usize) < monitors.len() {
        monitors[monitor_index as usize].clone()
    } else {
        if let Ok(Some(m)) = app.primary_monitor() {
            m
        } else {
            return Ok(());
        }
    };

    let scale = monitor.scale_factor();
    let monitor_position = monitor.position();

    // 窗口全局逻辑坐标 = 屏幕全局逻辑坐标 + 区域逻辑坐标
    let monitor_logical_x = monitor_position.x as f64 / scale;
    let monitor_logical_y = monitor_position.y as f64 / scale;

    let win_x = monitor_logical_x + area.x as f64;
    let win_y = monitor_logical_y + area.y as f64;
    let win_w = area.width as f64;
    let win_h = area.height as f64;

    if let Some(w) = app.get_webview_window("screen-recorder-area-indicator") {
        let _ = w.close();
    }

    let window = WebviewWindowBuilder::new(
        app,
        "screen-recorder-area-indicator",
        WebviewUrl::App("/screen-recorder-area-indicator".into()),
    )
    .title("Recording Area Indicator")
    .transparent(true)
    .always_on_top(true)
    .decorations(false)
    .shadow(false)
    .skip_taskbar(true)
    .visible(false)
    .position(win_x, win_y)
    .inner_size(win_w, win_h)
    .build()
    .map_err(|e| e.to_string())?;

    // 计算真实的物理像素尺寸和物理坐标，在 build 成功后强行做二次物理纠偏
    let phys_rect_x = (area.x as f64 * scale) as i32;
    let phys_rect_y = (area.y as f64 * scale) as i32;
    let phys_rect_w = ((area.width as f64 * scale) as u32) & !1;
    let phys_rect_h = ((area.height as f64 * scale) as u32) & !1;

    let phys_win_x = monitor_position.x + phys_rect_x;
    let phys_win_y = monitor_position.y + phys_rect_y;

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
        phys_win_x, phys_win_y,
    )));
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
        phys_rect_w,
        phys_rect_h,
    )));

    // 设置忽略鼠标指令，强制鼠标穿透，不干涉用户日常交互
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.show();

    Ok(())
}

/// 动态创建区域选择遮罩窗口
pub fn show_screen_recorder_area(app: &AppHandle, monitor_index: i32) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("screen-recorder-area") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    let monitors = app.available_monitors().unwrap_or_default();
    let monitor = if monitor_index >= 0 && (monitor_index as usize) < monitors.len() {
        monitors[monitor_index as usize].clone()
    } else {
        if let Ok(Some(m)) = app.primary_monitor() {
            m
        } else {
            return Ok(());
        }
    };

    let size = monitor.size();
    let scale = monitor.scale_factor();
    let position = monitor.position();

    let monitor_logical_x = position.x as f64 / scale;
    let monitor_logical_y = position.y as f64 / scale;
    let monitor_logical_w = size.width as f64 / scale;
    let monitor_logical_h = size.height as f64 / scale;

    let builder = WebviewWindowBuilder::new(
        app,
        "screen-recorder-area",
        WebviewUrl::App("/screen-recorder-area".into()),
    )
    .title("选择录屏区域")
    .decorations(false) // 无边框
    .transparent(true) // 透明背景
    .visible(false) // 初始隐藏，等前端 onMount 就绪后显示
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true) // 隐藏任务栏图标
    .shadow(false)
    // 关键：在 build 窗口的第一微秒起就设置好位置和大小，彻底消除 800x600 默认窗口的拉伸延迟
    .position(monitor_logical_x, monitor_logical_y)
    .inner_size(monitor_logical_w, monitor_logical_h);

    let window = builder.build().map_err(|e| e.to_string())?;

    // 双重纠偏：强行赋以最可信的物理全屏尺寸
    let _ = window.set_position(tauri::Position::Physical(*position));
    let _ = window.set_size(tauri::Size::Physical(*size));

    // 延迟 180ms 在 Rust 后台让窗口显示，保证物理尺寸彻底适配且 Svelte 渲染完毕
    let win_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(180)).await;
        let _ = win_clone.show();
        let _ = win_clone.set_focus();
    });

    Ok(())
}
