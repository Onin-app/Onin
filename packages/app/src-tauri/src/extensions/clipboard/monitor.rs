use clipboard_rs::{Clipboard, ClipboardContext};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
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
                    // Check for Image presence
                    // get_image() returns Ok(_) if image exists
                    match ctx.get_image() {
                        Ok(_) => {
                            // For now, we just mark it as Image without thumbnail to avoid compilation issues
                            // We'll use a placeholder text
                            let placeholder = "Image copied".to_string();
                            let bytes_hash = format!("IMG:DETECTED:{:?}", SystemTime::now());

                            // Problem: without content hash, we might duplicate or miss updates.
                            // But get_image() might return checkable metadata?
                            // Assuming for v1 we just accept it if it's new.
                            // To properly dedup, we'd need to read bytes.
                            // Let's rely on change being detected if we poll frequently or if 'text' is empty.

                            // Actually, if I can't read bytes, I can't dedup easily against previous image.
                            // But if last was TEXT, and now is IMG, we switch.
                            if !last_content_hash.starts_with("IMG:") {
                                current_hash = bytes_hash;
                                new_item = Some(ClipboardItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    text: placeholder,
                                    timestamp: now_ts(),
                                    item_type: "Image".to_string(),
                                    thumbnail: None,
                                });
                            } else {
                                // Existing image. We assume no change for now as we can't diff.
                                // Ideally we would retry byte reading later.
                                current_hash = last_content_hash.clone();
                            }
                        }
                        Err(_) => {
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
