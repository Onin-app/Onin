use crate::extensions::clipboard::monitor::ClipboardHistory;
use crate::extensions::clipboard::types::ClipboardItem;
use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};
use image::load_from_memory;
use tauri::{command, Emitter, Manager, State};

#[command]
pub fn get_clipboard_history(state: State<'_, ClipboardHistory>) -> Vec<ClipboardItem> {
    state.get_all()
}

fn write_to_clipboard(app: &tauri::AppHandle, item: &ClipboardItem) -> Result<(), String> {
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

    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;

    match item.item_type.as_str() {
        "Image" => {
            // Optimization: Try to read from file first to avoid Base64 decoding overhead
            let mut served_from_file = false;

            if let Some(filename) = &item.image_path {
                if let Ok(app_data_dir) = app.path().app_data_dir() {
                    let image_file = app_data_dir
                        .join("extensions")
                        .join("clipboard")
                        .join("images")
                        .join(filename);
                    if image_file.exists() {
                        if let Ok(bytes) = std::fs::read(&image_file) {
                            if let Ok(img) = load_from_memory(&bytes) {
                                let rust_image = RustImage::from_dynamic_image(img);
                                if let Ok(_) = ctx.set_image(rust_image) {
                                    served_from_file = true;
                                }
                            }
                        }
                    }
                }
            }

            if !served_from_file {
                if let Some(thumbnail) = &item.thumbnail {
                    // thumbnail format: "data:image/png;base64,..."
                    let parts: Vec<&str> = thumbnail.split(',').collect();
                    if parts.len() != 2 {
                        return Err("Invalid thumbnail format".to_string());
                    }
                    let base64_data = parts[1];
                    let bytes = general_purpose::STANDARD
                        .decode(base64_data)
                        .map_err(|e| e.to_string())?;

                    let img = load_from_memory(&bytes).map_err(|e| e.to_string())?;
                    let rust_image = RustImage::from_dynamic_image(img);
                    ctx.set_image(rust_image).map_err(|e| e.to_string())?;
                }
            }
        }
        "File" => {
            let files: Vec<String> = item.text.lines().map(|s| s.to_string()).collect();
            ctx.set_files(files).map_err(|e| e.to_string())?;
        }
        _ => {
            ctx.set_text(item.text.clone()).map_err(|e| e.to_string())?;
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
    // 立即返回，后台执行所有耗时操作
    let app = app.clone();
    let state = state.inner().clone();

    tauri::async_runtime::spawn(async move {
        // 0. 从内存获取 Item (避免 IPC 传输大图)
        let item = match state.get(&item_id) {
            Some(i) => i,
            None => {
                eprintln!("[Clipboard] Paste failed: Item not found {}", item_id);
                return;
            }
        };

        // 1. 数据库更新 (Move to front)
        state.move_to_front(&app, &item.id);

        // 2. 设置跳过监听
        state.set_skip_next();

        // 3. 通知前端更新
        let _ = app.emit("clipboard-update", ());

        // 4. 写入剪贴板 (耗时操作)
        if let Err(e) = write_to_clipboard(&app, &item) {
            eprintln!(
                "[Clipboard] Failed to write to clipboard in background: {}",
                e
            );
            return;
        }

        // 5. 模拟粘贴 (Native or Script)
        // 此时窗口应该已经隐藏（前端负责），我们这里只负责模拟按键
        if let Err(e) = crate::system_commands::simulate_paste_native(&app) {
            eprintln!("[Clipboard] Failed to simulate paste in background: {}", e);
        }
    });

    Ok(())
}
