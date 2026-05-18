use super::ColorPickerCapture;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, EventTarget, Manager, Monitor, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT, VK_SHIFT,
    VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetCursorPos, GetWindowRect, IsWindowVisible, SetForegroundWindow,
    ShowWindow, SW_HIDE,
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
static CACHED_CAPTURES: LazyLock<Mutex<HashMap<String, ColorPickerCapture>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ACTIVE_OVERLAY_LABELS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(vec![]));
static ACTIVE_OVERLAY_SPECS: LazyLock<Mutex<Vec<OverlaySpec>>> =
    LazyLock::new(|| Mutex::new(vec![]));
static RESTORE_MAIN_ON_FINISH: AtomicBool = AtomicBool::new(true);
static KEYBOARD_POLL_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ColorPickerKeyboardEvent {
    target_label: String,
    key: &'static str,
    shift_key: bool,
}

#[derive(Clone)]
struct OverlaySpec {
    label: String,
    position: tauri::PhysicalPosition<i32>,
    size: tauri::PhysicalSize<u32>,
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
}

impl OverlaySpec {
    fn contains_physical_point(&self, point: POINT) -> bool {
        let left = self.position.x;
        let top = self.position.y;
        let right = left + self.size.width as i32;
        let bottom = top + self.size.height as i32;
        point.x >= left && point.x < right && point.y >= top && point.y < bottom
    }
}

/// 启动取色流程（async 版本，避免阻塞事件循环）
pub async fn start_color_picker(
    app: tauri::AppHandle,
    restore_main_window: bool,
) -> Result<(), String> {
    let started = Instant::now();
    println!("[color-picker] start requested");
    RESTORE_MAIN_ON_FINISH.store(restore_main_window, Ordering::Relaxed);

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
    let captures = capture_all_screens(&app).map_err(|e| {
        println!("[color-picker] capture failed: {e}");
        // 截图失败，恢复主窗口
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        e
    })?;

    if captures.is_empty() {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        return Err("无法获取屏幕".to_string());
    }

    let mut overlay_specs: Vec<_> = captures
        .iter()
        .enumerate()
        .map(|(index, (monitor, capture))| OverlaySpec {
            label: overlay_label(index),
            position: *monitor.position(),
            size: *monitor.size(),
            logical_width: capture.logical_width,
            logical_height: capture.logical_height,
            scale_factor: capture.scale_factor,
        })
        .collect();
    let total_capture_bytes = captures
        .iter()
        .map(|(_, capture)| capture.rgba_data.len())
        .sum::<usize>();

    println!(
        "[color-picker] captures ready: {} monitor(s), {} bytes, elapsed {}ms",
        overlay_specs.len(),
        total_capture_bytes,
        started.elapsed().as_millis()
    );

    // 写入缓存
    {
        let mut cache = CACHED_CAPTURES
            .lock()
            .map_err(|_| "缓存锁定失败".to_string())?;
        cache.clear();
        for (index, (_, capture)) in captures.into_iter().enumerate() {
            cache.insert(overlay_label(index), capture);
        }
    }
    {
        let mut labels = ACTIVE_OVERLAY_LABELS
            .lock()
            .map_err(|_| "窗口状态锁定失败".to_string())?;
        *labels = overlay_specs
            .iter()
            .map(|spec| spec.label.clone())
            .collect();
    }
    {
        let mut specs = ACTIVE_OVERLAY_SPECS
            .lock()
            .map_err(|_| "窗口状态锁定失败".to_string())?;
        *specs = overlay_specs.clone();
    }
    println!("[color-picker] capture cached");

    for spec in &mut overlay_specs {
        if let Some(overlay) = app.get_webview_window(&spec.label) {
            println!("[color-picker] reusing overlay window: {}", spec.label);
            let _ = overlay.set_position(tauri::Position::Physical(spec.position));
            let _ = overlay.set_size(tauri::Size::Physical(spec.size));
            let _ = overlay.set_always_on_top(true);
            overlay
                .emit("color_picker_capture_ready", ())
                .map_err(|err| {
                    println!("[color-picker] emit capture_ready failed: {err}");
                    clear_capture_cache();
                    show_main_window(&app);
                    err.to_string()
                })?;
            continue;
        }

        println!(
            "[color-picker] building overlay window on demand: {}",
            spec.label
        );
        let overlay = WebviewWindowBuilder::new(
            &app,
            &spec.label,
            WebviewUrl::App("/color-picker-overlay".into()),
        )
        .title("Color Picker")
        .inner_size(spec.logical_width, spec.logical_height)
        .position(
            spec.position.x as f64 / spec.scale_factor,
            spec.position.y as f64 / spec.scale_factor,
        )
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
            show_main_window(&app);
            err.to_string()
        })?;
        let _ = overlay.set_position(tauri::Position::Physical(spec.position));
        let _ = overlay.set_size(tauri::Size::Physical(spec.size));

        attach_overlay_cleanup(app.clone(), &overlay);
    }

    let focused_spec = cursor_position()
        .and_then(|point| {
            overlay_specs
                .iter()
                .find(|spec| spec.contains_physical_point(point))
        })
        .or_else(|| overlay_specs.first());

    if let Some(spec) = focused_spec {
        start_keyboard_polling(app.clone());

        if let Some(overlay) = app.get_webview_window(&spec.label) {
            let overlay_clone = overlay.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                focus_overlay_window(&overlay_clone);
            });
        }
    }

    println!(
        "[color-picker] overlays ready, total {}ms",
        started.elapsed().as_millis()
    );

    Ok(())
}

