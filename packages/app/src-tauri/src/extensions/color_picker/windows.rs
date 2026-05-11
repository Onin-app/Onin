use super::ColorPickerCapture;
use image::{ImageBuffer, ImageFormat, Rgba};
use std::sync::{LazyLock, Mutex};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

const OVERLAY_LABEL: &str = "color-picker-overlay";

/// 截图缓存，供 Overlay WebView 读取
static CACHED_CAPTURE: LazyLock<Mutex<Option<ColorPickerCapture>>> =
    LazyLock::new(|| Mutex::new(None));

/// 启动取色流程（async 版本，避免阻塞事件循环）
pub async fn start_color_picker(app: tauri::AppHandle) -> Result<(), String> {
    // 关闭残留的 overlay
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = win.close();
    }

    // 先隐藏主窗口，再截图（避免主窗口出现在截图中）
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.hide();
    }

    // 延迟 150 毫秒，给系统 DWM 留出重绘时间，确保截图里不包含已隐藏的主窗口
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // 截图（在当前线程，避免 Windows 后台线程 GDI 截图黑屏）
    let capture = capture_primary_screen(&app).map_err(|e| {
        // 截图失败，恢复主窗口
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        e
    })?;

    let logical_width = capture.logical_width;
    let logical_height = capture.logical_height;

    // 写入缓存
    {
        let mut cache = CACHED_CAPTURE
            .lock()
            .map_err(|_| "缓存锁定失败".to_string())?;
        *cache = Some(capture);
    }

    let overlay = WebviewWindowBuilder::new(
        &app,
        OVERLAY_LABEL,
        WebviewUrl::App("/color-picker-overlay".into()),
    )
    .title("Color Picker")
    .inner_size(logical_width, logical_height)
    .position(0.0, 0.0)
    .decorations(false)
    .transparent(false)
    .visible(false) // 初始隐藏，等前端加载完图片再显示，彻底解决白屏黑屏闪烁
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true)
    .focused(true)
    .shadow(false)
    .build()
    .map_err(|err| {
        clear_capture_cache();
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        err.to_string()
    })?;

    // 监听窗口事件（例如 Alt+F4、异常崩溃等导致窗口销毁）
    // 如果缓存仍然存在，说明是没有走正常的 finish_color_picker 流程（异常关闭）
    let app_clone = app.clone();
    overlay.on_window_event(move |event| {
        use tauri::Emitter;
        if let tauri::WindowEvent::Destroyed = event {
            let is_abnormal = {
                CACHED_CAPTURE
                    .lock()
                    .map(|cache| cache.is_some())
                    .unwrap_or(false)
            };
            if is_abnormal {
                clear_capture_cache();
                if let Some(main) = app_clone.get_webview_window("main") {
                    let _ = main.show();
                    let _ = main.set_focus();
                }
                let _ = app_clone.emit("color_picker_result", Option::<String>::None);
            }
        }
    });

    Ok(())
}

/// Overlay WebView 启动后调用，获取截图数据
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    CACHED_CAPTURE
        .lock()
        .map_err(|_| "缓存锁定失败".to_string())?
        .clone()
        .ok_or_else(|| "截图缓存不存在".to_string())
}

/// 清理截图缓存（由 api.rs finish 命令调用）
pub fn clear_capture_cache() {
    if let Ok(mut cache) = CACHED_CAPTURE.lock() {
        *cache = None;
    }
}

/// 截取主屏幕并转为 base64 PNG data URL
fn capture_primary_screen(app: &tauri::AppHandle) -> Result<ColorPickerCapture, String> {
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "无法获取主屏幕".to_string())?;

    let size = monitor.size();
    let scale_factor = monitor.scale_factor();
    let width = size.width;
    let height = size.height;

    let desktop_dc = DesktopDc::new()?;

    let memory_dc = unsafe { CreateCompatibleDC(desktop_dc.hdc) };
    if memory_dc.0 == 0 {
        return Err("无法创建截图 DC".to_string());
    }
    let _memory_dc_guard = MemoryDcGuard(memory_dc);

    let bitmap = unsafe { CreateCompatibleBitmap(desktop_dc.hdc, width as i32, height as i32) };
    if bitmap.0 == 0 {
        return Err("无法创建截图位图".to_string());
    }
    let _bitmap_guard = BitmapGuard(bitmap);

    let old_obj = unsafe { SelectObject(memory_dc, bitmap) };
    if old_obj.0 == 0 {
        return Err("无法选择截图位图".to_string());
    }
    let _sel_guard = SelectionGuard {
        hdc: memory_dc,
        old: old_obj,
    };

    let pos = monitor.position();
    unsafe {
        BitBlt(
            memory_dc,
            0,
            0,
            width as i32,
            height as i32,
            desktop_dc.hdc,
            pos.x,
            pos.y,
            SRCCOPY,
        )
    }
    .map_err(|e| format!("屏幕截图失败: {}", e))?;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; (width * height * 4) as usize];
    let lines = unsafe {
        GetDIBits(
            memory_dc,
            bitmap,
            0,
            height,
            Some(bgra.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };
    if lines == 0 {
        return Err("读取截图像素失败".to_string());
    }

    // BGRA → RGBA
    for pixel in bgra.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        pixel[3] = 255;
    }

    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, bgra)
        .ok_or_else(|| "构建截图图像失败".to_string())?;

    let mut bmp_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut bmp_data), ImageFormat::Bmp)
        .map_err(|e| format!("保存截图文件失败: {}", e))?;

    Ok(ColorPickerCapture {
        width,
        height,
        logical_width: width as f64 / scale_factor,
        logical_height: height as f64 / scale_factor,
        scale_factor,
        bmp_data,
    })
}

// ── RAII 资源守卫 ────────────────────────────────────────────────────────────

struct DesktopDc {
    desktop: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
}

impl DesktopDc {
    fn new() -> Result<Self, String> {
        let desktop = HWND(0);
        let hdc = unsafe { GetDC(desktop) };
        if hdc.0 == 0 {
            return Err("无法读取屏幕 DC".to_string());
        }
        Ok(Self { desktop, hdc })
    }
}

impl Drop for DesktopDc {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(self.desktop, self.hdc);
        }
    }
}

struct MemoryDcGuard(windows::Win32::Graphics::Gdi::HDC);

impl Drop for MemoryDcGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.0);
        }
    }
}

struct BitmapGuard(windows::Win32::Graphics::Gdi::HBITMAP);

impl Drop for BitmapGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.0);
        }
    }
}

struct SelectionGuard {
    hdc: windows::Win32::Graphics::Gdi::HDC,
    old: windows::Win32::Graphics::Gdi::HGDIOBJ,
}

impl Drop for SelectionGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = SelectObject(self.hdc, self.old);
        }
    }
}
