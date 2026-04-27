use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use walkdir::{DirEntry, WalkDir};

use crate::app_config::AppConfigState;
use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

const INDEX_WRITE_BATCH_SIZE: usize = 2_000;
const DEFAULT_RESULT_LIMIT: usize = 30;
const INDEX_DB_FILE_NAME: &str = "file_search.sqlite";
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(2);
const FILE_WATCH_DEBOUNCE: Duration = Duration::from_millis(800);
const LIVE_SCAN_ID: &str = "live";

#[derive(Clone, Debug)]
struct IndexedFile {
    name: String,
    path: String,
    parent: String,
    extension: String,
    is_dir: bool,
}

#[derive(Clone, Debug)]
struct FileSearchOptions {
    roots: Vec<PathBuf>,
    excluded_paths: Vec<PathBuf>,
    include_hidden: bool,
}

#[derive(Default)]
pub struct FileSearchState {
    is_indexing: AtomicBool,
    indexed_count: AtomicUsize,
    watcher_generation: AtomicUsize,
    rebuild_requested: AtomicBool,
}

#[derive(Serialize)]
pub struct FileSearchStatus {
    pub is_indexing: bool,
    pub indexed_count: usize,
}

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

        let mut db_connection = match open_index_db(&app) {
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
            let existing_index_count = match count_index_db(&app, &root_string) {
                Ok(count) => count,
                Err(error) => {
                    eprintln!("[file_search] Failed to count SQLite index: {}", error);
                    0
                }
            };

            if existing_index_count > 0 && !reset_index {
                state
                    .indexed_count
                    .fetch_add(existing_index_count, Ordering::Relaxed);
            }

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
                    if existing_index_count == 0 || reset_index {
                        state.indexed_count.store(scanned_count, Ordering::Relaxed);
                    }

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
    state.rebuild_requested.store(true, Ordering::Relaxed);
    state.watcher_generation.fetch_add(1, Ordering::Relaxed);
    start_indexing(app, true, Duration::ZERO);
}

#[tauri::command]
pub fn get_file_search_status(state: tauri::State<FileSearchState>) -> FileSearchStatus {
    FileSearchStatus {
        is_indexing: state.is_indexing.load(Ordering::Relaxed),
        indexed_count: state.indexed_count.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn rebuild_file_search_index(app: AppHandle) -> Result<(), String> {
    let state = app.state::<FileSearchState>();
    if state.is_indexing.load(Ordering::Relaxed) {
        return Err("文件搜索正在建立索引".to_string());
    }

    state.indexed_count.store(0, Ordering::Relaxed);
    start_indexing(app, true, Duration::ZERO);
    Ok(())
}

#[tauri::command]
pub fn search_indexed_files(
    query: String,
    limit: Option<usize>,
    app: AppHandle,
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
    let options = file_search_options(&app);
    let mut candidates = Vec::new();
    for root in &options.roots {
        let root = root.to_string_lossy().to_string();
        match search_index_db(&app, &root, &terms) {
            Ok(root_candidates) => candidates.extend(root_candidates),
            Err(error) => {
                eprintln!("[file_search] Failed to search SQLite index: {}", error);
            }
        }
    }

    let mut top_results: Vec<(i32, IndexedFile)> = Vec::with_capacity(limit);

    for file in candidates {
        let Some(score) = score_file(&file, &terms) else {
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
            if compare_scored_files(&(score, file.clone()), &top_results[worst_index]).is_gt() {
                top_results[worst_index] = (score, file);
            }
        }
    }

    top_results.sort_by(|a, b| compare_scored_files(b, a));

    top_results
        .into_iter()
        .map(|(_, file)| launchable_item_from_file(&file))
        .collect()
}

#[tauri::command]
pub fn open_indexed_file(path: String, app: AppHandle) -> Result<(), String> {
    let path = validate_indexed_file_path(&app, &path)?;
    opener::open(&path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))
}

fn index_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_data_dir
        .join("extensions")
        .join("filesearch")
        .join(INDEX_DB_FILE_NAME))
}

