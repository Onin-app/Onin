use crate::extensions::clipboard::monitor::ClipboardHistory;
use crate::extensions::clipboard::types::ClipboardItem;
use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};
use image::load_from_memory;
use tauri::{command, State};

#[command]
pub fn get_clipboard_history(state: State<'_, ClipboardHistory>) -> Vec<ClipboardItem> {
    state.get_all()
}

#[command]
pub fn set_clipboard_item(item: ClipboardItem) -> Result<(), String> {
    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;

    match item.item_type.as_str() {
        "Image" => {
            if let Some(thumbnail) = item.thumbnail {
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
        "File" => {
            let files: Vec<String> = item.text.lines().map(|s| s.to_string()).collect();
            ctx.set_files(files).map_err(|e| e.to_string())?;
        }
        _ => {
            ctx.set_text(item.text).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
