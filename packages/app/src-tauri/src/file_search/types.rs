use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize},
    time::Duration,
};

use serde::Serialize;

pub(super) const INDEX_WRITE_BATCH_SIZE: usize = 2_000;
pub(super) const DEFAULT_RESULT_LIMIT: usize = 30;
pub(super) const SEARCH_CANDIDATE_LIMIT: usize = 1_000;
pub(super) const INDEX_DB_FILE_NAME: &str = "file_search.sqlite";
pub(super) const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(2);
pub(super) const FILE_WATCH_DEBOUNCE: Duration = Duration::from_millis(800);
pub(super) const LIVE_SCAN_ID: &str = "live";

#[derive(Clone, Debug)]
pub(super) struct IndexedFile {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) parent: String,
    pub(super) extension: String,
    pub(super) is_dir: bool,
}

#[derive(Clone, Debug)]
pub(super) struct FileSearchOptions {
    pub(super) roots: Vec<PathBuf>,
    pub(super) excluded_paths: Vec<PathBuf>,
    pub(super) include_hidden: bool,
}

#[derive(Default)]
pub struct FileSearchState {
    pub(super) is_indexing: AtomicBool,
    pub(super) indexed_count: AtomicUsize,
    pub(super) watcher_generation: AtomicUsize,
    pub(super) rebuild_requested: AtomicBool,
}

#[derive(Serialize)]
pub struct FileSearchStatus {
    pub is_indexing: bool,
    pub indexed_count: usize,
}
