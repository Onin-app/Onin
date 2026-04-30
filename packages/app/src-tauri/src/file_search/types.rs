use std::{path::PathBuf, sync::Mutex};

use serde::Serialize;

use crate::shared_types::LaunchableItem;

pub(super) const DEFAULT_RESULT_LIMIT: usize = 30;

#[derive(Clone, Debug)]
pub(super) struct FileSearchOptions {
    pub(super) roots: Vec<PathBuf>,
    pub(super) preferred_paths: Vec<PathBuf>,
    pub(super) excluded_paths: Vec<PathBuf>,
    pub(super) include_hidden: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PlatformFile {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) parent: String,
    pub(super) extension: String,
    pub(super) is_dir: bool,
    pub(super) modified_time: Option<u64>,
}

#[derive(Serialize)]
pub struct FileSearchResponse {
    pub items: Vec<LaunchableItem>,
    pub total_count: usize,
    pub total_count_is_exact: bool,
    pub has_more: bool,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Default)]
pub struct FileSearchState {
    active_search_count: Mutex<usize>,
    last_result_count: Mutex<usize>,
    last_error: Mutex<Option<String>>,
}

impl FileSearchState {
    pub(super) fn is_searching(&self) -> bool {
        self.active_search_count
            .lock()
            .map(|active_search_count| *active_search_count > 0)
            .unwrap_or(false)
    }

    pub(super) fn begin_search(&self) {
        if let Ok(mut active_search_count) = self.active_search_count.lock() {
            *active_search_count = active_search_count.saturating_add(1);
        }
    }

    pub(super) fn end_search(&self) {
        if let Ok(mut active_search_count) = self.active_search_count.lock() {
            *active_search_count = active_search_count.saturating_sub(1);
        }
    }

    pub(super) fn last_result_count(&self) -> usize {
        self.last_result_count
            .lock()
            .map(|last_result_count| *last_result_count)
            .unwrap_or(0)
    }

    pub(super) fn set_last_result_count(&self, count: usize) {
        if let Ok(mut last_result_count) = self.last_result_count.lock() {
            *last_result_count = count;
        }
    }

    pub(super) fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .ok()
            .and_then(|last_error| last_error.clone())
    }

    pub(super) fn set_last_error(&self, error: Option<String>) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = error;
        }
    }
}

#[derive(Serialize)]
pub struct FileSearchStatus {
    pub is_searching: bool,
    pub last_result_count: usize,
    pub backend: String,
    pub everything_installed: bool,
    pub everything_ipc_available: bool,
    pub everything_install_required: bool,
    pub available: bool,
    pub last_error: Option<String>,
}