pub fn should_restore_main_on_finish() -> bool {
    RESTORE_MAIN_ON_FINISH.load(Ordering::Relaxed)
}

pub fn focus_color_picker_overlay(app: &tauri::AppHandle, label: Option<String>) {
    let target_label = label.unwrap_or_else(|| OVERLAY_LABEL.to_string());
    if let Some(overlay) = app.get_webview_window(&target_label) {
        focus_overlay_window(&overlay);
    }
}

fn focus_overlay_window(overlay: &tauri::WebviewWindow) {
    let _ = overlay.set_focus();

    if let Ok(hwnd) = overlay.hwnd() {
        unsafe {
            let native_hwnd = HWND(hwnd.0 as isize);
            let _ = BringWindowToTop(native_hwnd);
            let _ = SetForegroundWindow(native_hwnd);
        }
    }
}

fn start_keyboard_polling(app: tauri::AppHandle) {
    if KEYBOARD_POLL_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let mut last_escape = false;
        let mut last_enter = false;
        let mut last_arrow_emit = Instant::now()
            .checked_sub(Duration::from_millis(120))
            .unwrap_or_else(Instant::now);

        while KEYBOARD_POLL_ACTIVE.load(Ordering::SeqCst) {
            let shift_key = key_down(VK_SHIFT);
            let escape = key_down(VK_ESCAPE);
            let enter = key_down(VK_RETURN);

            if escape && !last_escape {
                emit_keyboard_event(&app, "Escape", shift_key);
            }
            if enter && !last_enter {
                emit_keyboard_event(&app, "Enter", shift_key);
            }

            let now = Instant::now();
            if now.duration_since(last_arrow_emit) >= Duration::from_millis(24) {
                if key_down(VK_UP) {
                    emit_keyboard_event(&app, "ArrowUp", shift_key);
                    last_arrow_emit = now;
                }
                if key_down(VK_DOWN) {
                    emit_keyboard_event(&app, "ArrowDown", shift_key);
                    last_arrow_emit = now;
                }
                if key_down(VK_LEFT) {
                    emit_keyboard_event(&app, "ArrowLeft", shift_key);
                    last_arrow_emit = now;
                }
                if key_down(VK_RIGHT) {
                    emit_keyboard_event(&app, "ArrowRight", shift_key);
                    last_arrow_emit = now;
                }
            }

            last_escape = escape;
            last_enter = enter;
            tokio::time::sleep(Duration::from_millis(12)).await;
        }
    });
}

fn stop_keyboard_polling() {
    KEYBOARD_POLL_ACTIVE.store(false, Ordering::SeqCst);
}

