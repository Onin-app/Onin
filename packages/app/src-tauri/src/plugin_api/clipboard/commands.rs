//! Tauri 命令模块

use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::timestamp::{get_clipboard_timestamp, get_plugin_clipboard_timestamp};
use super::types::{
    ClipboardContent, ClipboardContentType, ClipboardError, ClipboardFile, ClipboardMetadata,
    WriteImageOptions, WriteTextOptions,
};

/// 读取剪贴板文本
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
            Err(ClipboardError::from(format!(
                "Failed to read clipboard text: {}",
                e
            )))
        }
    }
}

/// 写入剪贴板文本
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
            Err(ClipboardError::from(format!(
                "Failed to write clipboard text: {}",
                e
            )))
        }
    }
}

/// 读取剪贴板图片（暂未实现）
#[tauri::command]
pub async fn plugin_clipboard_read_image(
    _app: AppHandle,
) -> Result<Option<String>, ClipboardError> {
    println!("[Clipboard] Image reading is not yet implemented");
    Err(ClipboardError::from(
        "Image reading is not yet implemented. Please use text operations for now.",
    ))
}

/// 写入剪贴板图片（暂未实现）
#[tauri::command]
pub async fn plugin_clipboard_write_image(
    _app: AppHandle,
    _options: WriteImageOptions,
) -> Result<(), ClipboardError> {
    println!("[Clipboard] Image writing is not yet implemented");
    Err(ClipboardError::from(
        "Image writing is not yet implemented. Please use text operations for now.",
    ))
}

/// 清空剪贴板
#[tauri::command]
pub async fn plugin_clipboard_clear(app: AppHandle) -> Result<(), ClipboardError> {
    match app.clipboard().clear() {
        Ok(_) => {
            println!("[Clipboard] Cleared clipboard");
            Ok(())
        }
        Err(e) => {
            println!("[Clipboard] Failed to clear clipboard: {}", e);
            Err(ClipboardError::from(format!(
                "Failed to clear clipboard: {}",
                e
            )))
        }
    }
}

/// 获取剪贴板元数据
#[tauri::command]
pub async fn plugin_clipboard_get_metadata(
    app: AppHandle,
) -> Result<ClipboardMetadata, ClipboardError> {
    let timestamp = get_plugin_clipboard_timestamp();

    let age = timestamp.map(|ts| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(ts)
    });

    let mut content_type = ClipboardContentType::Empty;
    let mut text: Option<String> = None;
    let mut files: Option<Vec<ClipboardFile>> = None;

    // 先尝试读取文件路径（Windows/Linux）
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        use clipboard_rs::{Clipboard, ClipboardContext};
        use std::path::Path;

        if let Ok(ctx) = ClipboardContext::new() {
            if let Ok(file_list) = ctx.get_files() {
                if !file_list.is_empty() {
                    let mut clipboard_files = Vec::new();

                    for file_path in file_list.iter() {
                        let path = Path::new(file_path);
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let is_directory = path.is_dir();

                        clipboard_files.push(ClipboardFile {
                            path: file_path.clone(),
                            name,
                            is_directory,
                        });
                    }

                    content_type = ClipboardContentType::Files;
                    files = Some(clipboard_files);
                }
            }
        }
    }

    // 如果没有文件，尝试读取文本
    if content_type == ClipboardContentType::Empty {
        match app.clipboard().read_text() {
            Ok(clipboard_text) => {
                if !clipboard_text.is_empty() {
                    content_type = ClipboardContentType::Text;
                    text = Some(clipboard_text);
                }
            }
            Err(_) => {}
        }
    }

    // 检测是否有图片
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        use clipboard_rs::{Clipboard, ClipboardContext};

        if content_type == ClipboardContentType::Empty {
            if let Ok(ctx) = ClipboardContext::new() {
                if ctx.get_image().is_ok() {
                    content_type = ClipboardContentType::Image;
                }
            }
        }
    }

    println!(
        "[Clipboard] Plugin read metadata: type={:?}, has_text={}, has_files={}, timestamp={:?}, age={:?}",
        content_type,
        text.is_some(),
        files.is_some(),
        timestamp,
        age
    );

    Ok(ClipboardMetadata {
        text,
        files,
        content_type,
        timestamp,
        age,
    })
}

/// 读取剪贴板内容
#[tauri::command]
pub async fn get_clipboard_content(
    app: AppHandle,
) -> Result<ClipboardContent, ClipboardError> {
    let timestamp = get_clipboard_timestamp();

    // 先尝试读取文件路径
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        use clipboard_rs::{Clipboard, ClipboardContext};
        use std::path::Path;

        if let Ok(ctx) = ClipboardContext::new() {
            if let Ok(files) = ctx.get_files() {
                let mut clipboard_files = Vec::new();

                for file_path in files.iter() {
                    let path = Path::new(file_path);
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let is_directory = path.is_dir();

                    clipboard_files.push(ClipboardFile {
                        path: file_path.clone(),
                        name,
                        is_directory,
                    });
                }

                if !clipboard_files.is_empty() {
                    return Ok(ClipboardContent {
                        text: None,
                        files: Some(clipboard_files),
                        timestamp,
                    });
                }
            }
        }
    }

    // 如果没有文件，尝试读取文本
    match app.clipboard().read_text() {
        Ok(text) => Ok(ClipboardContent {
            text: Some(text),
            files: None,
            timestamp,
        }),
        Err(e) => Err(ClipboardError::from(format!(
            "Failed to read clipboard: {}",
            e
        ))),
    }
}
