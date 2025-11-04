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
    /// 剪贴板内容的时间戳（Unix 时间戳，秒）
    pub timestamp: Option<u64>,
}

use std::sync::Mutex;
use once_cell::sync::Lazy;

// 全局存储剪贴板序列号和对应的时间戳
static CLIPBOARD_TIMESTAMP_MAP: Lazy<Mutex<std::collections::HashMap<u64, u64>>> = 
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

// 剪贴板监控状态
#[cfg(target_os = "windows")]
static CLIPBOARD_MONITOR_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// 获取剪贴板序列号并转换为时间戳（Windows）
#[cfg(target_os = "windows")]
fn get_clipboard_timestamp() -> Option<u64> {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    unsafe {
        let seq = GetClipboardSequenceNumber();
        if seq == 0 {
            println!("[Clipboard] Failed to get clipboard sequence number");
            return None;
        }
        
        let seq_u64 = seq as u64;
        let mut map = CLIPBOARD_TIMESTAMP_MAP.lock().unwrap();
        
        // 如果这个序列号已经存在，返回之前记录的时间戳
        if let Some(&timestamp) = map.get(&seq_u64) {
            println!("[Clipboard] Sequence {} already exists, timestamp: {}", seq_u64, timestamp);
            return Some(timestamp);
        }
        
        // 否则，记录当前时间戳
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();
        
        println!("[Clipboard] New sequence {}, recording timestamp: {}", seq_u64, timestamp);
        map.insert(seq_u64, timestamp);
        
        // 清理旧的条目（保留最近100个）
        if map.len() > 100 {
            let mut entries: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
            entries.sort_by_key(|(_, v)| *v);
            let to_remove: Vec<_> = entries.iter().take(map.len() - 100).map(|(k, _)| *k).collect();
            for key in to_remove {
                map.remove(&key);
            }
        }
        
        Some(timestamp)
    }
}

/// 启动剪贴板监控（Windows）
#[cfg(target_os = "windows")]
pub fn start_clipboard_monitor() {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    
    let mut started = CLIPBOARD_MONITOR_STARTED.lock().unwrap();
    if *started {
        println!("[Clipboard] Monitor already started");
        return;
    }
    *started = true;
    drop(started);
    
    println!("[Clipboard] Starting clipboard monitor...");
    
    std::thread::spawn(move || {
        let mut last_seq: u64 = 0;
        
        loop {
            unsafe {
                let seq = GetClipboardSequenceNumber() as u64;
                
                if seq != 0 && seq != last_seq {
                    // 检测到剪贴板变化，记录时间戳
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    let mut map = CLIPBOARD_TIMESTAMP_MAP.lock().unwrap();
                    if !map.contains_key(&seq) {
                        println!("[Clipboard Monitor] New clipboard content detected, seq: {}, timestamp: {}", seq, timestamp);
                        map.insert(seq, timestamp);
                        
                        // 清理旧的条目（保留最近100个）
                        if map.len() > 100 {
                            let mut entries: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
                            entries.sort_by_key(|(_, v)| *v);
                            let to_remove: Vec<_> = entries.iter().take(map.len() - 100).map(|(k, _)| *k).collect();
                            for key in to_remove {
                                map.remove(&key);
                            }
                        }
                    }
                    
                    last_seq = seq;
                }
            }
            
            // 每500ms检查一次
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

/// 读取剪贴板内容（文本或文件路径）
#[tauri::command]
pub async fn get_clipboard_content(
    app: AppHandle,
) -> Result<ClipboardContent, ClipboardError> {
    // 获取剪贴板时间戳（Windows）
    #[cfg(target_os = "windows")]
    let timestamp = get_clipboard_timestamp();
    
    #[cfg(not(target_os = "windows"))]
    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    };
    
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
                    timestamp,
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
                timestamp,
            })
        }
        Err(e) => {
            Err(ClipboardError::from(format!("Failed to read clipboard: {}", e)))
        }
    }
}