fn emit_keyboard_event(app: &tauri::AppHandle, key: &'static str, shift_key: bool) {
    let target_label = current_keyboard_target_label();
    let _ = app.emit_to(
        EventTarget::webview_window(target_label.clone()),
        "color_picker_keyboard",
        ColorPickerKeyboardEvent {
            target_label,
            key,
            shift_key,
        },
    );
}

fn key_down(key: VIRTUAL_KEY) -> bool {
    unsafe { GetAsyncKeyState(key.0 as i32) < 0 }
}

fn current_keyboard_target_label() -> String {
    let Some(point) = cursor_position() else {
        return OVERLAY_LABEL.to_string();
    };

    ACTIVE_OVERLAY_SPECS
        .lock()
        .ok()
        .and_then(|specs| {
            specs
                .iter()
                .find(|spec| spec.contains_physical_point(point))
                .or_else(|| specs.first())
                .map(|spec| spec.label.clone())
        })
        .unwrap_or_else(|| OVERLAY_LABEL.to_string())
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
                CACHED_CAPTURES
                    .lock()
                    .map(|cache| !cache.is_empty())
                    .unwrap_or(false)
            };
            if is_abnormal {
                clear_capture_cache();
                show_main_window(&app);
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
pub fn get_color_picker_capture(label: Option<String>) -> Result<ColorPickerCapture, String> {
    let requested_label = label.unwrap_or_else(|| OVERLAY_LABEL.to_string());
    let result = CACHED_CAPTURES
        .lock()
        .map_err(|_| "缓存锁定失败".to_string())?
        .get(&requested_label)
        .map(|capture| ColorPickerCapture {
            width: capture.width,
            height: capture.height,
            logical_width: capture.logical_width,
            logical_height: capture.logical_height,
            scale_factor: capture.scale_factor,
            rgba_data: Vec::new(),
        })
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

/// Overlay WebView 启动后调用，获取 RGBA 像素数据
pub fn get_color_picker_image(label: Option<String>) -> Result<Vec<u8>, String> {
    let requested_label = label.unwrap_or_else(|| OVERLAY_LABEL.to_string());
    let result = CACHED_CAPTURES
        .lock()
        .map_err(|_| "缓存锁定失败".to_string())?
        .get(&requested_label)
        .map(|capture| capture.rgba_data.clone())
        .ok_or_else(|| "截图缓存不存在".to_string());
    match &result {
        Ok(bytes) => println!(
            "[color-picker] get image ok: label {}, bytes {}",
            requested_label,
            bytes.len()
        ),
        Err(err) => println!("[color-picker] get image failed: {err}"),
    }
    result
}

/// 清理截图缓存（由 api.rs finish 命令调用）
pub fn clear_capture_cache() {
    stop_keyboard_polling();
    if let Ok(mut cache) = CACHED_CAPTURES.lock() {
        cache.clear();
    }
    if let Ok(mut specs) = ACTIVE_OVERLAY_SPECS.lock() {
        specs.clear();
    }
}

pub fn active_overlay_labels() -> Vec<String> {
    ACTIVE_OVERLAY_LABELS
        .lock()
        .map(|labels| labels.clone())
        .unwrap_or_else(|_| vec![OVERLAY_LABEL.to_string()])
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}

fn overlay_label(index: usize) -> String {
    if index == 0 {
        OVERLAY_LABEL.to_string()
    } else {
        format!("{OVERLAY_LABEL}-{index}")
    }
}

fn cursor_position() -> Option<POINT> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some(point)
}

/// 截取所有屏幕并保留为 RGBA 像素数据
fn capture_all_screens(
    app: &tauri::AppHandle,
) -> Result<Vec<(Monitor, ColorPickerCapture)>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    monitors
        .into_iter()
        .map(|monitor| {
            let capture = capture_monitor(&monitor)?;
            Ok((monitor, capture))
        })
        .collect()
}

/// 截取指定屏幕并保留为 RGBA 像素数据
fn capture_monitor(monitor: &Monitor) -> Result<ColorPickerCapture, String> {
    let started = Instant::now();
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
