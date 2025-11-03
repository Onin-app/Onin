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

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardFile {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub files: Option<Vec<ClipboardFile>>,
}

/// 读取剪贴板内容（文本或文件路径）
#[tauri::command]
pub async fn get_clipboard_content(
    app: AppHandle,
) -> Result<ClipboardContent, ClipboardError> {
    // 先尝试读取文件路径
    #[cfg(target_os = "windows")]
    {
        use clipboard_win::{formats, get_clipboard};
        use std::path::{Path, PathBuf};
        
        // 尝试读取文件列表
        if let Ok(files) = get_clipboard::<Vec<PathBuf>, _>(formats::FileList) {
            let mut clipboard_files = Vec::new();
            
            for file_path in files.iter() {
                let path_str = file_path.to_string_lossy().to_string();
                let path = Path::new(&path_str);
                
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let is_directory = path.is_dir();
                
                clipboard_files.push(ClipboardFile {
                    path: path_str,
                    name,
                    is_directory,
                });
            }
            
            if !clipboard_files.is_empty() {
                return Ok(ClipboardContent {
                    text: None,
                    files: Some(clipboard_files),
                });
            }
        }
    }
    
    // 如果没有文件，尝试读取文本
    match app.clipboard().read_text() {
        Ok(text) => {
            Ok(ClipboardContent {
                text: Some(text),
                files: None,
            })
        }
        Err(e) => {
            Err(ClipboardError::from(format!("Failed to read clipboard: {}", e)))
        }
    }
}