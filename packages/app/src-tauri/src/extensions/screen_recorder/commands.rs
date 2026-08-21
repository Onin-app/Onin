use super::engine::{RecordConfig, RecordStateSnapshot, ScreenRecordEngine};
use std::sync::Arc;
use tauri::{command, AppHandle, Manager, State};

#[cfg(target_os = "windows")]
use base64::Engine;
#[cfg(target_os = "windows")]
use std::io::Cursor;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::*;

pub struct RecorderAppState {
    pub engine: Arc<dyn ScreenRecordEngine>,
    pub config: std::sync::Mutex<RecordConfig>,
}

fn get_recordings_dir(
    app: &AppHandle,
    config: &RecordConfig,
) -> Result<std::path::PathBuf, String> {
    let folder_type = config.save_folder_type.as_deref().unwrap_or("video");
    let path = match folder_type {
        "download" => app.path().download_dir().map(|p| p.join("Onin_Recordings")),
        "desktop" => app.path().desktop_dir().map(|p| p.join("Onin_Recordings")),
        "custom" => {
            if let Some(ref custom_path) = config.custom_save_folder {
                let p = std::path::PathBuf::from(custom_path);
                if p.exists() && p.is_dir() {
                    Ok(p)
                } else {
                    app.path().video_dir().map(|p| p.join("Onin_Recordings"))
                }
            } else {
                app.path().video_dir().map(|p| p.join("Onin_Recordings"))
            }
        }
        _ => app.path().video_dir().map(|p| p.join("Onin_Recordings")),
    }
    .map_err(|e| format!("Failed to resolve directory: {}", e))?;
    Ok(path)
}

#[command]
pub async fn start_screen_record(
    app: AppHandle,
    state: State<'_, RecorderAppState>,
    config: RecordConfig,
) -> Result<String, String> {
    let video_dir = get_recordings_dir(&app, &config)?;

    // 确保录屏文件夹存在
    std::fs::create_dir_all(&video_dir)
        .map_err(|e| format!("Failed to create recording directory: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("Onin_{}.mp4", timestamp);
    let output_path = video_dir.join(file_name);

    state.engine.start(&config, &output_path, &app)?;

    Ok(output_path.to_string_lossy().to_string())
}

#[command]
pub async fn pause_screen_record(state: State<'_, RecorderAppState>) -> Result<(), String> {
    state.engine.pause()
}

#[command]
pub async fn resume_screen_record(state: State<'_, RecorderAppState>) -> Result<(), String> {
    state.engine.resume()
}

#[command]
pub async fn stop_screen_record(
    app: tauri::AppHandle,
    state: State<'_, RecorderAppState>,
) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("screen-recorder-area-indicator") {
        let _ = w.close();
    }
    state.engine.stop()
}

#[command]
pub fn get_screen_record_state(state: State<'_, RecorderAppState>) -> RecordStateSnapshot {
    state.engine.get_state()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedVideo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_time: u64,
}

#[command]
pub fn get_recorded_videos(
    app: AppHandle,
    state: State<'_, RecorderAppState>,
) -> Result<Vec<RecordedVideo>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let video_dir = get_recordings_dir(&app, &config)?;

    if !video_dir.exists() {
        return Ok(Vec::new());
    }

    let re = regex::Regex::new(r"^Onin_\d{8}_\d{6}\.mp4$").unwrap();

    let mut videos = Vec::new();
    if let Ok(entries) = std::fs::read_dir(video_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if re.is_match(&name) {
                    if let Ok(metadata) = entry.metadata() {
                        let size_bytes = metadata.len();
                        let created_time = metadata
                            .created()
                            .or_else(|_| metadata.modified())
                            .map(|t| {
                                t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as u64
                            })
                            .unwrap_or(0);
                        videos.push(RecordedVideo {
                            name,
                            path: path.to_string_lossy().to_string(),
                            size_bytes,
                            created_time,
                        });
                    }
                }
            }
        }
    }

    // 按时间倒序
    videos.sort_by(|a, b| b.created_time.cmp(&a.created_time));
    Ok(videos)
}

#[command]
pub async fn open_video_file(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(&path, None::<String>)
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_video_file(path: String) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

#[command]
pub fn show_recorder_bar_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("screen-recorder-bar") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub thumbnail: String,
    pub scale_factor: f64,
}

