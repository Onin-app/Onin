use super::ColorPickerCapture;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, IsWindowVisible, ShowWindow, SW_HIDE,
};

const OVERLAY_LABEL: &str = "color-picker-overlay";
const STABILITY_DIFF_THRESHOLD: u64 = 180_000;
const COLOR_PICKER_DEBUG: bool = false;

macro_rules! println {
    ($($arg:tt)*) => {
        if COLOR_PICKER_DEBUG {
            std::println!($($arg)*);
        }
    };
}

/// 截图缓存，供 Overlay WebView 读取
static CACHED_CAPTURE: LazyLock<Mutex<Option<ColorPickerCapture>>> =
    LazyLock::new(|| Mutex::new(None));

/// 启动取色流程（async 版本，避免阻塞事件循环）
pub async fn start_color_picker(app: tauri::AppHandle) -> Result<(), String> {
    let started = Instant::now();
    println!("[color-picker] start requested");

    let main_window = app.get_webview_window("main");
    let main_hwnd = main_window
        .as_ref()
        .and_then(|main| main.hwnd().ok())
        .map(|hwnd| hwnd.0 as isize);
    let probe_rect = main_hwnd.and_then(get_probe_rect_for_hwnd);

    // 先隐藏主窗口，再截图（避免主窗口出现在截图中）
    if let Some(main) = &main_window {
        println!("[color-picker] hiding main window");
        hide_main_window_for_capture(&main);
    } else {
        println!("[color-picker] main window not found");
    }

    wait_for_dwm_after_hide(main_hwnd, probe_rect).await;
    println!(
        "[color-picker] after DWM wait: {}ms",
        started.elapsed().as_millis()
    );

    // 截图（在当前线程，避免 Windows 后台线程 GDI 截图黑屏）
    let capture = capture_primary_screen(&app).map_err(|e| {
        println!("[color-picker] capture failed: {e}");
        // 截图失败，恢复主窗口
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        e
    })?;

    let logical_width = capture.logical_width;
    let logical_height = capture.logical_height;
    println!(
        "[color-picker] capture ready: {}x{} physical, {:.1}x{:.1} logical, scale {:.2}, bytes {}, elapsed {}ms",
        capture.width,
        capture.height,
        logical_width,
        logical_height,
        capture.scale_factor,
        capture.rgba_data.len(),
        started.elapsed().as_millis()
    );

    // 写入缓存
    {
        let mut cache = CACHED_CAPTURE
            .lock()
            .map_err(|_| "缓存锁定失败".to_string())?;
        *cache = Some(capture);
    }
    println!("[color-picker] capture cached");

    if let Some(overlay) = app.get_webview_window(OVERLAY_LABEL) {
        println!("[color-picker] reusing overlay window");
        let _ = overlay.set_position(LogicalPosition::new(0.0, 0.0));
        let _ = overlay.set_size(LogicalSize::new(logical_width, logical_height));
        let _ = overlay.set_always_on_top(true);
        overlay
            .emit("color_picker_capture_ready", ())
            .map_err(|err| {
                println!("[color-picker] emit capture_ready failed: {err}");
                clear_capture_cache();
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.show();
                }
                err.to_string()
            })?;
        let overlay_clone = overlay.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            let _ = overlay_clone.set_focus();
        });
        println!(
            "[color-picker] capture_ready emitted, total {}ms",
            started.elapsed().as_millis()
        );
        return Ok(());
    }

    println!("[color-picker] building overlay window on demand");
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
        println!("[color-picker] overlay build failed: {err}");
        clear_capture_cache();
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        err.to_string()
    })?;

    attach_overlay_cleanup(app.clone(), &overlay);
    println!(
        "[color-picker] overlay built on demand, total {}ms",
        started.elapsed().as_millis()
    );

    Ok(())
}

fn hide_main_window_for_capture(main: &tauri::WebviewWindow) {
    let _ = main.hide();

    match main.hwnd() {
        Ok(hwnd) => unsafe {
            let native_hwnd = HWND(hwnd.0 as isize);
            let _ = ShowWindow(native_hwnd, SW_HIDE);
            println!(
                "[color-picker] native ShowWindow(SW_HIDE), visible={}",
                IsWindowVisible(native_hwnd).as_bool()
            );
        },
        Err(err) => println!("[color-picker] get main hwnd failed: {err}"),
    }
}

