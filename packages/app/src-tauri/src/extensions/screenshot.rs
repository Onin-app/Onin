//! Region screenshot extension.
//!
//! The selection window is transparent and never receives image pixels. This
//! keeps the launch path light; native capture happens only after confirmation.

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult,
};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, RgbaImage};
use serde::Serialize;
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Manager, Monitor, WebviewUrl, WebviewWindowBuilder};

const OVERLAY_LABEL: &str = "screenshot-selection";

pub static SCREENSHOT_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "screenshot",
    name: "截图",
    description: "框选屏幕区域并复制或保存",
    icon: "camera",
    commands: &[ExtensionCommand {
        code: "capture",
        name: "截图",
        description: Some("框选屏幕区域并复制或保存"),
        icon: Some("camera"),
        keywords: &["screenshot", "screen capture", "截图", "截屏"],
        matches: None,
    }],
};

pub struct ScreenshotExtension;
pub static SCREENSHOT_EXTENSION: ScreenshotExtension = ScreenshotExtension;

impl Extension for ScreenshotExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &SCREENSHOT_MANIFEST
    }

    fn execute(&self, _input: &str) -> ExtensionResult {
        ExtensionResult::error("截图命令必须通过截图流程执行".to_string())
    }

    fn execute_command(&self, command_code: &str, input: &str) -> ExtensionResult {
        match command_code {
            "capture" => self.execute(input),
            _ => ExtensionResult::error(format!("未知命令: {command_code}")),
        }
    }

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}

#[derive(Clone)]
struct ScreenshotSession {
    monitor: Monitor,
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
}

static ACTIVE_SESSION: LazyLock<Mutex<Option<ScreenshotSession>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOverlayInfo {
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[tauri::command]
pub async fn start_screenshot_selection(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let main_window = app.get_webview_window("main");
        let monitor = main_window
            .as_ref()
            .and_then(|window| window.current_monitor().ok().flatten())
            .or_else(|| app.primary_monitor().ok().flatten())
            .ok_or_else(|| "无法获取当前显示器".to_string())?;

        if let Some(window) = &main_window {
            crate::extensions::color_picker::prepare_main_window_for_capture(window).await;
        }

        let size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let session = ScreenshotSession {
            monitor: monitor.clone(),
            logical_width: size.width as f64 / scale_factor,
            logical_height: size.height as f64 / scale_factor,
            scale_factor,
        };
        *ACTIVE_SESSION
            .lock()
            .map_err(|_| "截图会话锁定失败".to_string())? = Some(session.clone());

        if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
            let _ = window.close();
        }

        let position = monitor.position();
        let window = WebviewWindowBuilder::new(
            &app,
            OVERLAY_LABEL,
            WebviewUrl::App("/screenshot-selection".into()),
        )
        .title("截图")
        .decorations(false)
        .transparent(true)
        .visible(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .position(
            position.x as f64 / session.scale_factor,
            position.y as f64 / session.scale_factor,
        )
        .inner_size(session.logical_width, session.logical_height)
        .build()
        .map_err(|err| format!("无法打开截图窗口: {err}"))?;

        let _ = window.set_position(tauri::Position::Physical(*position));
        let _ = window.set_size(tauri::Size::Physical(*size));
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("当前平台暂不支持区域截图".to_string())
    }
}

#[tauri::command]
pub fn get_screenshot_overlay_info() -> Result<ScreenshotOverlayInfo, String> {
    let session = ACTIVE_SESSION
        .lock()
        .map_err(|_| "截图会话锁定失败".to_string())?
        .clone()
        .ok_or_else(|| "截图会话已失效，请重新截图".to_string())?;
    Ok(ScreenshotOverlayInfo {
        logical_width: session.logical_width,
        logical_height: session.logical_height,
        scale_factor: session.scale_factor,
    })
}

#[tauri::command]
pub async fn copy_screenshot_region(app: AppHandle, rect: ScreenshotRect) -> Result<(), String> {
    let png = capture_region_png(&app, rect).await?;
    crate::extensions::clipboard::commands::write_png_to_clipboard(&png)
}

#[tauri::command]
pub async fn save_screenshot_region(
    app: AppHandle,
    rect: ScreenshotRect,
    path: String,
) -> Result<(), String> {
    let png = capture_region_png(&app, rect).await?;
    let path = std::path::PathBuf::from(path);
    if path.as_os_str().is_empty() {
        return Err("请选择保存位置".to_string());
    }
    std::fs::write(&path, png).map_err(|err| format!("保存截图失败: {err}"))
}

#[tauri::command]
pub fn finish_screenshot_selection(app: AppHandle, restore_launcher: bool) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.close();
    }
    if let Ok(mut session) = ACTIVE_SESSION.lock() {
        *session = None;
    }
    if restore_launcher {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
            let _ = main.set_focus();
        }
    }
}

async fn capture_region_png(app: &AppHandle, rect: ScreenshotRect) -> Result<Vec<u8>, String> {
    let session = ACTIVE_SESSION
        .lock()
        .map_err(|_| "截图会话锁定失败".to_string())?
        .clone()
        .ok_or_else(|| "截图会话已失效，请重新截图".to_string())?;

    let overlay = app
        .get_webview_window(OVERLAY_LABEL)
        .ok_or_else(|| "截图窗口已关闭".to_string())?;
    crate::extensions::color_picker::prepare_main_window_for_capture(&overlay).await;

    let result = encode_region(&session, rect);
    if result.is_err() {
        let _ = overlay.show();
        let _ = overlay.set_focus();
    }
    result
}

fn encode_region(session: &ScreenshotSession, rect: ScreenshotRect) -> Result<Vec<u8>, String> {
    let size = session.monitor.size();
    let x = logical_to_physical(rect.x, session.scale_factor, size.width)?;
    let y = logical_to_physical(rect.y, session.scale_factor, size.height)?;
    let width = logical_length(
        rect.width,
        session.scale_factor,
        size.width.saturating_sub(x),
    )?;
    let height = logical_length(
        rect.height,
        session.scale_factor,
        size.height.saturating_sub(y),
    )?;

    let capture = crate::extensions::color_picker::capture_monitor(&session.monitor)?;
    let image = RgbaImage::from_raw(capture.width, capture.height, capture.rgba_data)
        .ok_or_else(|| "截图数据异常".to_string())?;
    let cropped = image::imageops::crop_imm(&image, x, y, width, height).to_image();
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(cropped.as_raw(), width, height, ColorType::Rgba8.into())
        .map_err(|err| format!("编码截图失败: {err}"))?;
    Ok(png)
}

fn logical_to_physical(value: f64, scale: f64, maximum: u32) -> Result<u32, String> {
    if !value.is_finite() || value < 0.0 {
        return Err("截图区域无效".to_string());
    }
    Ok((value * scale).round().clamp(0.0, maximum as f64) as u32)
}

fn logical_length(value: f64, scale: f64, maximum: u32) -> Result<u32, String> {
    if !value.is_finite() || value <= 0.0 {
        return Err("请先拖动选择截图区域".to_string());
    }
    if maximum == 0 {
        return Err("截图区域超出屏幕范围".to_string());
    }
    Ok((value * scale).round().clamp(1.0, maximum as f64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_selection() {
        assert!(logical_length(0.0, 1.0, 100).is_err());
    }
}
