use super::ColorPickerCapture;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindowBuilder,
};

use objc2::msg_send;

const OVERLAY_LABEL: &str = "color-picker-overlay";
const COLOR_PICKER_DEBUG: bool = false;

macro_rules! dprintln {
    ($($arg:tt)*) => {
        if COLOR_PICKER_DEBUG {
            eprintln!($($arg)*);
        }
    };
}

// ── CoreGraphics FFI ─────────────────────────────────────────────────────────

type CGDirectDisplayID = u32;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

const KCGERROR_SUCCESS: i32 = 0;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> i32;

    fn CGDisplayBounds(display: CGDirectDisplayID) -> CGRect;

    fn CGDisplayCreateImage(display: CGDirectDisplayID) -> *mut std::ffi::c_void;
    fn CGImageGetWidth(image: *mut std::ffi::c_void) -> usize;
    fn CGImageGetHeight(image: *mut std::ffi::c_void) -> usize;
    fn CGImageGetBytesPerRow(image: *mut std::ffi::c_void) -> usize;
    fn CGImageGetDataProvider(image: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGDataProviderCopyData(provider: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CFDataGetBytePtr(data: *mut std::ffi::c_void) -> *const u8;
    fn CFDataGetLength(data: *mut std::ffi::c_void) -> isize;
    fn CFRelease(cf: *mut std::ffi::c_void);
}

// ── Global State ─────────────────────────────────────────────────────────────

static CACHED_CAPTURES: LazyLock<Mutex<HashMap<String, ColorPickerCapture>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ACTIVE_OVERLAY_LABELS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(vec![]));
static ACTIVE_OVERLAY_SPECS: LazyLock<Mutex<Vec<OverlaySpec>>> =
    LazyLock::new(|| Mutex::new(vec![]));
static RESTORE_MAIN_ON_FINISH: AtomicBool = AtomicBool::new(true);

#[derive(Clone)]
struct OverlaySpec {
    label: String,
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    logical_width: f64,
    logical_height: f64,
    scale_factor: f64,
}

impl OverlaySpec {}

// ── Display Enumeration ──────────────────────────────────────────────────────

fn get_active_display_ids() -> Result<Vec<CGDirectDisplayID>, String> {
    let max_displays: u32 = 32;
    let mut ids = vec![0u32; max_displays as usize];
    let mut count: u32 = 0;

    let err = unsafe { CGGetActiveDisplayList(max_displays, ids.as_mut_ptr(), &mut count) };
    if err != KCGERROR_SUCCESS {
        return Err(format!("CGGetActiveDisplayList 失败: {}", err));
    }

    ids.truncate(count as usize);
    Ok(ids)
}