fn open_index_db(app: &AppHandle) -> Result<Connection, String> {
    let db_path = index_db_path(app)?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let connection = Connection::open(db_path).map_err(|e| e.to_string())?;
    connection
        .busy_timeout(SQLITE_BUSY_TIMEOUT)
        .map_err(|e| e.to_string())?;
    initialize_index_db(&connection)?;
    Ok(connection)
}

fn initialize_index_db(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS file_search_entries (
                path TEXT PRIMARY KEY,
                root TEXT NOT NULL,
                name TEXT NOT NULL,
                parent TEXT NOT NULL,
                extension TEXT NOT NULL,
                is_dir INTEGER NOT NULL,
                scan_id TEXT NOT NULL DEFAULT ''
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS file_search_entries_fts USING fts5(
                path UNINDEXED,
                name,
                parent,
                full_path,
                extension
            );

            CREATE TABLE IF NOT EXISTS file_search_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| e.to_string())?;

    if !has_column(connection, "file_search_entries", "scan_id")? {
        connection
            .execute(
                "ALTER TABLE file_search_entries ADD COLUMN scan_id TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|e| e.to_string())?;
    }

    connection
        .execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_file_search_entries_root_scan_id
                ON file_search_entries(root, scan_id);
            CREATE INDEX IF NOT EXISTS idx_file_search_entries_root
                ON file_search_entries(root);
            CREATE INDEX IF NOT EXISTS idx_file_search_entries_name
                ON file_search_entries(name);
            CREATE INDEX IF NOT EXISTS idx_file_search_entries_extension
                ON file_search_entries(extension);
            "#,
        )
        .map_err(|e| e.to_string())?;

    backfill_fts_index_if_needed(connection)
}

fn file_search_options(app: &AppHandle) -> FileSearchOptions {
    let config_state = app.state::<AppConfigState>();
    let Ok(config) = config_state.0.lock() else {
        return FileSearchOptions {
            roots: Vec::new(),
            excluded_paths: Vec::new(),
            include_hidden: false,
        };
    };

    let roots = normalize_config_paths(&config.file_search_roots);
    let excluded_paths = normalize_config_paths(&config.file_search_excluded_paths);

    FileSearchOptions {
        roots,
        excluded_paths,
        include_hidden: config.file_search_include_hidden,
    }
}

fn normalize_config_paths(paths: &[String]) -> Vec<PathBuf> {
    let mut normalized = paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn clear_index_db(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            DELETE FROM file_search_entries;
            DELETE FROM file_search_entries_fts;
            "#,
        )
        .map_err(|e| e.to_string())?;
    set_file_search_meta(connection, "fts_backfilled", "1")?;
    Ok(())
}

fn backfill_fts_index_if_needed(connection: &Connection) -> Result<(), String> {
    if file_search_meta_value(connection, "fts_backfilled")?.as_deref() == Some("1") {
        return Ok(());
    }

    let entry_count = connection
        .query_row("SELECT COUNT(*) FROM file_search_entries", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())?;
    if entry_count == 0 {
        set_file_search_meta(connection, "fts_backfilled", "1")?;
        return Ok(());
    }

    let fts_count = connection
        .query_row("SELECT COUNT(*) FROM file_search_entries_fts", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())?;

    if fts_count == entry_count {
        set_file_search_meta(connection, "fts_backfilled", "1")?;
        return Ok(());
    }

    connection
        .execute_batch("DELETE FROM file_search_entries_fts")
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO file_search_entries_fts
                (path, name, parent, full_path, extension)
            SELECT path, name, parent, path, extension
            FROM file_search_entries",
            [],
        )
        .map_err(|e| e.to_string())?;
    set_file_search_meta(connection, "fts_backfilled", "1")?;

    Ok(())
}

fn file_search_meta_value(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value FROM file_search_meta WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())
}

