use crate::icon_utils;
use crate::shared_types::{CommandKeyword, ItemSource, ItemType, LaunchableItem};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tokio::sync::Mutex; // Use the async-aware Mutex from Tokio

const FILE_COMMANDS_FILENAME: &str = "file_commands.json";

#[derive(Debug)]
pub struct FileCommandManager {
    // Change to an Option to support lazy loading.
    pub file_commands: Mutex<Option<Vec<LaunchableItem>>>,
    app_handle: tauri::AppHandle,
}

impl FileCommandManager {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // Initialize with None. Data will be loaded on first access.
        Self {
            file_commands: Mutex::new(None),
            app_handle,
        }
    }

    fn get_storage_path(&self) -> PathBuf {
        self.app_handle
            .path()
            .app_data_dir()
            .unwrap()
            .join(FILE_COMMANDS_FILENAME)
    }

    /// Ensures the file commands are loaded from disk, only performs load on first call.
    async fn ensure_loaded(&self) {
        let mut guard = self.file_commands.lock().await;
        if guard.is_none() {
            let storage_path = self.get_storage_path();
            let items = if storage_path.exists() {
                match tokio::fs::read_to_string(storage_path).await {
                    Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                    Err(_) => vec![], // File exists but is unreadable
                }
            } else {
                vec![] // No file, start with empty list
            };
            *guard = Some(items);
        }
    }

    /// Helper function to write a slice of items to the JSON file.
    /// This function does not lock the state itself, preventing deadlocks.
    async fn write_items_to_disk(&self, items: &[LaunchableItem]) {
        let storage_path = self.get_storage_path();
        match serde_json::to_string_pretty(items) {
            Ok(content) => {
                if let Some(parent) = storage_path.parent() {
                    // Create parent directory if it doesn't exist.
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                if let Err(e) = tokio::fs::write(storage_path, content).await {
                    tracing::error!("Failed to write file commands to disk: {:?}", e);
                }
            }
            Err(e) => tracing::error!("Failed to serialize file commands: {:?}", e),
        };
    }

    pub async fn get_items(&self) -> Vec<LaunchableItem> {
        self.ensure_loaded().await;
        // We can unwrap safely because ensure_loaded guarantees it's Some.
        self.file_commands.lock().await.as_ref().unwrap().clone()
    }

    pub async fn add_paths(&self, paths: Vec<String>) -> Vec<LaunchableItem> {
        self.ensure_loaded().await;

        // The logic is enclosed in a block to ensure the lock is released before `await`.
        let updated_items = {
            let mut items_guard = self.file_commands.lock().await;
            // We can unwrap safely because ensure_loaded guarantees it's Some.
            let items = items_guard.as_mut().unwrap();

            let mut existing_paths: HashSet<String> =
                items.iter().map(|item| item.path.clone()).collect();

            for path_str in paths {
                let path = Path::new(&path_str);
                if path.exists() && !existing_paths.contains(&path_str) {
                    if let Some(item) = self.create_item_from_path(path).await {
                        existing_paths.insert(item.path.clone());
                        items.push(item);
                    }
                }
            }

            items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            items.clone()
        }; // `items_guard` is dropped here, releasing the lock.

        // Now that the lock is released, we can safely perform async I/O.
        self.write_items_to_disk(&updated_items).await;
        self.delete_commands_json();

        updated_items
    }

    pub async fn remove_item(&self, path_to_remove: &str) -> Vec<LaunchableItem> {
        self.ensure_loaded().await;

        let updated_items = {
            let mut items_guard = self.file_commands.lock().await;
            let items = items_guard.as_mut().unwrap();
            // 根据路径过滤掉要删除的项
            items.retain(|item| item.path != path_to_remove);
            items.clone()
        }; // `items_guard` is dropped here.

        self.write_items_to_disk(&updated_items).await;
        self.delete_commands_json();

        updated_items
    }

    async fn create_item_from_path(&self, path: &Path) -> Option<LaunchableItem> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let path_str = path.to_string_lossy().into_owned();

        let (item_type, icon) = if path.is_dir() {
            (ItemType::Folder, "".to_string()) // TODO: Generic folder icon
        } else if path.is_file() {
            let is_exe = path.extension().map_or(false, |e| e == "exe");
            let item_type = if is_exe {
                ItemType::App
            } else {
                ItemType::File
            };
            // Because icon extraction can be a blocking I/O operation,
            // we run it in a blocking task to avoid stalling the async runtime.
            let path_for_icon = path_str.clone();
            let icon_str = match tokio::task::spawn_blocking(move || {
                icon_utils::extract_icon_from_path(&path_for_icon)
            })
            .await
            {
                Ok(Some(icon)) => icon,    // Task completed and returned an icon
                Ok(None) => String::new(), // Task completed and returned no icon
                Err(e) => {
                    tracing::error!("Icon extraction task failed: {:?}", e);
                    String::new() // Task failed to execute (e.g., panicked)
                }
            };

            (item_type, icon_str)
        } else {
            return None; // Path is not a file or directory (e.g., a symlink to nowhere)
        };

        Some(LaunchableItem {
            name: name.clone(),
            keywords: vec![CommandKeyword {
                name,
                disabled: None,
                is_default: Some(true),
            }],
            path: path_str,
            icon,
            icon_type: crate::shared_types::IconType::Base64,
            item_type,
            source: ItemSource::FileCommand,
            action: None,
            origin: None,
        })
    }
    fn delete_commands_json(&self) {
        let path = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap()
            .join("commands.json");
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                tracing::error!("Failed to delete commands.json: {:?}", e);
            }
        }
    }
}

#[tauri::command]
pub async fn get_file_commands(
    manager: tauri::State<'_, FileCommandManager>,
) -> Result<Vec<LaunchableItem>, String> {
    Ok(manager.get_items().await)
}

#[tauri::command]
pub async fn add_file_commands(
    paths: Vec<String>,
    manager: tauri::State<'_, FileCommandManager>,
) -> Result<Vec<LaunchableItem>, String> {
    Ok(manager.add_paths(paths).await)
}

#[tauri::command]
pub async fn remove_file_command(
    path: String,
    manager: tauri::State<'_, FileCommandManager>,
) -> Result<Vec<LaunchableItem>, String> {
    Ok(manager.remove_item(&path).await)
}
