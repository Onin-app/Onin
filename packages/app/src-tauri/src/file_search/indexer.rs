use std::{sync::atomic::Ordering, thread, time::Duration};

use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

use super::db::{
    clear_index_db, count_index_db_for_roots, finalize_index_scan, open_index_db_for_indexing,
    save_index_batch,
};
use super::path_utils::{file_search_options, indexed_file_from_entry, should_skip_entry};
use super::types::{FileSearchState, INDEX_WRITE_BATCH_SIZE};
use super::utils::new_scan_id;
use super::watcher::start_file_watcher;

pub fn init(app: AppHandle) {
    start_indexing(app, false, Duration::from_secs(3));
}

fn start_indexing(app: AppHandle, reset_index: bool, delay: Duration) {
    thread::spawn(move || {
        if !delay.is_zero() {
            thread::sleep(delay);
        }

        let state = app.state::<FileSearchState>();
        if state.is_indexing.swap(true, Ordering::Relaxed) {
            if reset_index {
                state.rebuild_requested.store(true, Ordering::Relaxed);
                state.watcher_generation.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
        state.rebuild_requested.store(false, Ordering::Relaxed);

        let options = file_search_options(&app);
        let generation = state.watcher_generation.fetch_add(1, Ordering::Relaxed) + 1;

        if options.roots.is_empty() {
            state.indexed_count.store(0, Ordering::Relaxed);
            state.is_indexing.store(false, Ordering::Relaxed);
            return;
        }
        state.indexed_count.store(0, Ordering::Relaxed);

        let mut db_connection = match open_index_db_for_indexing(&app, reset_index) {
            Ok(connection) => {
                if reset_index {
                    if let Err(error) = clear_index_db(&connection) {
                        eprintln!("[file_search] Failed to clear SQLite index: {}", error);
                    }
                }
                Some(connection)
            }
            Err(error) => {
                eprintln!("[file_search] Failed to open SQLite index: {}", error);
                None
            }
        };

        let scan_id = new_scan_id();
        let mut scanned_count = 0usize;
        let mut cancelled = false;

        for root in &options.roots {
            if state.watcher_generation.load(Ordering::Relaxed) != generation {
                cancelled = true;
                break;
            }

            let root_string = root.to_string_lossy().to_string();
            let mut batch = Vec::with_capacity(INDEX_WRITE_BATCH_SIZE);
            let mut root_scanned_count = 0usize;

            let walker = WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| !should_skip_entry(entry, &options));

            for entry in walker.filter_map(Result::ok) {
                if state.watcher_generation.load(Ordering::Relaxed) != generation {
                    cancelled = true;
                    break;
                }

                if let Some(file) = indexed_file_from_entry(&entry) {
                    batch.push(file);
                    scanned_count += 1;
                    root_scanned_count += 1;
                    state.indexed_count.store(scanned_count, Ordering::Relaxed);

                    if batch.len() >= INDEX_WRITE_BATCH_SIZE {
                        if let Some(connection) = db_connection.as_mut() {
                            if let Err(error) =
                                save_index_batch(connection, &root_string, &scan_id, &batch)
                            {
                                eprintln!(
                                    "[file_search] Failed to persist SQLite batch: {}",
                                    error
                                );
                                db_connection = None;
                            }
                        }
                        batch.clear();
                    }

                    if root_scanned_count % 1_000 == 0 {
                        thread::sleep(Duration::from_millis(5));
                    }
                }
            }

            if cancelled {
                break;
            }

            if let Some(connection) = db_connection.as_mut() {
                if !batch.is_empty() {
                    if let Err(error) = save_index_batch(connection, &root_string, &scan_id, &batch)
                    {
                        eprintln!(
                            "[file_search] Failed to persist final SQLite batch: {}",
                            error
                        );
                        db_connection = None;
                    }
                }
            }

            if let Some(connection) = db_connection.as_mut() {
                if let Err(error) = finalize_index_scan(connection, &root_string, &scan_id) {
                    eprintln!(
                        "[file_search] Failed to finalize SQLite index scan: {}",
                        error
                    );
                }
            }
        }

        if cancelled {
            eprintln!("[file_search] Index scan cancelled because a rebuild was requested");
        } else if let Some(connection) = db_connection.as_mut() {
            match count_index_db_for_roots(connection, &options.roots) {
                Ok(count) => state.indexed_count.store(count, Ordering::Relaxed),
                Err(error) => {
                    eprintln!("[file_search] Failed to recount SQLite index: {}", error);
                    state.indexed_count.store(scanned_count, Ordering::Relaxed);
                }
            }
        } else {
            state.indexed_count.store(scanned_count, Ordering::Relaxed);
        }

        state.is_indexing.store(false, Ordering::Relaxed);

        if state.rebuild_requested.swap(false, Ordering::Relaxed) {
            start_indexing(app, true, Duration::ZERO);
            return;
        }

        if cancelled {
            return;
        }

        start_file_watcher(app, options, generation);
    });
}

pub fn rebuild_file_search_index_after_config_change(app: AppHandle) {
    let state = app.state::<FileSearchState>();
    state.indexed_count.store(0, Ordering::Relaxed);
    state.rebuild_requested.store(true, Ordering::Relaxed);
    state.watcher_generation.fetch_add(1, Ordering::Relaxed);
    start_indexing(app, true, Duration::ZERO);
}
