use crate::extensions::clipboard::types::ClipboardItem;
use base64::{engine::general_purpose, Engine as _};
use std::collections::VecDeque;
use std::fs;
use tauri::{AppHandle, Manager};

fn get_storage_dir(app: &AppHandle) -> std::path::PathBuf {
    let app_data_dir = app.path().app_data_dir().unwrap();
    app_data_dir.join("extensions").join("clipboard")
}

pub fn load_history(app: &AppHandle) -> VecDeque<ClipboardItem> {
    let storage_dir = get_storage_dir(app);
    let history_file = storage_dir.join("history.json");

    if !history_file.exists() {
        return VecDeque::new();
    }

    match fs::read_to_string(&history_file) {
        Ok(content) => {
            let mut items: VecDeque<ClipboardItem> =
                serde_json::from_str(&content).unwrap_or_default();

            // Post-process items: verify image files and load thumbnails if needed
            // For performance, we might want to lazy load, but for now let's load them back
            // so the monitor has the full state as expected by current frontend.
            for item in items.iter_mut() {
                if item.item_type == "Image" {
                    if let Some(path_str) = &item.image_path {
                        let image_path = storage_dir.join("images").join(path_str);
                        if image_path.exists() {
                            if let Ok(bytes) = fs::read(image_path) {
                                let base64_img = general_purpose::STANDARD.encode(&bytes);
                                item.thumbnail =
                                    Some(format!("data:image/png;base64,{}", base64_img));
                            }
                        }
                    }
                }
            }
            items
        }
        Err(e) => {
            eprintln!("Failed to read clipboard history: {}", e);
            VecDeque::new()
        }
    }
}

pub fn save_history(app: &AppHandle, items: &VecDeque<ClipboardItem>) {
    let storage_dir = get_storage_dir(app);
    let images_dir = storage_dir.join("images");
    let history_file = storage_dir.join("history.json");

    if !storage_dir.exists() {
        let _ = fs::create_dir_all(&images_dir);
    }
    if !images_dir.exists() {
        let _ = fs::create_dir_all(&images_dir);
    }

    // Prepare items for saving (separate images)
    let mut saved_items = Vec::new();
    let mut active_image_paths = Vec::new();

    for item in items {
        let mut item_to_save = item.clone();

        if item.item_type == "Image" {
            // Check if we need to save the image to disk
            // If it has a thumbnail (Base64) but no image_path, save it.
            if item_to_save.image_path.is_none() {
                if let Some(thumbnail) = &item.thumbnail {
                    // format: "data:image/png;base64,..."
                    if let Some(base64_data) = thumbnail.split(',').nth(1) {
                        if let Ok(bytes) = general_purpose::STANDARD.decode(base64_data) {
                            let filename = format!("{}.png", item.id);
                            let file_path = images_dir.join(&filename);
                            if let Ok(_) = fs::write(&file_path, bytes) {
                                item_to_save.image_path = Some(filename.clone());
                                // Clear thumbnail from JSON to save space
                                item_to_save.thumbnail = None;
                            }
                        }
                    }
                }
            } else {
                // If already has path, ensure we don't save the huge thumbnail in JSON
                item_to_save.thumbnail = None;
            }

            if let Some(path) = &item_to_save.image_path {
                active_image_paths.push(path.clone());
            }
        }

        saved_items.push(item_to_save);
    }

    // Write JSON using fs::write directly which is simpler than OpenOptions
    // Use serde_json::to_string for formatting
    match serde_json::to_string(&saved_items) {
        Ok(json) => {
            if let Err(e) = fs::write(&history_file, json) {
                eprintln!("Failed to write clipboard history: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to serialize clipboard history: {}", e),
    }

    // Cleanup old images
    // List all files in images_dir, delete those not in active_image_paths
    if let Ok(entries) = fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if !active_image_paths.contains(&file_name) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
}

