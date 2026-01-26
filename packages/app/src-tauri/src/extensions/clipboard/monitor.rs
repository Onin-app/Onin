use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};
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
                    // deduplicate
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

pub fn init(app: &AppHandle) {
    let history = ClipboardHistory::new();
    app.manage(history.clone());

    let app_handle = app.clone();

    std::thread::spawn(move || {
        let ctx = ClipboardContext::new().unwrap();
        let mut last_content_hash = String::new();

        loop {
            let mut new_item: Option<ClipboardItem> = None;
            let mut current_hash = String::new();

            match ctx.get_files() {
                Ok(files) if !files.is_empty() => {
                    let file_list = files.join("\n");
                    current_hash = format!("FILE:{}", file_list);
                    if current_hash != last_content_hash {
                        new_item = Some(ClipboardItem {
                            id: uuid::Uuid::new_v4().to_string(),
                            text: file_list,
                            timestamp: now_ts(),
                            item_type: "File".to_string(),
                            thumbnail: None,
                        });
                    }
                }
                _ => {
                    // Strategy: Use clipboard-rs high-level get_image() and save to temp file
                    // This avoids dealing with specific format names "PNG" vs "CF_DIB" and internal struct APIs.

                    match ctx.get_image() {
                        Ok(img) => {
                            // writeln!(file, "get_image() success. Saving to temp...").unwrap();

                            // Create a temp file path
                            let mut temp_path = env::temp_dir();
                            temp_path.push(format!("clipboard_temp_{}.png", uuid::Uuid::new_v4()));
                            let temp_path_str = temp_path.to_string_lossy().to_string();

                            // rust-clippy might complain about to_string_lossy, but it's safe here.
                            match img.save_to_path(&temp_path_str) {
                                Ok(_) => {
                                    // writeln!(file, "save_to_path success: {}", temp_path_str).unwrap();

                                    // Read back
                                    match fs::read(&temp_path) {
                                        Ok(bytes) => {
                                            // writeln!(file, "Read {} bytes", bytes.len()).unwrap();

                                            let base64_img =
                                                general_purpose::STANDARD.encode(&bytes);
                                            let thumbnail =
                                                format!("data:image/png;base64,{}", base64_img);
                                            let bytes_hash = format!("IMG:{}", thumbnail);

                                            if !last_content_hash.starts_with("IMG:")
                                                || bytes_hash != last_content_hash
                                            {
                                                current_hash = bytes_hash.clone();
                                                new_item = Some(ClipboardItem {
                                                    id: uuid::Uuid::new_v4().to_string(),
                                                    text: "Image copied".to_string(),
                                                    timestamp: now_ts(),
                                                    item_type: "Image".to_string(),
                                                    thumbnail: Some(thumbnail),
                                                });
                                            } else {
                                                current_hash = last_content_hash.clone();
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to read temp file: {}", e);
                                        }
                                    }
                                    // Cleanup
                                    let _ = fs::remove_file(temp_path);
                                }
                                Err(e) => {
                                    eprintln!("Failed to save clipboard image to temp file: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            // writeln!(file, "get_image() failed: {}", e).unwrap();
                            // Fallback to check_text
                            check_text(&ctx, &mut current_hash, &mut new_item, &last_content_hash);
                        }
                    }
                }
            }

            if !current_hash.is_empty() {
                if let Some(item) = new_item {
                    last_content_hash = current_hash;
                    history.push(item);
                    let _ = app_handle.emit("clipboard-update", ());
                } else if current_hash != last_content_hash {
                    last_content_hash = current_hash;
                }
            }

            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn check_text(
    ctx: &ClipboardContext,
    current_hash: &mut String,
    new_item: &mut Option<ClipboardItem>,
    last_hash: &str,
) {
    if let Ok(text) = ctx.get_text() {
        if !text.is_empty() {
            *current_hash = format!("TEXT:{}", text);
            if *current_hash != last_hash {
                *new_item = Some(ClipboardItem {
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

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