#[cfg(target_os = "windows")]
fn capture_monitor_thumbnail(monitor: &tauri::Monitor) -> Result<String, String> {
    let size = monitor.size();
    let width = size.width;
    let height = size.height;

    unsafe {
        // 1. 获取物理显示器设备名称，转为 UTF-16 宽字符以供给 CreateDCW 使用
        let name_str = monitor.name().map(|v| v.as_str()).unwrap_or("");
        let name_wide: Vec<u16> = name_str.encode_utf16().chain(std::iter::once(0)).collect();

        // 2. 利用 CreateDCW 获取该专属独立物理屏幕的 DC
        // 从而直接在当前显示器的硬件显存画布上完成高速复制，避开拉取全屏幕虚拟桌面 DC 的庞大性能消耗
        let display_hdc = CreateDCW(None, windows::core::PCWSTR(name_wide.as_ptr()), None, None);

        if display_hdc.0 == 0 {
            return Err(format!("Failed to CreateDCW for monitor: {}", name_str));
        }

        // 3. 创建兼容 DC 和 compatible bitmap
        let mem_hdc = CreateCompatibleDC(display_hdc);
        if mem_hdc.0 == 0 {
            let _ = DeleteDC(display_hdc);
            return Err("Failed to create compatible dc".to_string());
        }

        let bitmap = CreateCompatibleBitmap(display_hdc, width as i32, height as i32);
        if bitmap.0 == 0 {
            let _ = DeleteDC(mem_hdc);
            let _ = DeleteDC(display_hdc);
            return Err("Failed to create compatible bitmap".to_string());
        }

        let old_obj = SelectObject(mem_hdc, bitmap);

        // 4. 从该独立显示器的局点 (0, 0) 直接高速复制
        let ok = BitBlt(
            mem_hdc,
            0,
            0,
            width as i32,
            height as i32,
            display_hdc,
            0,
            0,
            SRCCOPY,
        );

        if ok.is_err() {
            SelectObject(mem_hdc, old_obj);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_hdc);
            let _ = DeleteDC(display_hdc);
            return Err("BitBlt failed".to_string());
        }

        // 5. 获取 DIB 位图字节
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // 负数代表自上而下
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bgra = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            mem_hdc,
            bitmap,
            0,
            height,
            Some(bgra.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // 6. 清理 GDI 资源，display_hdc 必须通过 DeleteDC 销毁
        SelectObject(mem_hdc, old_obj);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem_hdc);
        let _ = DeleteDC(display_hdc);

        if lines == 0 {
            return Err("GetDIBits failed".to_string());
        }

        // 5. 转换 BGRA 为 RGBA
        for chunk in bgra.chunks_exact_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
        }

        // 6. 用 image 库进行缩放和压缩
        if let Some(img_buf) =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, bgra)
        {
            // 计算缩放比例，目标宽度 280 物理像素
            let target_w = 280;
            let target_h = ((height as f32 / width as f32) * target_w as f32) as u32;

            let thumb = image::imageops::thumbnail(&img_buf, target_w, target_h);

            let mut png_data = Cursor::new(Vec::new());
            if thumb
                .write_to(&mut png_data, image::ImageFormat::Png)
                .is_ok()
            {
                let base64_str =
                    base64::engine::general_purpose::STANDARD.encode(png_data.get_ref());
                return Ok(format!("data:image/png;base64,{}", base64_str));
            }
        }

        Err("Failed to compress thumbnail".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn capture_monitor_thumbnail(_monitor: &tauri::Monitor) -> Result<String, String> {
    Ok("".to_string())
}

#[command]
pub fn get_available_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;

    let mut infos = Vec::new();
    for (i, m) in monitors.iter().enumerate() {
        let name = m
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("显示器 {}", i + 1));
        let size = m.size();
        let scale_factor = m.scale_factor();
        let is_primary = if let Ok(Some(pm)) = app.primary_monitor() {
            pm.name() == m.name()
        } else {
            false
        };

        // 捕获缩略图，若失败则返回空字符串
        let thumbnail = capture_monitor_thumbnail(m).unwrap_or_default();

        infos.push(MonitorInfo {
            name,
            width: size.width,
            height: size.height,
            is_primary,
            thumbnail,
            scale_factor,
        });
    }
    Ok(infos)
}

#[command]
pub fn save_recorder_config(
    state: State<'_, RecorderAppState>,
    config: RecordConfig,
) -> Result<(), String> {
    let mut current_config = state.config.lock().map_err(|e| e.to_string())?;
    *current_config = config;
    Ok(())
}

#[command]
pub fn get_recorder_config(state: State<'_, RecorderAppState>) -> Result<RecordConfig, String> {
    let current_config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(current_config.clone())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub handle: String,
    pub title: String,
    pub process_name: String,
    pub width: u32,
    pub height: u32,
}

#[command]
pub fn get_available_windows(_app: AppHandle) -> Result<Vec<WindowInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        use windows_capture::window::Window;
        let win_list = Window::enumerate().map_err(|e| e.to_string())?;
        let mut infos = Vec::new();
        for w in win_list {
            if !w.is_valid() {
                continue;
            }
            let title = match w.title() {
                Ok(t) if !t.trim().is_empty() => t,
                _ => continue,
            };
            let process_name = w.process_name().unwrap_or_default();
            let process_name_lower = process_name.to_lowercase();

            // 过滤自身应用窗口（Onin）
            if process_name_lower.contains("onin") || title.contains("Onin") {
                continue;
            }

            let hwnd_ptr = w.as_raw_hwnd();
            use windows::Win32::Foundation::HWND;
            let hwnd = HWND(hwnd_ptr as isize);
            if !is_real_window(hwnd) {
                continue;
            }

            let (width, height) = get_window_size(hwnd_ptr as isize);
            if width == 0 || height == 0 {
                continue;
            }

            let handle = (hwnd_ptr as isize).to_string();

            infos.push(WindowInfo {
                handle,
                title,
                process_name,
                width,
                height,
            });
        }
        Ok(infos)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "windows")]
fn get_window_size(hwnd_val: isize) -> (u32, u32) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    let hwnd = HWND(hwnd_val);
    let mut rect = windows::Win32::Foundation::RECT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 0 && h > 0 {
                return (w as u32, h as u32);
            }
        }
    }
    (0, 0)
}

#[cfg(target_os = "windows")]
fn is_real_window(hwnd: windows::Win32::Foundation::HWND) -> bool {
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
    use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    unsafe {
        // 1. 窗口必须是可见的
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        // 2. 检查窗口是否被 DWM 遮蔽 (Cloaked)
        // 被遮蔽的窗口包括：隐藏的 UWP 后台挂起窗口 (比如已关闭的设置窗口)、虚拟桌面上的其他窗口等
        let mut cloaked: u32 = 0;
        let attribute_size = std::mem::size_of::<u32>() as u32;
        let res = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut std::ffi::c_void,
            attribute_size,
        );
        if res.is_ok() && cloaked != 0 {
            return false;
        }
    }

    true
}
