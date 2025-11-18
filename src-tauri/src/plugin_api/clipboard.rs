use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
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
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

// ============================================================================
// 时间戳设计说明
// ============================================================================
// 
// 这里维护了两个独立的时间戳：
//
// 1. CLIPBOARD_TIMESTAMP (应用层时间戳)
//    - 用于应用的自动粘贴功能 (auto_paste_time_limit)
//    - 用于应用的自动清空功能 (auto_clear_time_limit)
//    - 会被应用的清理机制重置为 0
//    - 影响主输入框的自动粘贴行为
//
// 2. PLUGIN_CLIPBOARD_TIMESTAMP (插件层时间戳)
//    - 专门为插件 API 提供
//    - 记录系统剪贴板实际变化的时间
//    - 不受应用清理机制影响
//    - 保证插件能获取真实的剪贴板状态
//
// 为什么要分开？
// - 职责分离：应用的清理机制不应该影响插件获取真实数据
// - 数据一致性：剪贴板内容还在，时间戳不应该被清空
// - 插件独立性：插件应该能独立判断内容的年龄
//
// 示例场景：
// - 用户复制了文本 "hello"
// - 5 秒后，应用窗口隐藏
// - 10 秒后，auto_clear_time_limit 触发，CLIPBOARD_TIMESTAMP 被重置为 0
// - 但系统剪贴板仍然有 "hello"
// - 插件调用 getMetadata() 应该能看到：
//   - text: "hello"
//   - timestamp: 15 秒前的时间戳 (来自 PLUGIN_CLIPBOARD_TIMESTAMP)
//   - age: 15 秒
// - 而不是 timestamp: null, age: null
//
// ============================================================================

// 应用层时间戳：用于应用的自动粘贴和清空功能
static CLIPBOARD_TIMESTAMP: Lazy<Mutex<u64>> = 
    Lazy::new(|| Mutex::new(0));

// 插件层时间戳：专门为插件 API 提供，不受应用清理影响
static PLUGIN_CLIPBOARD_TIMESTAMP: Lazy<Mutex<u64>> = 
    Lazy::new(|| Mutex::new(0));

// 剪贴板监控状态
static CLIPBOARD_MONITOR_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// 全局存储 AppHandle 用于定时清空剪贴板
static APP_HANDLE: Lazy<Mutex<Option<Arc<AppHandle>>>> = Lazy::new(|| Mutex::new(None));

// 全局存储窗口隐藏时的时间戳
static WINDOW_HIDE_TIMESTAMP: Lazy<Mutex<Option<u64>>> = Lazy::new(|| Mutex::new(None));

/// 更新剪贴板时间戳
/// 
/// 当系统剪贴板内容发生变化时调用此函数。
/// 会同时更新应用层和插件层的时间戳。
fn update_clipboard_timestamp() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // 更新应用层时间戳（用于自动粘贴和清空功能）
    let mut ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
    *ts = timestamp;
    drop(ts);
    
    // 更新插件层时间戳（用于插件 API，不受应用清理影响）
    let mut plugin_ts = PLUGIN_CLIPBOARD_TIMESTAMP.lock().unwrap();
    *plugin_ts = timestamp;
    
    println!("[Clipboard] Updated timestamp: {}", timestamp);
}

/// 获取应用层剪贴板时间戳
/// 
/// 用于应用的自动粘贴和清空功能。
/// 会被 auto_clear_time_limit 重置为 0。
fn get_clipboard_timestamp() -> Option<u64> {
    let ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
    if *ts == 0 {
        None
    } else {
        Some(*ts)
    }
}

/// 获取插件层剪贴板时间戳
/// 
/// 专门为插件 API 提供，返回系统剪贴板实际变化的时间。
/// 不受应用的 auto_clear_time_limit 影响。
/// 
/// 设计原因：
/// - 插件应该能获取真实的剪贴板状态
/// - 应用的清理机制是为了控制应用自己的行为，不应该影响插件
/// - 保证数据一致性：如果剪贴板有内容，就应该有对应的时间戳
fn get_plugin_clipboard_timestamp() -> Option<u64> {
    let ts = PLUGIN_CLIPBOARD_TIMESTAMP.lock().unwrap();
    if *ts == 0 {
        None
    } else {
        Some(*ts)
    }
}

