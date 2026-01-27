use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher,
    ClipboardWatcherContext,
};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const MAX_HISTORY_SIZE: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub item_type: String,
    pub thumbnail: Option<String>,
}

#[derive(Clone)]
pub struct ClipboardHistory {
    items: Arc<Mutex<VecDeque<ClipboardItem>>>,
}

impl ClipboardHistory {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, item: ClipboardItem) {
        let mut items = self.items.lock().unwrap();

        if let Some(front) = items.front() {
            if front.item_type == item.item_type && front.text == item.text {
                if front.item_type == "Image" {
                    // 去重
                } else {
                    return;
                }
            }
        }

        items.push_front(item);
        if items.len() > MAX_HISTORY_SIZE {
            items.pop_back();
        }
    }

    pub fn get_all(&self) -> Vec<ClipboardItem> {
        let items = self.items.lock().unwrap();
        items.iter().cloned().collect()
    }
}

pub struct ClipboardMonitor {
    ctx: ClipboardContext,
    app_handle: AppHandle,
    last_content_hash: String,
    history: ClipboardHistory,
}

impl ClipboardMonitor {
    pub fn new(app: AppHandle, history: ClipboardHistory) -> Self {
        Self {
            ctx: ClipboardContext::new().unwrap(),
            app_handle: app,
            last_content_hash: String::new(),
            history,
        }
    }

    fn process_image(&mut self, current_hash: &mut String, new_item: &mut Option<ClipboardItem>) {
        if let Ok(img) = self.ctx.get_image() {
            // 创建临时文件路径
            let mut temp_path = env::temp_dir();
            temp_path.push(format!("clipboard_temp_{}.png", uuid::Uuid::new_v4()));
            let temp_path_str = temp_path.to_string_lossy().to_string();

            match img.save_to_path(&temp_path_str) {
                Ok(_) => {
                    if let Ok(bytes) = fs::read(&temp_path) {
                        let base64_img = general_purpose::STANDARD.encode(&bytes);
                        let thumbnail = format!("data:image/png;base64,{}", base64_img);
                        let bytes_hash = format!("IMG:{}", thumbnail);

                        if !self.last_content_hash.starts_with("IMG:")
                            || bytes_hash != self.last_content_hash
                        {
                            *current_hash = bytes_hash.clone();
                            *new_item = Some(ClipboardItem {
                                id: uuid::Uuid::new_v4().to_string(),
                                text: "Image copied".to_string(),
                                timestamp: now_ts(),
                                item_type: "Image".to_string(),
                                thumbnail: Some(thumbnail),
                            });
                        } else {
                            *current_hash = self.last_content_hash.clone();
                        }
                    }
                    // 清理
                    let _ = fs::remove_file(temp_path);
                }
                Err(e) => {
                    eprintln!("Failed to save clipboard image to temp file: {}", e);
                }
            }
        }
    }
}

impl ClipboardHandler for ClipboardMonitor {
    fn on_clipboard_change(&mut self) {
        let mut new_item: Option<ClipboardItem> = None;
        let mut current_hash = String::new();

        // 1. 检查文件
        let mut files_found = false;
        if let Ok(files) = self.ctx.get_files() {
            if !files.is_empty() {
                files_found = true;
                let file_list = files.join("\n");
                current_hash = format!("FILE:{}", file_list);
                if current_hash != self.last_content_hash {
                    new_item = Some(ClipboardItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        text: file_list,
                        timestamp: now_ts(),
                        item_type: "File".to_string(),
                        thumbnail: None,
                    });
                }
            }
        }

        // 2. 检查图片（仅当没有文件时）
        if !files_found {
            // 检查是否有图片
            // 我们使用辅助函数保持代码整洁，因为逻辑比较复杂
            self.process_image(&mut current_hash, &mut new_item);
        }

        // 3. 回退到文本（如果没有检测到新的图片或文件，
        // 或者根本没有发现任何图片/文件内容）
        // 注意：之前的逻辑是：如果检测到文件，则使用文件。否则 get_image。
        // 如果 get_image 失败，检查文本。
        // 这里我们要按顺序检查。

        if new_item.is_none() && current_hash.is_empty() {
            if let Ok(text) = self.ctx.get_text() {
                if !text.is_empty() {
                    current_hash = format!("TEXT:{}", text);
                    if current_hash != self.last_content_hash {
                        new_item = Some(ClipboardItem {
                            id: uuid::Uuid::new_v4().to_string(),
                            text,
                            timestamp: now_ts(),
                            item_type: "Text".to_string(),
                            thumbnail: None,
                        });
                    }
                }
            }
        }

        if !current_hash.is_empty() {
            if let Some(item) = new_item {
                self.last_content_hash = current_hash;
                self.history.push(item);
                let _ = self.app_handle.emit("clipboard-update", ());
            } else if current_hash != self.last_content_hash {
                self.last_content_hash = current_hash;
            }
        }
    }
}

pub fn init(app: &AppHandle) {
    let history = ClipboardHistory::new();
    app.manage(history.clone());

    let app_for_thread = app.clone();
    let history_for_thread = history.clone();

    std::thread::spawn(move || {
        let monitor = ClipboardMonitor::new(app_for_thread, history_for_thread);
        match ClipboardWatcherContext::new() {
            Ok(mut watcher) => {
                let _shutdown = watcher.add_handler(monitor).get_shutdown_channel();
                watcher.start_watch();
            }
            Err(e) => {
                eprintln!("Failed to initialize clipboard watcher: {}", e);
            }
        }
    });
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
