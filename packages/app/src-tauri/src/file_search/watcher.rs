use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{atomic::Ordering, mpsc},
    time::Instant,
};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

use super::db::{count_index_db_for_roots, delete_index_path, open_index_db, save_index_batch};
use super::path_utils::{
    indexed_file_from_entry, indexed_file_from_path, should_ignore_path, should_skip_entry,
};
use super::types::{
    FileSearchOptions, FileSearchState, FILE_WATCH_DEBOUNCE, INDEX_WRITE_BATCH_SIZE, LIVE_SCAN_ID,
};

pub(super) fn start_file_watcher(app: AppHandle, options: FileSearchOptions, generation: usize) {
    let (event_tx, event_rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = match RecommendedWatcher::new(
        move |event| {
            let _ = event_tx.send(event);
        },
        Config::default(),
    ) {
        Ok(watcher) => watcher,
        Err(error) => {
            eprintln!("[file_search] Failed to create file watcher: {}", error);
            return;
        }
    };

    for root in &options.roots {
        if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
            eprintln!(
                "[file_search] Failed to watch file search root {:?}: {}",
                root, error
            );
        }
    }

    let mut pending_paths: HashSet<PathBuf> = HashSet::new();
    let mut last_event_at: Option<Instant> = None;

    loop {
        let state = app.state::<FileSearchState>();
        if state.watcher_generation.load(Ordering::Relaxed) != generation {
            break;
        }

        match event_rx.recv_timeout(FILE_WATCH_DEBOUNCE) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    pending_paths.insert(path);
                }
                last_event_at = Some(Instant::now());
            }
            Ok(Err(error)) => {
                eprintln!("[file_search] File watcher error: {}", error);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let Some(last_event) = last_event_at else {
                    continue;
                };

                if last_event.elapsed() < FILE_WATCH_DEBOUNCE || pending_paths.is_empty() {
                    continue;
                }

                let paths = pending_paths.drain().collect::<Vec<_>>();
                last_event_at = None;

                if let Err(error) = apply_file_watch_changes(&app, &options, paths) {
                    eprintln!(
                        "[file_search] Failed to apply file watch changes: {}",
                        error
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("[file_search] File watcher channel disconnected");
                break;
            }
        }
    }
}

fn apply_file_watch_changes(
    app: &AppHandle,
    options: &FileSearchOptions,
    paths: Vec<PathBuf>,
) -> Result<(), String> {
    let mut connection = open_index_db(app)?;

    for path in sorted_changed_paths(paths) {
        let Some(root) = root_for_path(&path, &options.roots) else {
            continue;
        };
        let root_string = root.to_string_lossy().to_string();

        if &path == root || should_ignore_path(&path, options) {
            continue;
        }

        if path.exists() {
            index_changed_path(&mut connection, &root_string, &path, options)?;
        } else {
            delete_index_path(&connection, &root_string, &path)?;
        }
    }

    if let Ok(count) = count_index_db_for_roots(&connection, &options.roots) {
        let state = app.state::<FileSearchState>();
        state.indexed_count.store(count, Ordering::Relaxed);
    }

    Ok(())
}

fn root_for_path<'a>(path: &Path, roots: &'a [PathBuf]) -> Option<&'a PathBuf> {
    roots
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
}

fn sorted_changed_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut normalized = paths
        .into_iter()
        .filter(|path| !path.as_os_str().is_empty())
        .collect::<Vec<_>>();
    normalized.sort_by_key(|path| path.components().count());
    normalized
}

fn index_changed_path(
    connection: &mut Connection,
    root: &str,
    path: &Path,
    options: &FileSearchOptions,
) -> Result<(), String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => {
            delete_index_path(connection, root, path)?;
            return Ok(());
        }
    };

    if metadata.is_dir() {
        let mut batch = Vec::with_capacity(INDEX_WRITE_BATCH_SIZE);
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !should_skip_entry(entry, options))
            .filter_map(Result::ok)
        {
            if let Some(file) = indexed_file_from_entry(&entry) {
                batch.push(file);
                if batch.len() >= INDEX_WRITE_BATCH_SIZE {
                    save_index_batch(connection, root, LIVE_SCAN_ID, &batch)?;
                    batch.clear();
                }
            }
        }

        if !batch.is_empty() {
            save_index_batch(connection, root, LIVE_SCAN_ID, &batch)?;
        }
    } else if let Some(file) = indexed_file_from_path(path, false) {
        save_index_batch(connection, root, LIVE_SCAN_ID, &[file])?;
    }

    Ok(())
}
