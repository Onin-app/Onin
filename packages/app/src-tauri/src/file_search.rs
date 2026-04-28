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

#[tauri::command]
pub fn get_file_search_status(
    _app: AppHandle,
    state: tauri::State<FileSearchState>,
) -> FileSearchStatus {
    let everything_installed = provider::everything_installed();
    let everything_ipc_available = provider::everything_ipc_available();

    FileSearchStatus {
        is_indexing: state.is_searching(),
        indexed_count: 0,
        backend: provider::backend_name().to_string(),
        everything_installed,
        everything_ipc_available,
        everything_install_required: provider::everything_install_required(),
        available: provider::backend_available(),
        last_error: state.last_error(),
    }
}

#[tauri::command]
pub async fn install_file_search_everything() -> Result<(), String> {
    tokio::task::spawn_blocking(provider::install_everything_backend)
        .await
        .unwrap_or_else(|error| Err(error.to_string()))
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
