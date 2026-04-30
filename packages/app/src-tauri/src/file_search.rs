mod path_utils;
mod provider;
#[cfg(test)]
mod tests;
mod types;

use std::{
    fs::File,
    io::{Read, Take},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

pub use types::FileSearchState;
use types::{FileSearchResponse, FileSearchStatus};

const TEXT_PREVIEW_MAX_BYTES: u64 = 512 * 1024;

#[derive(Serialize)]
pub struct FileTextPreview {
    pub content: String,
    pub truncated: bool,
    pub bytes_read: usize,
    pub total_bytes: u64,
}

pub fn init(_app: AppHandle) {
    // File search delegates to the platform search backend on demand.
}

#[tauri::command]
pub fn get_file_search_status(
    _app: AppHandle,
    state: tauri::State<FileSearchState>,
) -> FileSearchStatus {
    let everything_installed = provider::everything_installed();
    let everything_ipc_available = provider::everything_ipc_available();

    FileSearchStatus {
        is_searching: state.is_searching(),
        last_result_count: state.last_result_count(),
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
pub async fn search_files(
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
    app: AppHandle,
) -> FileSearchResponse {
    let state = app.state::<FileSearchState>();
    state.begin_search();

    let app_for_search = app.clone();
    let result = tokio::task::spawn_blocking(move || {
        provider::search_platform_files(query, limit, offset, &app_for_search)
    })
    .await
    .unwrap_or_else(|error| Err(error.to_string()));

    state.end_search();

    match result {
        Ok(items) => {
            state.set_last_result_count(items.total_count);
            state.set_last_error(None);
            items
        }
        Err(error) => {
            state.set_last_result_count(0);
            state.set_last_error(Some(error));
            FileSearchResponse {
                items: Vec::new(),
                total_count: 0,
                total_count_is_exact: true,
                has_more: false,
                offset: offset.unwrap_or(0),
                limit: limit.unwrap_or(types::DEFAULT_RESULT_LIMIT),
            }
        }
    }
}

#[tauri::command]
pub fn open_file_search_result(path: String, app: AppHandle) -> Result<(), String> {
    let path = path_utils::validate_search_result_path(&app, &path)?;
    opener::open(&path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))
}

#[tauri::command]
pub async fn read_file_text_preview(
    path: String,
    app: AppHandle,
) -> Result<FileTextPreview, String> {
    tokio::task::spawn_blocking(move || {
        let path = path_utils::validate_search_result_path(&app, &path)?;
        let metadata =
            std::fs::metadata(&path).map_err(|error| format!("无法读取文件信息: {}", error))?;

        if !metadata.is_file() {
            return Err("只能预览文本文件".to_string());
        }

        let total_bytes = metadata.len();
        let mut buffer = Vec::new();
        let file = File::open(&path).map_err(|error| format!("无法打开文件: {}", error))?;
        let mut reader: Take<File> = file.take(TEXT_PREVIEW_MAX_BYTES + 1);
        reader
            .read_to_end(&mut buffer)
            .map_err(|error| format!("读取文件失败: {}", error))?;

        let truncated = buffer.len() as u64 > TEXT_PREVIEW_MAX_BYTES;
        if truncated {
            buffer.truncate(TEXT_PREVIEW_MAX_BYTES as usize);
        }

        let content = match String::from_utf8(buffer) {
            Ok(content) => content,
            Err(error) if truncated && error.utf8_error().error_len().is_none() => {
                let valid_up_to = error.utf8_error().valid_up_to();
                let mut buffer = error.into_bytes();
                buffer.truncate(valid_up_to);
                String::from_utf8(buffer)
                    .map_err(|_| "文件不是 UTF-8 文本，无法预览".to_string())?
            }
            Err(_) => return Err("文件不是 UTF-8 文本，无法预览".to_string()),
        }
        .trim_start_matches('\u{feff}')
        .to_string();

        Ok(FileTextPreview {
            bytes_read: content.len(),
            content,
            truncated,
            total_bytes,
        })
    })
    .await
    .unwrap_or_else(|error| Err(error.to_string()))
}
