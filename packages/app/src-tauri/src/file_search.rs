mod db;
mod indexer;
mod path_utils;
mod search;
#[cfg(test)]
mod tests;
mod types;
mod utils;
mod watcher;

use std::sync::atomic::Ordering;

use tauri::AppHandle;

use crate::shared_types::LaunchableItem;

pub use indexer::{init, rebuild_file_search_index_after_config_change};
pub use types::FileSearchState;
use types::FileSearchStatus;

#[tauri::command]
pub fn get_file_search_status(state: tauri::State<FileSearchState>) -> FileSearchStatus {
    FileSearchStatus {
        is_indexing: state.is_indexing.load(Ordering::Relaxed),
        indexed_count: state.indexed_count.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn rebuild_file_search_index(app: AppHandle) -> Result<(), String> {
    rebuild_file_search_index_after_config_change(app);
    Ok(())
}

#[tauri::command]
pub async fn search_indexed_files(
    query: String,
    limit: Option<usize>,
    app: AppHandle,
) -> Vec<LaunchableItem> {
    tokio::task::spawn_blocking(move || search::search_indexed_files_blocking(query, limit, app))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn open_indexed_file(path: String, app: AppHandle) -> Result<(), String> {
    let path = path_utils::validate_indexed_file_path(&app, &path)?;
    opener::open(&path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))
}