fn get_display_bounds_with_virtual_height(
    display_ids: &[CGDirectDisplayID],
) -> Result<(Vec<(CGDirectDisplayID, CGRect)>, f64), String> {
    let mut list: Vec<(CGDirectDisplayID, CGRect)> = display_ids
        .iter()
        .map(|&did| (did, unsafe { CGDisplayBounds(did) }))
        .collect();

    if list.is_empty() {
        return Err("未检测到显示器".to_string());
    }

    let min_y = list.iter().map(|(_, b)| b.origin.y).fold(0.0_f64, f64::min);
    let max_top = list
        .iter()
        .map(|(_, b)| b.origin.y + b.size.height)
        .fold(0.0_f64, f64::max);

    let virtual_height = max_top - min_y;

    // Sort by top-left Y (converted from bottom-left), then X
    list.sort_by(|(_, a), (_, b)| {
        let ay = virtual_height - a.origin.y - a.size.height;
        let by = virtual_height - b.origin.y - b.size.height;
        ay.partial_cmp(&by)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.origin
                    .x
                    .partial_cmp(&b.origin.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    Ok((list, virtual_height))
}

fn overlay_label(index: usize) -> String {
    if index == 0 {
        OVERLAY_LABEL.to_string()
    } else {
        format!("{OVERLAY_LABEL}-{index}")
    }
}

// ── Screen Capture ───────────────────────────────────────────────────────────

fn capture_display(
    display_id: CGDirectDisplayID,
    scale_factor: f64,
) -> Result<ColorPickerCapture, String> {
    let image_ptr = unsafe { CGDisplayCreateImage(display_id) };
    if image_ptr.is_null() {
        return Err("截图失败: CGDisplayCreateImage 返回 NULL（可能需要屏幕录制权限）".to_string());
    }

    let width = unsafe { CGImageGetWidth(image_ptr) } as u32;
    let height = unsafe { CGImageGetHeight(image_ptr) } as u32;

    let provider = unsafe { CGImageGetDataProvider(image_ptr) };
    if provider.is_null() {
        unsafe { CFRelease(image_ptr) };
        return Err("截图失败: CGImageGetDataProvider 返回 NULL".to_string());
    }

    let cf_data = unsafe { CGDataProviderCopyData(provider) };
    if cf_data.is_null() {
        unsafe { CFRelease(image_ptr) };
        return Err("截图失败: CGDataProviderCopyData 返回 NULL".to_string());
    }

    let buf_ptr = unsafe { CFDataGetBytePtr(cf_data) };
    let buf_len = unsafe { CFDataGetLength(cf_data) } as usize;
    let bytes_per_row = unsafe { CGImageGetBytesPerRow(image_ptr) };

    let expected_min = width as usize * height as usize * 4;
    let mut rgba_data = if buf_len >= expected_min && bytes_per_row >= width as usize * 4 {
        // Copy row-by-row, discarding any trailing padding bytes in each row.
        let mut pixels = Vec::with_capacity(expected_min);
        let row_data = unsafe { std::slice::from_raw_parts(buf_ptr, buf_len) };
        for y in 0..height as usize {
            let start = y * bytes_per_row;
            pixels.extend_from_slice(&row_data[start..start + width as usize * 4]);
        }
        pixels
    } else if buf_len == expected_min {
        unsafe { std::slice::from_raw_parts(buf_ptr, buf_len) }.to_vec()
    } else {
        unsafe { CFRelease(cf_data) };
        unsafe { CFRelease(image_ptr) };
        return Err(format!(
            "截图像素数据长度不匹配: 期望 {} 实际 {} (bytesPerRow={})",
            expected_min, buf_len, bytes_per_row
        ));
    };

    unsafe { CFRelease(cf_data) };
    unsafe { CFRelease(image_ptr) };

    // CoreGraphics returns BGRA on macOS; frontend expects RGBA.
    // Swap bytes 0 <-> 2 (B <-> R), set alpha to 255.
    for pixel in rgba_data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        pixel[3] = 255;
    }

    let logical_width = width as f64 / scale_factor;
    let logical_height = height as f64 / scale_factor;

    dprintln!(
        "[color-picker-macos] captured display {}: {}x{} px, scale {:.2}",
        display_id,
        width,
        height,
        scale_factor
    );

    Ok(ColorPickerCapture {
        width,
        height,
        logical_width,
        logical_height,
        scale_factor,
        rgba_data,
    })
}

fn capture_monitor_via_cg_display(
    monitor: &Monitor,
    display_id: CGDirectDisplayID,
) -> Result<ColorPickerCapture, String> {
    let scale_factor = monitor.scale_factor();
    capture_display(display_id, scale_factor)
}

// ── Display Matching ─────────────────────────────────────────────────────────

/// Match each Tauri Monitor with a CoreGraphics display.
///
/// Both lists are sorted by their respective top-left Y, then X.
/// Tauri monitors are already in top-left coordinates.
/// CGDisplay bounds are in bottom-left coords; we convert to top-left
/// using the virtual desktop height.
fn match_monitors_to_displays(
    monitors: &[Monitor],
    display_bounds: &[(CGDirectDisplayID, CGRect)],
    virtual_height: f64,
) -> Vec<Option<CGDirectDisplayID>> {
    let mut sorted_monitors: Vec<(usize, &Monitor)> = monitors.iter().enumerate().collect();
    sorted_monitors.sort_by(|(_, a), (_, b)| {
        let pa = a.position();
        let pb = b.position();
        pa.y.cmp(&pb.y).then_with(|| pa.x.cmp(&pb.x))
    });

    let mut matched: Vec<Option<CGDirectDisplayID>> = vec![None; monitors.len()];

    for (monitor_idx, _) in sorted_monitors.iter() {
        let monitor = &monitors[*monitor_idx];
        let mpos = monitor.position();
        let msf = monitor.scale_factor();
        let mlx = mpos.x as f64 / msf;
        let mly = mpos.y as f64 / msf;
        let mw = monitor.size().width as f64 / msf;
        let mh = monitor.size().height as f64 / msf;

        // Find a CGDisplay whose top-left converted rect overlaps
        for (did, bounds) in display_bounds {
            let dlx = bounds.origin.x;
            let dly = virtual_height - bounds.origin.y - bounds.size.height;
            let dw = bounds.size.width;
            let dh = bounds.size.height;

            // Check if centers are close (within half-size of each)
            let mc_x = mlx + mw / 2.0;
            let mc_y = mly + mh / 2.0;
            let dc_x = dlx + dw / 2.0;
            let dc_y = dly + dh / 2.0;

            let dx = (mc_x - dc_x).abs();
            let dy = (mc_y - dc_y).abs();

            // Monitors match if their centers are within each other's half-dimensions
            if dx < (mw / 2.0 + dw / 2.0) * 0.1 && dy < (mh / 2.0 + dh / 2.0) * 0.1 {
                matched[*monitor_idx] = Some(*did);
                break;
            }
        }
    }

    matched
}

fn capture_all_screens(app: &AppHandle) -> Result<Vec<(Monitor, ColorPickerCapture)>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    if monitors.is_empty() {
        return Err("未检测到显示器".to_string());
    }

    let display_ids = get_active_display_ids()?;
    let (display_bounds, virtual_height) = get_display_bounds_with_virtual_height(&display_ids)?;

    let matched = match_monitors_to_displays(&monitors, &display_bounds, virtual_height);

    let mut results = Vec::new();
    for (i, monitor) in monitors.into_iter().enumerate() {
        let did =
            matched[i].ok_or_else(|| format!("无法将显示器 {} 映射到 CoreGraphics 显示", i))?;
        let capture = capture_monitor_via_cg_display(&monitor, did)?;
        results.push((monitor, capture));
    }

    Ok(results)
}

// ── Keyboard is handled natively by the WebView on macOS ─────────────────────

fn start_keyboard_polling(_app: AppHandle) {
    // On macOS, the focused WebView receives keyboard events natively,
    // so no Rust-side polling with GetAsyncKeyState equivalent is needed.
    dprintln!("[color-picker-macos] keyboard polling not needed on macOS");
}

fn stop_keyboard_polling() {
    // No-op on macOS
}

// ── Public API ───────────────────────────────────────────────────────────────

pub fn should_restore_main_on_finish() -> bool {
    RESTORE_MAIN_ON_FINISH.load(Ordering::Relaxed)
}

pub fn active_overlay_labels() -> Vec<String> {
    ACTIVE_OVERLAY_LABELS
        .lock()
        .map(|labels| labels.clone())
        .unwrap_or_else(|_| vec![OVERLAY_LABEL.to_string()])
}

pub fn get_color_picker_capture(label: Option<String>) -> Result<ColorPickerCapture, String> {
    let requested_label = label.unwrap_or_else(|| OVERLAY_LABEL.to_string());
    CACHED_CAPTURES
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
        .ok_or_else(|| "截图缓存不存在".to_string())
}

pub fn get_color_picker_image(label: Option<String>) -> Result<Vec<u8>, String> {
    let requested_label = label.unwrap_or_else(|| OVERLAY_LABEL.to_string());
    CACHED_CAPTURES
        .lock()
        .map_err(|_| "缓存锁定失败".to_string())?
        .get(&requested_label)
        .map(|capture| capture.rgba_data.clone())
        .ok_or_else(|| "截图缓存不存在".to_string())
}

pub fn clear_capture_cache() {
    stop_keyboard_polling();
    if let Ok(mut cache) = CACHED_CAPTURES.lock() {
        cache.clear();
    }
    if let Ok(mut specs) = ACTIVE_OVERLAY_SPECS.lock() {
        specs.clear();
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}

pub fn focus_color_picker_overlay(app: &AppHandle, _label: Option<String>) {
    if let Some(overlay) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = overlay.set_focus();
    }
}

fn elevate_overlay_window(app: &AppHandle, overlay: &tauri::WebviewWindow) {
    if let Ok(ns_window) = overlay.ns_window() {
        let app = app.clone();
        let ptr = ns_window as usize;
        let _ = app.run_on_main_thread(move || {
            let ptr = ptr as *mut objc2::runtime::NSObject;
            unsafe {
                let _: () = msg_send![ptr, setLevel: 53i32];
            }
        });
    }
}

fn hide_main_window_for_capture(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        dprintln!("[color-picker-macos] hiding main window");
        let _ = main.hide();
    }
}