#[derive(Clone, Copy)]
struct CaptureRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn get_probe_rect_for_hwnd(hwnd: isize) -> Option<CaptureRect> {
    let hwnd = HWND(hwnd);
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        println!("[color-picker] GetWindowRect failed");
        return None;
    }

    let window_width = rect.right - rect.left;
    let window_height = rect.bottom - rect.top;
    if window_width <= 0 || window_height <= 0 {
        return None;
    }

    let width = window_width.min(320) as u32;
    let height = window_height.min(180) as u32;
    let x = rect.left + (window_width - width as i32) / 2;
    let y = rect.top + (window_height - height as i32) / 2;
    println!(
        "[color-picker] stability probe rect: {},{} {}x{}",
        x, y, width, height
    );

    Some(CaptureRect {
        x,
        y,
        width,
        height,
    })
}

fn attach_overlay_cleanup(app: tauri::AppHandle, overlay: &tauri::WebviewWindow) {
    // 监听窗口事件（例如 Alt+F4、异常崩溃等导致窗口销毁）。
    // 如果缓存仍然存在，说明是没有走正常的 finish_color_picker 流程（异常关闭）。
    overlay.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let is_abnormal = {
                CACHED_CAPTURE
                    .lock()
                    .map(|cache| cache.is_some())
                    .unwrap_or(false)
            };
            if is_abnormal {
                clear_capture_cache();
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.show();
                    let _ = main.set_focus();
                }
                let _ = app.emit("color_picker_result", Option::<String>::None);
            }
        }
    });
}

