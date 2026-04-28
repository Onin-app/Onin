use std::{path::PathBuf, sync::Mutex};

use serde::Serialize;

pub(super) const DEFAULT_RESULT_LIMIT: usize = 30;

#[derive(Clone, Debug)]
pub(super) struct FileSearchOptions {
    pub(super) roots: Vec<PathBuf>,
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
}

#[derive(Default)]
pub struct FileSearchState {
    is_searching: Mutex<bool>,
    last_error: Mutex<Option<String>>,
}

impl FileSearchState {
    pub(super) fn is_searching(&self) -> bool {
        self.is_searching
            .lock()
            .map(|is_searching| *is_searching)
            .unwrap_or(false)
    }

    pub(super) fn set_searching(&self, searching: bool) {
        if let Ok(mut is_searching) = self.is_searching.lock() {
            *is_searching = searching;
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
    pub is_indexing: bool,
    pub indexed_count: usize,
    pub backend: String,
    pub available: bool,
    pub last_error: Option<String>,
}