/// 启动剪贴板监控（使用 clipboard-rs）
pub fn start_clipboard_monitor(app: AppHandle) {
    let mut started = CLIPBOARD_MONITOR_STARTED.lock().unwrap();
    if *started {
        println!("[Clipboard] Monitor already started");
        return;
    }
    *started = true;
    drop(started);
    
    // 保存 AppHandle
    {
        let mut handle = APP_HANDLE.lock().unwrap();
        *handle = Some(Arc::new(app.clone()));
    }
    
    println!("[Clipboard] Starting clipboard monitor with clipboard-rs...");
    
    // 初始化时间戳
    update_clipboard_timestamp();
    
    // 启动剪贴板变化监控线程
    std::thread::spawn(move || {
        use clipboard_rs::{
            ClipboardContext, ClipboardHandler, 
            ClipboardWatcher, ClipboardWatcherContext
        };
        
        struct ClipboardManager {
            #[allow(dead_code)]
            ctx: ClipboardContext,
        }
        
        impl ClipboardManager {
            pub fn new() -> Self {
                let ctx = ClipboardContext::new().unwrap();
                ClipboardManager { ctx }
            }
        }
        
        impl ClipboardHandler for ClipboardManager {
            fn on_clipboard_change(&mut self) {
                println!("[Clipboard Monitor] Clipboard content changed");
                update_clipboard_timestamp();
            }
        }
        
        let manager = ClipboardManager::new();
        let mut watcher = ClipboardWatcherContext::new().unwrap();
        watcher.add_handler(manager);
        
        println!("[Clipboard Monitor] Watcher started");
        watcher.start_watch();
    });
    
    // 启动定时清空检查线程
    std::thread::spawn(move || {
        use crate::app_config::AppConfigState;
        use tauri::Emitter;
        
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            // 获取 AppHandle
            let app_handle = {
                let handle = APP_HANDLE.lock().unwrap();
                handle.as_ref().map(|h| Arc::clone(h))
            };
            
            if let Some(app) = app_handle {
                // 检查窗口是否隐藏
                let window_hidden = if let Some(window) = app.get_webview_window("main") {
                    !window.is_visible().unwrap_or(true)
                } else {
                    false
                };
                
                // 只有在窗口隐藏时才执行自动清空逻辑
                if !window_hidden {
                    // 窗口可见，重置窗口隐藏时间戳
                    let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                    *hide_ts = None;
                    continue;
                }
                
                // 记录窗口隐藏的时间戳
                let hide_timestamp = {
                    let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                    if hide_ts.is_none() {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        *hide_ts = Some(now);
                        now
                    } else {
                        hide_ts.unwrap()
                    }
                };
                
                // 获取配置
                let config_state = app.state::<AppConfigState>();
                let config = config_state.0.lock().unwrap();
                let auto_clear_time_limit = config.auto_clear_time_limit;
                drop(config);
                
                // 如果设置了自动清空时间限制
                if auto_clear_time_limit > 0 {
                    if get_clipboard_timestamp().is_some() {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        // 计算从窗口隐藏开始的时间
                        let elapsed_since_hide = now - hide_timestamp;
                        
                        // 只有当窗口隐藏时间超过设置的时间限制时才清空
                        if elapsed_since_hide >= auto_clear_time_limit {
                            println!("[Clipboard Monitor] Window hidden for {} seconds, clearing app clipboard content", elapsed_since_hide);
                            
                            // 发送事件到前端，让前端清空应用内部的剪贴板内容
                            if let Err(e) = app.emit("clear_app_clipboard", ()) {
                                println!("[Clipboard Monitor] Failed to emit clear event: {}", e);
                            } else {
                                println!("[Clipboard Monitor] Clear event sent to frontend");
                            }
                            
                            // 重置时间戳，避免重复清空
                            let mut ts = CLIPBOARD_TIMESTAMP.lock().unwrap();
                            *ts = 0;
                            // 重置窗口隐藏时间戳
                            let mut hide_ts = WINDOW_HIDE_TIMESTAMP.lock().unwrap();
                            *hide_ts = None;
                        }
                    }
                }
            }
        }
    });
}

/// 剪贴板内容类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardContentType {
    Text,
    Image,
    Files,
    Empty,
}

/// 插件 API：获取剪贴板元数据（增强版）
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardMetadata {
    /// 剪贴板文本内容（如果有）
    pub text: Option<String>,
    /// 文件路径列表（如果有）
    pub files: Option<Vec<ClipboardFile>>,
    /// 内容类型
    #[serde(rename = "contentType")]
    pub content_type: ClipboardContentType,
    /// Unix 时间戳（秒），剪贴板内容最后更新的时间
    /// 注意：如果应用配置了 auto_clear_time_limit，时间戳可能会被重置为 null
    pub timestamp: Option<u64>,
    /// 距离当前时间的秒数（如果 timestamp 存在）
    pub age: Option<u64>,
}

#[tauri::command]
pub async fn plugin_clipboard_get_metadata(
    app: AppHandle,
) -> Result<ClipboardMetadata, ClipboardError> {
    // 重要：这里使用插件层时间戳，而不是应用层时间戳
    // 
    // 原因：
    // 1. 应用层时间戳会被 auto_clear_time_limit 重置为 0
    // 2. 但系统剪贴板的内容可能还在
    // 3. 插件应该能获取真实的剪贴板状态和年龄
    // 4. 应用的清理机制不应该影响插件的数据获取
    //
    // 示例：
    // - 用户复制 "hello" 后 20 秒
    // - 应用的 auto_clear_time_limit 触发，应用层时间戳被重置
    // - 但系统剪贴板仍有 "hello"
    // - 插件调用此 API 应该看到：
    //   text: "hello", timestamp: 20秒前, age: 20
    // - 而不是：text: "hello", timestamp: null, age: null
    let timestamp = get_plugin_clipboard_timestamp();
    
    // 计算 age
    let age = timestamp.map(|ts| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(ts)
    });
    
    // 检测内容类型
    let mut content_type = ClipboardContentType::Empty;
    let mut text: Option<String> = None;
    let mut files: Option<Vec<ClipboardFile>> = None;
    
    // 先尝试读取文件路径（Windows）
    #[cfg(target_os = "windows")]
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
            Err(_) => {
                // 文本读取失败，可能是图片
            }
        }
    }
    
    // 检测是否有图片（简单检测）
    // 注意：这里只是尝试检测，实际的图片读取功能暂未实现
    #[cfg(target_os = "windows")]
    {
        use clipboard_rs::{Clipboard, ClipboardContext};
        
        if content_type == ClipboardContentType::Empty {
            if let Ok(ctx) = ClipboardContext::new() {
                // 尝试获取图片，如果成功说明有图片
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

/// 读取剪贴板内容（文本或文件路径）
#[tauri::command]
pub async fn get_clipboard_content(
    app: AppHandle,
) -> Result<ClipboardContent, ClipboardError> {
    // 获取剪贴板时间戳
    let timestamp = get_clipboard_timestamp();
    
    // 先尝试读取文件路径
    #[cfg(target_os = "windows")]
    {
        use clipboard_rs::{Clipboard, ClipboardContext};
        use std::path::Path;
        
        // 尝试读取文件列表
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