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

    // 居中定位在屏幕顶部偏下
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let x = (size.width as f64 / scale - 380.0) / 2.0;
        let y = 76.0;
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }

    Ok(())
}