fn set_file_search_meta(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO file_search_meta (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn count_index_db(app: &AppHandle, root: &str) -> Result<usize, String> {
    let db_path = index_db_path(app)?;
    if !db_path.exists() {
        return Ok(0);
    }

    let connection = open_index_db(app)?;
    count_index_db_with_connection(&connection, root)
}

fn count_index_db_for_roots(connection: &Connection, roots: &[PathBuf]) -> Result<usize, String> {
    roots.iter().try_fold(0usize, |total, root| {
        let root = root.to_string_lossy().to_string();
        count_index_db_with_connection(connection, &root).map(|count| total + count)
    })
}

fn count_index_db_with_connection(connection: &Connection, root: &str) -> Result<usize, String> {
    let count = connection
        .query_row(
            "SELECT COUNT(*) FROM file_search_entries WHERE root = ?1",
            params![root],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(count.max(0) as usize)
}

fn indexed_path_exists(connection: &Connection, path: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM file_search_entries WHERE path = ?1)",
            params![path],
            |row| row.get::<_, i64>(0),
        )
        .map(|exists| exists != 0)
        .map_err(|e| e.to_string())
}

fn validate_indexed_file_path(app: &AppHandle, path: &str) -> Result<PathBuf, String> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err("文件路径不能为空".to_string());
    }

    let requested_path = PathBuf::from(trimmed_path);
    let canonical_path =
        fs::canonicalize(&requested_path).map_err(|_| "文件不存在或无法访问".to_string())?;
    let options = file_search_options(app);
    if !is_path_allowed_by_options(&canonical_path, &options) {
        return Err("文件不在当前文件搜索索引范围内".to_string());
    }

    let connection = open_index_db(app)?;
    let canonical_path_string = canonical_path.to_string_lossy().to_string();
    if !indexed_path_exists(&connection, trimmed_path)?
        && !indexed_path_exists(&connection, &canonical_path_string)?
    {
        return Err("文件不在当前索引中".to_string());
    }

    Ok(canonical_path)
}

fn is_path_allowed_by_options(path: &Path, options: &FileSearchOptions) -> bool {
    if options.roots.is_empty() {
        return false;
    }

    let is_in_root = options
        .roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .any(|root| path.starts_with(root));

    if !is_in_root {
        return false;
    }

    !options
        .excluded_paths
        .iter()
        .filter_map(|excluded_path| fs::canonicalize(excluded_path).ok())
        .any(|excluded_path| path.starts_with(excluded_path))
}

fn search_index_db(
    app: &AppHandle,
    root: &str,
    terms: &[String],
) -> Result<Vec<IndexedFile>, String> {
    let db_path = index_db_path(app)?;
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let connection = open_index_db(app)?;
    search_index_db_with_connection(&connection, root, terms)
}

fn search_index_db_with_connection(
    connection: &Connection,
    root: &str,
    terms: &[String],
) -> Result<Vec<IndexedFile>, String> {
    let uses_fts = terms.iter().any(|term| !term.starts_with('.'));

    match search_index_db_with_fts(connection, root, terms) {
        Ok(entries) if !entries.is_empty() || !uses_fts => Ok(entries),
        Ok(_) => search_index_db_with_like(connection, root, terms),
        Err(error) => {
            eprintln!(
                "[file_search] Failed to search SQLite FTS index, falling back to LIKE: {}",
                error
            );
            search_index_db_with_like(connection, root, terms)
        }
    }
}