async fn wait_for_dwm_after_hide(main_hwnd: Option<isize>, probe_rect: Option<CaptureRect>) {
    let started = Instant::now();
    tokio::time::sleep(std::time::Duration::from_millis(16)).await;

    let mut previous_probe: Option<Vec<u8>> = None;
    let mut stable_count = 0;

    for attempt in 1..=10 {
        let _ = flush_dwm(&format!("hide-{attempt}"));
        let visible = main_hwnd
            .map(|hwnd| unsafe { IsWindowVisible(HWND(hwnd)).as_bool() })
            .unwrap_or(false);
        let diff = match probe_rect {
            Some(rect) => match capture_region_bgra(rect) {
                Ok(next_probe) => {
                    let diff = previous_probe
                        .as_ref()
                        .map(|prev| image_diff_score(prev, &next_probe))
                        .unwrap_or(u64::MAX);
                    previous_probe = Some(next_probe);
                    if diff <= STABILITY_DIFF_THRESHOLD {
                        stable_count += 1;
                    } else {
                        stable_count = 0;
                    }
                    Some(diff)
                }
                Err(err) => {
                    println!("[color-picker] stability probe failed: {err}");
                    None
                }
            },
            None => None,
        };
        println!(
            "[color-picker] hide wait attempt {attempt}, visible={visible}, diff={:?}, stable={}, elapsed={}ms",
            diff,
            stable_count,
            started.elapsed().as_millis()
        );

        if !visible && stable_count >= 2 {
            break;
        }

        if started.elapsed() >= std::time::Duration::from_millis(180) {
            println!("[color-picker] stability wait reached max");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
}

fn flush_dwm(label: &str) -> bool {
    let flush_started = Instant::now();
    match unsafe { DwmFlush() } {
        Ok(_) => {
            println!(
                "[color-picker] DwmFlush {label} ok: {}ms",
                flush_started.elapsed().as_millis()
            );
            true
        }
        Err(err) => {
            println!("[color-picker] DwmFlush {label} failed: {err}");
            false
        }
    }
}

fn capture_region_bgra(rect: CaptureRect) -> Result<Vec<u8>, String> {
    let desktop_dc = DesktopDc::new()?;

    let memory_dc = unsafe { CreateCompatibleDC(desktop_dc.hdc) };
    if memory_dc.0 == 0 {
        return Err("无法创建探测 DC".to_string());
    }
    let _memory_dc_guard = MemoryDcGuard(memory_dc);

    let bitmap =
        unsafe { CreateCompatibleBitmap(desktop_dc.hdc, rect.width as i32, rect.height as i32) };
    if bitmap.0 == 0 {
        return Err("无法创建探测位图".to_string());
    }
    let _bitmap_guard = BitmapGuard(bitmap);

    let old_obj = unsafe { SelectObject(memory_dc, bitmap) };
    if old_obj.0 == 0 {
        return Err("无法选择探测位图".to_string());
    }
    let _sel_guard = SelectionGuard {
        hdc: memory_dc,
        old: old_obj,
    };

    unsafe {
        BitBlt(
            memory_dc,
            0,
            0,
            rect.width as i32,
            rect.height as i32,
            desktop_dc.hdc,
            rect.x,
            rect.y,
            SRCCOPY,
        )
    }
    .map_err(|e| format!("探测截图失败: {}", e))?;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: rect.width as i32,
            biHeight: -(rect.height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; (rect.width * rect.height * 4) as usize];
    let lines = unsafe {
        GetDIBits(
            memory_dc,
            bitmap,
            0,
            rect.height,
            Some(bgra.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };
    if lines == 0 {
        return Err("读取探测像素失败".to_string());
    }

    Ok(bgra)
}

fn image_diff_score(previous: &[u8], next: &[u8]) -> u64 {
    previous
        .chunks_exact(16)
        .zip(next.chunks_exact(16))
        .map(|(a, b)| {
            a.iter()
                .zip(b.iter())
                .take(12)
                .map(|(x, y)| x.abs_diff(*y) as u64)
                .sum::<u64>()
        })
        .sum()
}

/// Overlay WebView 启动后调用，获取截图数据
pub fn get_color_picker_capture() -> Result<ColorPickerCapture, String> {
    let result = CACHED_CAPTURE
        .lock()
        .map_err(|_| "缓存锁定失败".to_string())?
        .clone()
        .ok_or_else(|| "截图缓存不存在".to_string());
    match &result {
        Ok(capture) => println!(
            "[color-picker] get capture ok: {}x{}, bytes {}",
            capture.width,
            capture.height,
            capture.rgba_data.len()
        ),
        Err(err) => println!("[color-picker] get capture failed: {err}"),
    }
    result
}

/// 清理截图缓存（由 api.rs finish 命令调用）
pub fn clear_capture_cache() {
    if let Ok(mut cache) = CACHED_CAPTURE.lock() {
        *cache = None;
    }
}

/// 截取主屏幕并保留为 RGBA 像素数据
fn capture_primary_screen(app: &tauri::AppHandle) -> Result<ColorPickerCapture, String> {
    let started = Instant::now();
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "无法获取主屏幕".to_string())?;

    let size = monitor.size();
    let scale_factor = monitor.scale_factor();
    let width = size.width;
    let height = size.height;
    println!(
        "[color-picker] capture monitor: {}x{}, scale {:.2}, pos {},{}",
        width,
        height,
        scale_factor,
        monitor.position().x,
        monitor.position().y
    );

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
    println!(
        "[color-picker] BitBlt done: {}ms",
        started.elapsed().as_millis()
    );

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
    println!(
        "[color-picker] GetDIBits done: {}ms",
        started.elapsed().as_millis()
    );

    convert_bgra_to_rgba(&mut bgra);
    println!(
        "[color-picker] BGRA to RGBA done: {}ms",
        started.elapsed().as_millis()
    );

    Ok(ColorPickerCapture {
        width,
        height,
        logical_width: width as f64 / scale_factor,
        logical_height: height as f64 / scale_factor,
        scale_factor,
        rgba_data: bgra,
    })
}

fn convert_bgra_to_rgba(bytes: &mut [u8]) {
    let (prefix, words, suffix) = unsafe { bytes.align_to_mut::<u32>() };

    if !prefix.is_empty() || !suffix.is_empty() {
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        return;
    }

    for word in words {
        let bgra = *word;
        *word = 0xFF00_0000
            | ((bgra & 0x00FF_0000) >> 16)
            | (bgra & 0x0000_FF00)
            | ((bgra & 0x0000_00FF) << 16);
    }
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
