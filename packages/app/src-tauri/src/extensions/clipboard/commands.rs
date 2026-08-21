use crate::extensions::clipboard::monitor::ClipboardHistory;
use crate::extensions::clipboard::types::ClipboardItem;
use base64::{engine::general_purpose, Engine as _};
#[cfg_attr(target_os = "windows", allow(unused_imports))]
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};
#[cfg_attr(target_os = "windows", allow(unused_imports))]
use image::load_from_memory;
use tauri::{command, Emitter, Manager, State};

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GlobalFree(hmem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

#[cfg(target_os = "windows")]
pub fn write_png_to_clipboard(png_bytes: &[u8]) -> Result<(), String> {
    use windows::Win32::Foundation::{HANDLE, HWND};
    use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
    };

    // 定义剪贴板标准格式常量，提高代码可读性
    const CF_DIB: u32 = 8;

    eprintln!(
        "[Clipboard] write_png_to_clipboard: writing {} bytes",
        png_bytes.len()
    );

    // 1. 注册 PNG 剪贴板格式
    let png_format_name: Vec<u16> = "PNG\0".encode_utf16().collect();
    let png_format =
        unsafe { RegisterClipboardFormatW(windows::core::PCWSTR(png_format_name.as_ptr())) };
    if png_format == 0 {
        return Err("RegisterClipboardFormatW failed".to_string());
    }

    // 2. 解码图片为 RGBA8，并就地将其转换为 BGRA8 格式
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)
        .map_err(|e| format!("Decode failed: {}", e))?;
    let mut rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    // 性能优化：就地调换 R (0) 和 B (2) 通道，以得到符合 DIB 位图要求的 BGRA 数组
    let raw_pixels = rgba.as_mut();
    for chunk in raw_pixels.chunks_exact_mut(4) {
        let r = chunk[0];
        chunk[0] = chunk[2];
        chunk[2] = r;
    }

    // 32bpp 每行刚好是 4 字节的倍数，无需额外的 stride 补齐计算
    let pixel_data_size = width * height * 4;
    let header_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    let total_size = header_size + pixel_data_size;

    unsafe {
        // 3. 打开剪贴板，使用 HWND::default() 免受窗口状态影响
        OpenClipboard(HWND::default()).map_err(|_| "OpenClipboard failed".to_string())?;
        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(format!("EmptyClipboard failed: {}", e));
        }

        // 4. 写入 PNG 原始数据
        let h_png = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, png_bytes.len()).map_err(|_| {
            let _ = CloseClipboard();
            "GlobalAlloc for PNG failed".to_string()
        })?;
        let ptr_png = GlobalLock(h_png) as *mut u8;
        if ptr_png.is_null() {
            let _ = GlobalFree((h_png.0 as usize) as *mut _);
            let _ = CloseClipboard();
            return Err("GlobalLock for PNG failed".to_string());
        }
        std::ptr::copy_nonoverlapping(png_bytes.as_ptr(), ptr_png, png_bytes.len());
        let _ = GlobalUnlock(h_png);

        if SetClipboardData(png_format, HANDLE(h_png.0 as isize)).is_err() {
            let _ = GlobalFree((h_png.0 as usize) as *mut _);
            let _ = CloseClipboard();
            return Err("SetClipboardData for PNG failed".to_string());
        }

        // 5. 写入 CF_DIB 标准位图数据 (正立 32bpp)
        let h_dib =
            GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size as usize).map_err(|_| {
                let _ = CloseClipboard();
                "GlobalAlloc for DIB failed".to_string()
            })?;
        let ptr_dib = GlobalLock(h_dib) as *mut u8;
        if ptr_dib.is_null() {
            let _ = GlobalFree((h_dib.0 as usize) as *mut _);
            let _ = CloseClipboard();
            return Err("GlobalLock for DIB failed".to_string());
        }

        let header = &mut *(ptr_dib as *mut BITMAPINFOHEADER);
        header.biSize = header_size;
        header.biWidth = width as i32;
        header.biHeight = -(height as i32); // 负数高度代表从顶往下的 Top-Down 图像
        header.biPlanes = 1;
        header.biBitCount = 32; // 32bpp
        header.biCompression = BI_RGB.0;
        header.biSizeImage = pixel_data_size;

        let pixels = ptr_dib.add(header_size as usize);
        std::ptr::copy_nonoverlapping(rgba.as_raw().as_ptr(), pixels, rgba.as_raw().len());
        let _ = GlobalUnlock(h_dib);

        if SetClipboardData(CF_DIB, HANDLE(h_dib.0 as isize)).is_err() {
            let _ = GlobalFree((h_dib.0 as usize) as *mut _);
            let _ = CloseClipboard();
            return Err("SetClipboardData for DIB failed".to_string());
        }

        CloseClipboard().map_err(|_| "CloseClipboard failed".to_string())?;
    }

    eprintln!("[Clipboard] write_png_to_clipboard: OK");
    Ok(())
}

#[command]
pub fn get_clipboard_history(state: State<'_, ClipboardHistory>) -> Vec<ClipboardItem> {
    state.get_all()
}

