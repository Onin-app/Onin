//! Screenshot extension.
//!
//! The command deliberately contains no shortcut registration.  The shortcut
//! manager invokes `extension:screenshot:capture` when a user chooses to bind
//! one, while the launcher invokes the same Tauri command directly.

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult,
};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use tauri::{AppHandle, Manager};

pub static SCREENSHOT_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "screenshot",
    name: "截图",
    description: "截取当前屏幕并复制到系统剪贴板",
    icon: "camera",
    commands: &[ExtensionCommand {
        code: "capture",
        name: "截图",
        description: Some("截取当前屏幕并复制到系统剪贴板"),
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

    /// The Extension trait has no AppHandle, so it cannot perform a capture.
    /// Normal launcher and shortcut dispatch intentionally call
    /// `take_screenshot` instead. Generic backend dispatch reaches this guard
    /// only when a caller bypasses that native command.
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

/// Capture the display containing the launcher window (or the primary display)
/// and put the result on the system clipboard.
///
/// This is intentionally Windows-only for now. The extension contract remains
/// platform-neutral so macOS/Linux backends can be added without changing the
/// command name or shortcut bindings.
#[tauri::command]
pub async fn take_screenshot(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let main_window = app.get_webview_window("main");
        let target_monitor = main_window
            .as_ref()
            .and_then(|window| window.current_monitor().ok().flatten())
            .or_else(|| app.primary_monitor().ok().flatten())
            .ok_or_else(|| "无法获取当前显示器".to_string())?;

        // Reuse the color picker's DWM-aware hide flow instead of relying on a
        // fixed delay, which can capture the launcher on a busy Windows system.
        if let Some(window) = &main_window {
            crate::extensions::color_picker::prepare_main_window_for_capture(window).await;
        }

        let result = (|| {
            let capture = crate::extensions::color_picker::capture_monitor(&target_monitor)?;
            let mut png = Vec::new();
            PngEncoder::new(&mut png)
                .write_image(
                    &capture.rgba_data,
                    capture.width,
                    capture.height,
                    ColorType::Rgba8.into(),
                )
                .map_err(|err| format!("编码截图失败: {err}"))?;
            crate::extensions::clipboard::commands::write_png_to_clipboard(&png)
        })();

        // A successful screenshot should return the user to the app they were
        // working in. Only restore the launcher when an error needs attention.
        if result.is_err() {
            if let Some(window) = &main_window {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        result
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("当前平台暂不支持截图".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_a_single_capture_command() {
        assert_eq!(SCREENSHOT_MANIFEST.id, "screenshot");
        assert_eq!(SCREENSHOT_MANIFEST.commands[0].code, "capture");
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(!SCREENSHOT_EXTENSION.execute_command("unknown", "").success);
    }
}
