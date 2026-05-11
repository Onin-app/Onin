use serde::Serialize;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
mod windows;

/// 截图缓存结构，供 Overlay WebView 使用
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPickerCapture {
    /// 物理像素宽度
    pub width: u32,
    /// 物理像素高度
    pub height: u32,
    /// 逻辑宽度（用于窗口尺寸）
    pub logical_width: f64,
    /// 逻辑高度（用于窗口尺寸）
    pub logical_height: f64,
    /// 缩放系数（物理 / 逻辑）
    pub scale_factor: f64,
    /// 原始 BMP 二进制数据（不传输给前端 JSON，仅供单独的二进制接口获取）
    #[serde(skip)]
    pub bmp_data: Vec<u8>,
}

// ── Windows ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub async fn start_color_picker(app: AppHandle) -> Result<(), String> {
    windows::start_color_picker(app).await
}

#[cfg(target_os = "windows")]
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    windows::get_color_picker_capture()
}

#[cfg(target_os = "windows")]
pub fn clear_capture_cache() {
    windows::clear_capture_cache();
}

// ── macOS（未实现）──────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub async fn start_color_picker(_app: AppHandle) -> Result<(), String> {
    Err("macOS 取色暂未实现".to_string())
}

#[cfg(target_os = "macos")]
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    Err("macOS 取色暂未实现".to_string())
}

#[cfg(target_os = "macos")]
pub fn clear_capture_cache() {}

// ── Linux（未实现）──────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub async fn start_color_picker(_app: AppHandle) -> Result<(), String> {
    Err("Linux 取色暂未实现".to_string())
}

#[cfg(target_os = "linux")]
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    Err("Linux 取色暂未实现".to_string())
}

#[cfg(target_os = "linux")]
pub fn clear_capture_cache() {}

// ── 其他平台 ─────────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub async fn start_color_picker(_app: AppHandle) -> Result<(), String> {
    Err("当前平台不支持取色".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    Err("当前平台不支持取色".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn clear_capture_cache() {}