fn write_to_clipboard(app: &tauri::AppHandle, item: &ClipboardItem) -> Result<(), String> {
    let mut last_err = String::new();
    for _i in 0..5 {
        match write_to_clipboard_inner(app, item) {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_err = e;
                #[cfg(target_os = "windows")]
                {
                    eprintln!(
                        "[Clipboard] write failed (attempt {}): {}. Retrying in 50ms...",
                        _i + 1,
                        last_err
                    );
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err(last_err);
                }
            }
        }
    }
    Err(format!("Failed after 5 attempts. Last error: {}", last_err))
}

fn write_to_clipboard_inner(app: &tauri::AppHandle, item: &ClipboardItem) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if item.item_type == "Image" {
            if let Some(filename) = &item.image_path {
                if let Ok(app_data_dir) = app.path().app_data_dir() {
                    let image_file = app_data_dir
                        .join("extensions")
                        .join("clipboard")
                        .join("images")
                        .join(filename);
                    if image_file.exists() {
                        if let Ok(bytes) = std::fs::read(&image_file) {
                            // macOS Optimization: Write raw PNG bytes directly to NSPasteboard
                            // This bypasses image::load_from_memory decoding (CPU heavy) and re-encoding.
                            use objc2_app_kit::NSPasteboard;
                            use objc2_foundation::{NSData, NSString};

                            let pb = NSPasteboard::generalPasteboard();
                            let _ = pb.clearContents();

                            // Allow "public.png"
                            let type_png = NSString::from_str("public.png");
                            let ns_data = NSData::from_vec(bytes); // objc2-foundation 0.3+ supports this

                            let success = pb.setData_forType(Some(&ns_data), &type_png);
                            if success {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    match item.item_type.as_str() {
        "Image" => {
            let mut served = false;

            if let Some(filename) = &item.image_path {
                if let Ok(app_data_dir) = app.path().app_data_dir() {
                    let image_file = app_data_dir
                        .join("extensions")
                        .join("clipboard")
                        .join("images")
                        .join(filename);
                    if image_file.exists() {
                        if let Ok(bytes) = std::fs::read(&image_file) {
                            #[cfg(target_os = "windows")]
                            {
                                if write_png_to_clipboard(&bytes).is_ok() {
                                    served = true;
                                }
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                if let Ok(img) = image::load_from_memory_with_format(
                                    &bytes,
                                    image::ImageFormat::Png,
                                ) {
                                    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;
                                    let rust_image = RustImage::from_dynamic_image(img);
                                    if ctx.set_image(rust_image).is_ok() {
                                        served = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !served {
                if let Some(thumbnail) = &item.thumbnail {
                    let parts: Vec<&str> = thumbnail.split(',').collect();
                    if parts.len() != 2 {
                        return Err("Invalid thumbnail format".to_string());
                    }
                    let bytes = general_purpose::STANDARD
                        .decode(parts[1])
                        .map_err(|e| e.to_string())?;

                    #[cfg(target_os = "windows")]
                    {
                        write_png_to_clipboard(&bytes)?;
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let img = load_from_memory(&bytes).map_err(|e| e.to_string())?;
                        let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;
                        let rust_image = RustImage::from_dynamic_image(img);
                        ctx.set_image(rust_image).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        "File" | _ => {
            let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;
            if item.item_type == "File" {
                let files: Vec<String> = item.text.lines().map(|s| s.to_string()).collect();
                ctx.set_files(files).map_err(|e| e.to_string())?;
            } else {
                ctx.set_text(item.text.clone()).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[command]
pub fn set_clipboard_item(
    app: tauri::AppHandle,
    item: ClipboardItem,
    state: State<'_, ClipboardHistory>,
) -> Result<(), String> {
    // 先将记录移到最前面
    state.move_to_front(&app, &item.id);

    // 设置跳过标志,避免监听器将此次操作记录为新项
    state.set_skip_next();

    // 触发更新事件,通知前端刷新列表
    let _ = app.emit("clipboard-update", ());

    write_to_clipboard(&app, &item)
}

#[command]
pub fn paste_clipboard_item(
    app: tauri::AppHandle,
    item_id: String,
    state: State<'_, ClipboardHistory>,
) -> Result<(), String> {
    let app_clone = app.clone();
    let state_clone = state.inner().clone();

    tauri::async_runtime::spawn(async move {
        // 0. 从内存获取 Item
        let item = match state_clone.get(&item_id) {
            Some(i) => i,
            None => {
                eprintln!("[Clipboard] Paste failed: Item not found {}", item_id);
                return;
            }
        };

        // 1. 同步设置跳过监听 (内存操作，极快，防止竞态)
        state_clone.set_skip_next();

        // 2. 立即写入剪贴板 (优先执行)
        if let Err(e) = write_to_clipboard(&app_clone, &item) {
            eprintln!(
                "[Clipboard] Failed to write to clipboard in background: {}",
                e
            );
            return;
        }

        // 3. 并行执行数据库写盘和前端更新事件，不阻塞粘贴流程
        let app_db = app_clone.clone();
        let state_db = state_clone.clone();
        let item_id_db = item.id.clone();
        tauri::async_runtime::spawn(async move {
            state_db.move_to_front(&app_db, &item_id_db);
            let _ = app_db.emit("clipboard-update", ());
        });

        // 4. 等待窗口隐藏完成 (调优为 40ms 延迟)
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;

        // 5. 模拟粘贴
        if let Err(e) = crate::system_commands::simulate_paste_native(&app_clone) {
            eprintln!("[Clipboard] Failed to simulate paste in background: {}", e);
        }
    });

    Ok(())
}