fn search_index_db_with_fts(
    connection: &Connection,
    root: &str,
    terms: &[String],
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];
    let mut fts_terms = Vec::new();

    for term in terms {
        if term.starts_with('.') {
            clauses.push("extension = ?".to_string());
            values.push(Value::Text(term.clone()));
            continue;
        }

        let fts_term = fts_prefix_term(term);
        fts_terms.push(format!(
            "(name:{0} OR parent:{0} OR full_path:{0})",
            fts_term
        ));
    }

    if !fts_terms.is_empty() {
        clauses.push(
            "path IN (
                SELECT path
                FROM file_search_entries_fts
                WHERE file_search_entries_fts MATCH ?
            )"
            .to_string(),
        );
        values.push(Value::Text(fts_terms.join(" AND ")));
    }

    let sql = format!(
        r#"
        SELECT name, path, parent, extension, is_dir
        FROM file_search_entries
        WHERE {}
        "#,
        clauses.join(" AND ")
    );

    let mut statement = connection.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), |row| {
            Ok(IndexedFile {
                name: row.get(0)?,
                path: row.get(1)?,
                parent: row.get(2)?,
                extension: row.get(3)?,
                is_dir: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }

    Ok(entries)
}

fn search_index_db_with_like(
    connection: &Connection,
    root: &str,
    terms: &[String],
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];

    for term in terms {
        if term.starts_with('.') {
            clauses.push("extension = ?".to_string());
            values.push(Value::Text(term.clone()));
            continue;
        }

        let like_term = format!("%{}%", escape_like_term(term));
        clauses.push(
            "(name LIKE ? ESCAPE '\\' OR parent LIKE ? ESCAPE '\\' OR path LIKE ? ESCAPE '\\')"
                .to_string(),
        );
        values.push(Value::Text(like_term.clone()));
        values.push(Value::Text(like_term.clone()));
        values.push(Value::Text(like_term));
    }

    let sql = format!(
        r#"
        SELECT name, path, parent, extension, is_dir
        FROM file_search_entries
        WHERE {}
        "#,
        clauses.join(" AND ")
    );

    let mut statement = connection.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), |row| {
            Ok(IndexedFile {
                name: row.get(0)?,
                path: row.get(1)?,
                parent: row.get(2)?,
                extension: row.get(3)?,
                is_dir: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }

    Ok(entries)
}

fn save_index_batch(
    connection: &mut Connection,
    root: &str,
    scan_id: &str,
    entries: &[IndexedFile],
) -> Result<(), String> {
    let transaction = connection.transaction().map_err(|e| e.to_string())?;

    {
        let mut entry_statement = transaction
            .prepare(
                r#"
                INSERT INTO file_search_entries
                    (path, root, name, parent, extension, is_dir, scan_id)
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(path) DO UPDATE SET
                    root = excluded.root,
                    name = excluded.name,
                    parent = excluded.parent,
                    extension = excluded.extension,
                    is_dir = excluded.is_dir,
                    scan_id = excluded.scan_id
                "#,
            )
            .map_err(|e| e.to_string())?;
        let mut delete_fts_statement = transaction
            .prepare("DELETE FROM file_search_entries_fts WHERE path = ?1")
            .map_err(|e| e.to_string())?;
        let mut insert_fts_statement = transaction
            .prepare(
                r#"
                INSERT INTO file_search_entries_fts
                    (path, name, parent, full_path, extension)
                VALUES
                    (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .map_err(|e| e.to_string())?;

        for entry in entries {
            entry_statement
                .execute(params![
                    &entry.path,
                    root,
                    &entry.name,
                    &entry.parent,
                    &entry.extension,
                    if entry.is_dir { 1_i64 } else { 0_i64 },
                    scan_id,
                ])
                .map_err(|e| e.to_string())?;
            delete_fts_statement
                .execute(params![&entry.path])
                .map_err(|e| e.to_string())?;
            insert_fts_statement
                .execute(params![
                    &entry.path,
                    &entry.name,
                    &entry.parent,
                    &entry.path,
                    &entry.extension,
                ])
                .map_err(|e| e.to_string())?;
        }
    }

    transaction.commit().map_err(|e| e.to_string())
}

fn finalize_index_scan(connection: &Connection, root: &str, scan_id: &str) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM file_search_entries_fts
            WHERE path IN (
                SELECT path
                FROM file_search_entries
                WHERE root = ?1 AND scan_id != ?2
            )",
            params![root, scan_id],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "DELETE FROM file_search_entries WHERE root = ?1 AND scan_id != ?2",
            params![root, scan_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn start_file_watcher(app: AppHandle, options: FileSearchOptions, generation: usize) {
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

fn delete_index_path(connection: &Connection, root: &str, path: &Path) -> Result<(), String> {
    let path = path.to_string_lossy().to_string();
    let separator = std::path::MAIN_SEPARATOR;
    let child_prefix = format!("{}{}%", escape_like_term(&path), separator);

    connection
        .execute(
            "DELETE FROM file_search_entries_fts
            WHERE path IN (
                SELECT path
                FROM file_search_entries
                WHERE root = ?1 AND (path = ?2 OR path LIKE ?3 ESCAPE '\\')
            )",
            params![root, path, child_prefix],
        )
        .map_err(|e| e.to_string())?;

    connection
        .execute(
            "DELETE FROM file_search_entries WHERE root = ?1 AND (path = ?2 OR path LIKE ?3 ESCAPE '\\')",
            params![root, path, child_prefix],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn has_column(connection: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;

    for row in rows {
        if row.map_err(|e| e.to_string())? == column {
            return Ok(true);
        }
    }

    Ok(false)
}

fn new_scan_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("scan-{}", millis)
}

fn escape_like_term(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn fts_prefix_term(term: &str) -> String {
    format!("\"{}\"*", term.replace('"', "\"\""))
}

fn indexed_file_from_entry(entry: &DirEntry) -> Option<IndexedFile> {
    indexed_file_from_path(entry.path(), entry.file_type().is_dir())
}

fn indexed_file_from_path(path: &Path, is_dir: bool) -> Option<IndexedFile> {
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
        is_dir,
    })
}

fn should_skip_entry(entry: &DirEntry, options: &FileSearchOptions) -> bool {
    let path = entry.path();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if entry.depth() == 0 {
        return false;
    }

    if should_ignore_path(path, options) {
        return true;
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

fn should_ignore_path(path: &Path, options: &FileSearchOptions) -> bool {
    if is_platform_cache_dir(path) {
        return true;
    }

    if options
        .excluded_paths
        .iter()
        .any(|excluded_path| path.starts_with(excluded_path))
    {
        return true;
    }

    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_lowercase();
        if !options.include_hidden && name.starts_with('.') {
            return true;
        }

        matches!(
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
        )
    })
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
    (score_a, file_a): &(i32, IndexedFile),
    (score_b, file_b): &(i32, IndexedFile),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn indexed_file(name: &str, path: &str, extension: &str) -> IndexedFile {
        IndexedFile {
            name: name.to_string(),
            path: path.to_string(),
            parent: "C:/root/docs".to_string(),
            extension: extension.to_string(),
            is_dir: false,
        }
    }

    fn setup_search_db() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize_index_db(&connection).unwrap();
        save_index_batch(
            &mut connection,
            "C:/root",
            "scan-test",
            &[
                indexed_file("ProjectNotes.md", "C:/root/docs/ProjectNotes.md", ".md"),
                indexed_file("image.png", "C:/root/docs/image.png", ".png"),
            ],
        )
        .unwrap();
        connection
    }

    #[test]
    fn search_uses_fts_prefix_matches() {
        let connection = setup_search_db();
        let results =
            search_index_db_with_connection(&connection, "C:/root", &parse_terms("proj")).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ProjectNotes.md");
    }

    #[test]
    fn search_falls_back_to_like_for_substring_matches() {
        let connection = setup_search_db();
        let results =
            search_index_db_with_connection(&connection, "C:/root", &parse_terms("ject")).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ProjectNotes.md");
    }

    #[test]
    fn search_combines_fts_and_extension_filters() {
        let connection = setup_search_db();
        let results =
            search_index_db_with_connection(&connection, "C:/root", &parse_terms("proj .md"))
                .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ProjectNotes.md");
    }

    #[test]
    fn indexed_path_exists_checks_exact_index_entries() {
        let connection = setup_search_db();

        assert!(indexed_path_exists(&connection, "C:/root/docs/ProjectNotes.md").unwrap());
        assert!(!indexed_path_exists(&connection, "C:/root/docs/missing.md").unwrap());
    }

    #[test]
    fn path_allowed_by_options_accepts_roots_and_rejects_excludes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().join("root");
        let excluded = root.join("excluded");
        let allowed_file = root.join("allowed.txt");
        let excluded_file = excluded.join("blocked.txt");
        fs::create_dir_all(&excluded).unwrap();
        fs::write(&allowed_file, "allowed").unwrap();
        fs::write(&excluded_file, "blocked").unwrap();

        let options = FileSearchOptions {
            roots: vec![root],
            excluded_paths: vec![excluded],
            include_hidden: false,
        };

        assert!(is_path_allowed_by_options(
            &fs::canonicalize(allowed_file).unwrap(),
            &options
        ));
        assert!(!is_path_allowed_by_options(
            &fs::canonicalize(excluded_file).unwrap(),
            &options
        ));
    }
}