fn attach_overlay_cleanup(app: AppHandle, overlay: &tauri::WebviewWindow) {
    overlay.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let is_abnormal = CACHED_CAPTURES
                .lock()
                .map(|cache| !cache.is_empty())
                .unwrap_or(false);
            if is_abnormal {
                clear_capture_cache();
                show_main_window(&app);
                let _ = app.emit("color_picker_result", Option::<String>::None);
            }
        }
    });
}

pub async fn start_color_picker(app: AppHandle, restore_main_window: bool) -> Result<(), String> {
    dprintln!("[color-picker-macos] start requested");

    RESTORE_MAIN_ON_FINISH.store(restore_main_window, Ordering::Relaxed);

    // 1. Hide main window
    hide_main_window_for_capture(&app);

    // 2. Small delay for the hide to take effect (no DWM flush needed on macOS)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 3. Capture all monitors
    let captures = capture_all_screens(&app)?;

    let mut overlay_specs: Vec<OverlaySpec> = captures
        .iter()
        .enumerate()
        .map(|(index, (monitor, capture))| {
            let pos = monitor.position();
            let size = monitor.size();
            OverlaySpec {
                label: overlay_label(index),
                position: *pos,
                size: *size,
                logical_width: capture.logical_width,
                logical_height: capture.logical_height,
                scale_factor: capture.scale_factor,
            }
        })
        .collect();

    dprintln!(
        "[color-picker-macos] captures ready: {} monitor(s)",
        overlay_specs.len()
    );

    // 4. Cache captures
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
        *labels = overlay_specs.iter().map(|s| s.label.clone()).collect();
    }
    {
        let mut specs = ACTIVE_OVERLAY_SPECS
            .lock()
            .map_err(|_| "窗口状态锁定失败".to_string())?;
        *specs = overlay_specs.clone();
    }

    // 5. Create overlay windows for each monitor
    for spec in &mut overlay_specs {
        if let Some(overlay) = app.get_webview_window(&spec.label) {
            dprintln!(
                "[color-picker-macos] reusing overlay window: {}",
                spec.label
            );
            let _ = overlay.set_position(tauri::Position::Physical(spec.position));
            let _ = overlay.set_size(tauri::Size::Physical(spec.size));
            let _ = overlay.set_always_on_top(true);
            elevate_overlay_window(&app, &overlay);
            overlay
                .emit("color_picker_capture_ready", ())
                .map_err(|err| {
                    dprintln!("[color-picker-macos] emit capture_ready failed: {err}");
                    clear_capture_cache();
                    show_main_window(&app);
                    err.to_string()
                })?;
            continue;
        }

        dprintln!(
            "[color-picker-macos] building overlay window on demand: {}",
            spec.label
        );

        let overlay = WebviewWindowBuilder::new(
            &app,
            &spec.label,
            WebviewUrl::App("/color-picker-overlay-macos".into()),
        )
        .title("Color Picker")
        .inner_size(spec.logical_width, spec.logical_height)
        .position(
            spec.position.x as f64 / spec.scale_factor,
            spec.position.y as f64 / spec.scale_factor,
        )
        .decorations(false)
        .transparent(false)
        .visible(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .focused(true)
        .shadow(false)
        .build()
        .map_err(|err| {
            dprintln!("[color-picker-macos] overlay build failed: {err}");
            clear_capture_cache();
            show_main_window(&app);
            err.to_string()
        })?;

        let _ = overlay.set_position(tauri::Position::Physical(spec.position));
        let _ = overlay.set_size(tauri::Size::Physical(spec.size));

        elevate_overlay_window(&app, &overlay);
        attach_overlay_cleanup(app.clone(), &overlay);
    }

    // 6. No keyboard polling needed on macOS (WebView handles keys natively)
    start_keyboard_polling(app.clone());

    // 7. Focus the first overlay
    if let Some(overlay) = app.get_webview_window(OVERLAY_LABEL) {
        let overlay_clone = overlay.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = overlay_clone.set_focus();
        });
    }

    dprintln!("[color-picker-macos] overlays ready");
    Ok(())
}
