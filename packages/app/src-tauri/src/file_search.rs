use std::{
    env,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        RwLock,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use walkdir::{DirEntry, WalkDir};

use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

const MAX_INDEXED_ENTRIES: usize = 300_000;
const DEFAULT_RESULT_LIMIT: usize = 30;

#[derive(Clone, Debug)]
struct IndexedFile {
    name: String,
    path: String,
    parent: String,
    extension: String,
    is_dir: bool,
}

#[derive(Default)]
pub struct FileSearchState {
    index: RwLock<Vec<IndexedFile>>,
    is_indexing: AtomicBool,
    indexed_count: AtomicUsize,
}

#[derive(Serialize)]
pub struct FileSearchStatus {
    pub is_indexing: bool,
    pub indexed_count: usize,
}

pub fn init(app: AppHandle) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));

        let state = app.state::<FileSearchState>();
        if state.is_indexing.swap(true, Ordering::Relaxed) {
            return;
        }

        let root = match default_search_root() {
            Some(root) => root,
            None => {
                state.is_indexing.store(false, Ordering::Relaxed);
                return;
            }
        };

        let mut index = Vec::with_capacity(32_768);

        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !should_skip_entry(entry));

        for entry in walker.filter_map(Result::ok) {
            if index.len() >= MAX_INDEXED_ENTRIES {
                break;
            }

            if let Some(file) = indexed_file_from_entry(&entry) {
                index.push(file);
                let count = index.len();
                state.indexed_count.store(count, Ordering::Relaxed);

                if count % 1_000 == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }

        if let Ok(mut current_index) = state.index.write() {
            *current_index = index;
        }

        state.is_indexing.store(false, Ordering::Relaxed);
    });
}

#[tauri::command]
pub fn get_file_search_status(state: tauri::State<FileSearchState>) -> FileSearchStatus {
    FileSearchStatus {
        is_indexing: state.is_indexing.load(Ordering::Relaxed),
        indexed_count: state.indexed_count.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn search_indexed_files(
    query: String,
    limit: Option<usize>,
    state: tauri::State<FileSearchState>,
) -> Vec<LaunchableItem> {
    let query = query.trim().to_lowercase();
    if query.len() < 2 {
        return Vec::new();
    }

    let terms = parse_terms(&query);
    if terms.is_empty() {
        return Vec::new();
    }

    let limit = limit.unwrap_or(DEFAULT_RESULT_LIMIT).clamp(1, 100);
    let index = match state.index.read() {
        Ok(index) => index,
        Err(_) => return Vec::new(),
    };

    let mut top_results: Vec<(i32, &IndexedFile)> = Vec::with_capacity(limit);

    for file in index.iter() {
        let Some(score) = score_file(file, &terms) else {
            continue;
        };

        if top_results.len() < limit {
            top_results.push((score, file));
            continue;
        }

        if let Some((worst_index, _)) = top_results
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| compare_scored_files(a, b))
        {
            if compare_scored_files(&(score, file), &top_results[worst_index]).is_gt() {
                top_results[worst_index] = (score, file);
            }
        }
    }

    top_results.sort_by(|a, b| compare_scored_files(b, a));

    top_results
        .into_iter()
        .map(|(_, file)| launchable_item_from_file(file))
        .collect()
}

#[tauri::command]
pub fn open_indexed_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Failed to open file {}: {}", path, e))
}

fn default_search_root() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("USERPROFILE").map(PathBuf::from)
    }

    #[cfg(not(target_os = "windows"))]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
}

fn indexed_file_from_entry(entry: &DirEntry) -> Option<IndexedFile> {
    let path = entry.path();
    let name = path.file_name()?.to_string_lossy().to_string();
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    Some(IndexedFile {
        name,
        path: path.to_string_lossy().to_string(),
        parent,
        extension,
        is_dir: entry.file_type().is_dir(),
    })
}

fn should_skip_entry(entry: &DirEntry) -> bool {
    let path = entry.path();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if entry.depth() == 0 {
        return false;
    }

    if entry.file_type().is_dir() {
        return matches!(
            name.as_str(),
            ".git"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".cache"
                | "cache"
                | "caches"
                | "temp"
                | "tmp"
        ) || is_platform_cache_dir(path);
    }

    false
}

fn is_platform_cache_dir(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    normalized.contains("/appdata/local/temp") || normalized.contains("/library/caches")
}

fn parse_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn score_file(file: &IndexedFile, terms: &[String]) -> Option<i32> {
    let name = file.name.to_lowercase();
    let parent = file.parent.to_lowercase();
    let path = file.path.to_lowercase();
    let mut score = if file.is_dir { 10 } else { 0 };

    for term in terms {
        if term.starts_with('.') {
            if file.extension == *term {
                score += 30;
                continue;
            }
            return None;
        }

        if name == *term {
            score += 100;
        } else if name.starts_with(term) {
            score += 75;
        } else if name.contains(term) {
            score += 45;
        } else if parent.contains(term) {
            score += 15;
        } else if path.contains(term) {
            score += 5;
        } else {
            return None;
        }
    }

    Some(score)
}

fn compare_scored_files(
    (score_a, file_a): &(i32, &IndexedFile),
    (score_b, file_b): &(i32, &IndexedFile),
) -> std::cmp::Ordering {
    score_a
        .cmp(score_b)
        .then_with(|| file_a.is_dir.cmp(&file_b.is_dir))
        .then_with(|| file_b.name.len().cmp(&file_a.name.len()))
}

fn launchable_item_from_file(file: &IndexedFile) -> LaunchableItem {
    LaunchableItem {
        name: file.name.clone(),
        description: Some(file.parent.clone()),
        keywords: vec![CommandKeyword {
            name: file.name.clone(),
            disabled: None,
            is_default: Some(true),
        }],
        path: file.path.clone(),
        icon: if file.is_dir {
            "folder".to_string()
        } else {
            "file".to_string()
        },
        icon_type: IconType::Iconfont,
        item_type: if file.is_dir {
            ItemType::Folder
        } else {
            ItemType::File
        },
        source: ItemSource::FileSearch,
        action: None,
        origin: None,
        source_display: Some("File".to_string()),
        matches: None,
        requires_confirmation: false,
    }
}
