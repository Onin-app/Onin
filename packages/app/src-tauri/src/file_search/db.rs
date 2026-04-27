use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use super::types::{IndexedFile, INDEX_DB_FILE_NAME, SQLITE_BUSY_TIMEOUT};
use super::utils::{escape_like_term, fts_rowid_for_path};

pub(super) fn index_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_data_dir
        .join("extensions")
        .join("filesearch")
        .join(INDEX_DB_FILE_NAME))
}

pub(super) fn open_index_db(app: &AppHandle) -> Result<Connection, String> {
    open_index_db_with_backfill(app, false)
}

pub(super) fn open_index_db_for_indexing(
    app: &AppHandle,
    _reset_index: bool,
) -> Result<Connection, String> {
    open_index_db_with_backfill(app, false)
}

fn open_index_db_with_backfill(app: &AppHandle, backfill_fts: bool) -> Result<Connection, String> {
    let db_path = index_db_path(app)?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let connection = Connection::open(db_path).map_err(|e| e.to_string())?;
    connection
        .busy_timeout(SQLITE_BUSY_TIMEOUT)
        .map_err(|e| e.to_string())?;
    initialize_index_db(&connection, backfill_fts)?;
    Ok(connection)
}

pub(super) fn initialize_index_db(
    connection: &Connection,
    backfill_fts: bool,
) -> Result<(), String> {
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

            CREATE VIRTUAL TABLE IF NOT EXISTS file_search_entries_trigram USING fts5(
                path UNINDEXED,
                name,
                parent,
                full_path,
                tokenize = 'trigram'
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

    if backfill_fts {
        backfill_fts_index_if_needed(connection)?;
    }

    Ok(())
}

pub(super) fn clear_index_db(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            DELETE FROM file_search_entries;
            DELETE FROM file_search_entries_fts;
            DELETE FROM file_search_entries_trigram;
            "#,
        )
        .map_err(|e| e.to_string())?;
    set_file_search_meta(connection, "fts_backfilled", "1")?;
    Ok(())
}

fn backfill_fts_index_if_needed(connection: &Connection) -> Result<(), String> {
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

    let trigram_count = connection
        .query_row(
            "SELECT COUNT(*) FROM file_search_entries_trigram",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;

    if file_search_meta_value(connection, "fts_backfilled")?.as_deref() == Some("1")
        && fts_count == entry_count
        && trigram_count == entry_count
    {
        set_file_search_meta(connection, "fts_backfilled", "1")?;
        return Ok(());
    }

    connection
        .execute_batch("DELETE FROM file_search_entries_fts")
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch("DELETE FROM file_search_entries_trigram")
        .map_err(|e| e.to_string())?;
    let mut select_statement = connection
        .prepare("SELECT path, name, parent, extension FROM file_search_entries")
        .map_err(|e| e.to_string())?;
    let rows = select_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }

    let mut insert_statement = connection
        .prepare(
            "INSERT OR REPLACE INTO file_search_entries_fts
                (rowid, path, name, parent, full_path, extension)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|e| e.to_string())?;
    let mut insert_trigram_statement = connection
        .prepare(
            "INSERT OR REPLACE INTO file_search_entries_trigram
                (rowid, path, name, parent, full_path)
            VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| e.to_string())?;
    for (path, name, parent, extension) in entries {
        insert_statement
            .execute(params![
                fts_rowid_for_path(&path),
                &path,
                &name,
                &parent,
                &path,
                &extension,
            ])
            .map_err(|e| e.to_string())?;
        insert_trigram_statement
            .execute(params![
                fts_rowid_for_path(&path),
                &path,
                &name,
                &parent,
                &path,
            ])
            .map_err(|e| e.to_string())?;
    }
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

pub(super) fn count_index_db_for_roots(
    connection: &Connection,
    roots: &[PathBuf],
) -> Result<usize, String> {
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

pub(super) fn indexed_path_exists(connection: &Connection, path: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM file_search_entries WHERE path = ?1)",
            params![path],
            |row| row.get::<_, i64>(0),
        )
        .map(|exists| exists != 0)
        .map_err(|e| e.to_string())
}

pub(super) fn save_index_batch(
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
        let mut insert_fts_statement = transaction
            .prepare(
                r#"
                INSERT OR REPLACE INTO file_search_entries_fts
                    (rowid, path, name, parent, full_path, extension)
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )
            .map_err(|e| e.to_string())?;
        let mut insert_trigram_statement = transaction
            .prepare(
                r#"
                INSERT OR REPLACE INTO file_search_entries_trigram
                    (rowid, path, name, parent, full_path)
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
            insert_fts_statement
                .execute(params![
                    fts_rowid_for_path(&entry.path),
                    &entry.path,
                    &entry.name,
                    &entry.parent,
                    &entry.path,
                    &entry.extension,
                ])
                .map_err(|e| e.to_string())?;
            insert_trigram_statement
                .execute(params![
                    fts_rowid_for_path(&entry.path),
                    &entry.path,
                    &entry.name,
                    &entry.parent,
                    &entry.path,
                ])
                .map_err(|e| e.to_string())?;
        }
    }

    transaction.commit().map_err(|e| e.to_string())
}

pub(super) fn finalize_index_scan(
    connection: &Connection,
    root: &str,
    scan_id: &str,
) -> Result<(), String> {
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
            "DELETE FROM file_search_entries_trigram
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

pub(super) fn delete_index_path(
    connection: &Connection,
    root: &str,
    path: &Path,
) -> Result<(), String> {
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
            "DELETE FROM file_search_entries_trigram
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
