mod path_utils;
mod provider;
#[cfg(test)]
mod tests;
mod types;

use tauri::{AppHandle, Manager};

use crate::shared_types::LaunchableItem;

pub use types::FileSearchState;
use types::FileSearchStatus;

pub fn init(_app: AppHandle) {
    // File search now delegates to the platform search index on demand.
}

pub fn rebuild_file_search_index_after_config_change(_app: AppHandle) {
    // There is no local index to rebuild. Platform index maintenance is owned by the OS.
}

#[tauri::command]
pub fn get_file_search_status(
    _app: AppHandle,
    state: tauri::State<FileSearchState>,
) -> FileSearchStatus {
    FileSearchStatus {
        is_indexing: state.is_searching(),
        indexed_count: 0,
        backend: provider::backend_name().to_string(),
        available: provider::backend_available(),
        last_error: state.last_error(),
    }
}

#[tauri::command]
pub fn rebuild_file_search_index(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn search_indexed_files(
    query: String,
    limit: Option<usize>,
    app: AppHandle,
) -> Vec<LaunchableItem> {
    let state = app.state::<FileSearchState>();
    state.set_searching(true);

    let app_for_search = app.clone();
    let result = tokio::task::spawn_blocking(move || {
        provider::search_platform_files(query, limit, &app_for_search)
    })
    .await
    .unwrap_or_else(|error| Err(error.to_string()));

    state.set_searching(false);

    match result {
        Ok(items) => {
            state.set_last_error(None);
            items
        }
        Err(error) => {
            state.set_last_error(Some(error));
            Vec::new()
        }
    }
}

#[tauri::command]
pub fn open_indexed_file(path: String, app: AppHandle) -> Result<(), String> {
    let path = path_utils::validate_search_result_path(&app, &path)?;
    opener::open(&path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))
}
