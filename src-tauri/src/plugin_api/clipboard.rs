use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardError {
    pub name: String,
    pub message: String,
    pub code: Option<String>,
}

impl From<String> for ClipboardError {
    fn from(message: String) -> Self {
        ClipboardError {
            name: "ClipboardError".to_string(),
            message,
            code: None,
        }
    }
}

impl From<&str> for ClipboardError {
    fn from(message: &str) -> Self {
        ClipboardError {
            name: "ClipboardError".to_string(),
            message: message.to_string(),
            code: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteTextOptions {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteImageOptions {
    #[serde(rename = "imageData")]
    pub image_data: Vec<u8>,
}

#[tauri::command]
pub async fn plugin_clipboard_read_text(
    app: AppHandle,
) -> Result<Option<String>, ClipboardError> {
    match app.clipboard().read_text() {
        Ok(text) => {
            println!("[Clipboard] Read text: {} chars", text.len());
            Ok(Some(text))
        }
        Err(e) => {
            println!("[Clipboard] Failed to read text: {}", e);
            Err(ClipboardError::from(format!("Failed to read clipboard text: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn plugin_clipboard_write_text(
    app: AppHandle,
    options: WriteTextOptions,
) -> Result<(), ClipboardError> {
    match app.clipboard().write_text(options.text.clone()) {
        Ok(_) => {
            println!("[Clipboard] Wrote text: {} chars", options.text.len());
            Ok(())
        }
        Err(e) => {
            println!("[Clipboard] Failed to write text: {}", e);
            Err(ClipboardError::from(format!("Failed to write clipboard text: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn plugin_clipboard_read_image(
    _app: AppHandle,
) -> Result<Option<String>, ClipboardError> {
    // 图像读取功能暂时不实现，因为需要更复杂的图像处理
    println!("[Clipboard] Image reading is not yet implemented");
    Err(ClipboardError::from("Image reading is not yet implemented. Please use text operations for now."))
}

#[tauri::command]
pub async fn plugin_clipboard_write_image(
    _app: AppHandle,
    _options: WriteImageOptions,
) -> Result<(), ClipboardError> {
    // 图像写入功能暂时不实现，因为需要更复杂的图像处理
    println!("[Clipboard] Image writing is not yet implemented");
    Err(ClipboardError::from("Image writing is not yet implemented. Please use text operations for now."))
}

#[tauri::command]
pub async fn plugin_clipboard_clear(
    app: AppHandle,
) -> Result<(), ClipboardError> {
    match app.clipboard().clear() {
        Ok(_) => {
            println!("[Clipboard] Cleared clipboard");
            Ok(())
        }
        Err(e) => {
            println!("[Clipboard] Failed to clear clipboard: {}", e);
            Err(ClipboardError::from(format!("Failed to clear clipboard: {}", e)))
        }
    }
}