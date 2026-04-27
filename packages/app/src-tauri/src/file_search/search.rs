use rusqlite::{params_from_iter, types::Value, Connection};
use tauri::AppHandle;

use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

use super::db::open_index_db;
use super::path_utils::file_search_options;
use super::types::{IndexedFile, DEFAULT_RESULT_LIMIT, SEARCH_CANDIDATE_LIMIT};
use super::utils::{escape_like_term, fts_phrase_term, fts_prefix_term};

pub(super) fn search_indexed_files_blocking(
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
    let candidate_limit = SEARCH_CANDIDATE_LIMIT.max(limit * 20);
    let mut candidates = Vec::new();

    match open_index_db(&app) {
        Ok(connection) => {
            for root in &options.roots {
                let root = root.to_string_lossy().to_string();
                match search_index_db_with_connection(&connection, &root, &terms, candidate_limit) {
                    Ok(root_candidates) => candidates.extend(root_candidates),
                    Err(error) => {
                        eprintln!("[file_search] Failed to search SQLite index: {}", error);
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("[file_search] Failed to open SQLite index: {}", error);
            return Vec::new();
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

pub(super) fn search_index_db_with_connection(
    connection: &Connection,
    root: &str,
    terms: &[String],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let uses_fts = terms.iter().any(|term| !term.starts_with('.'));
    let can_use_trigram = terms
        .iter()
        .filter(|term| !term.starts_with('.'))
        .all(|term| term.chars().count() >= 3);

    match search_index_db_with_fts(connection, root, terms, candidate_limit) {
        Ok(entries) if !entries.is_empty() || !uses_fts => Ok(entries),
        Ok(_) if can_use_trigram => {
            match search_index_db_with_trigram(connection, root, terms, candidate_limit) {
                Ok(entries) if !entries.is_empty() => Ok(entries),
                Ok(_) => search_index_db_with_like(connection, root, terms, candidate_limit),
                Err(error) => {
                    eprintln!(
                        "[file_search] Failed to search SQLite trigram index, falling back to LIKE: {}",
                        error
                    );
                    search_index_db_with_like(connection, root, terms, candidate_limit)
                }
            }
        }
        Ok(_) => search_index_db_with_like(connection, root, terms, candidate_limit),
        Err(error) => {
            eprintln!(
                "[file_search] Failed to search SQLite FTS index, falling back to LIKE: {}",
                error
            );
            if can_use_trigram {
                match search_index_db_with_trigram(connection, root, terms, candidate_limit) {
                    Ok(entries) if !entries.is_empty() => Ok(entries),
                    Ok(_) => search_index_db_with_like(connection, root, terms, candidate_limit),
                    Err(error) => {
                        eprintln!(
                            "[file_search] Failed to search SQLite trigram index, falling back to LIKE: {}",
                            error
                        );
                        search_index_db_with_like(connection, root, terms, candidate_limit)
                    }
                }
            } else {
                search_index_db_with_like(connection, root, terms, candidate_limit)
            }
        }
    }
}

fn search_index_db_with_fts(
    connection: &Connection,
    root: &str,
    terms: &[String],
    candidate_limit: usize,
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
                SELECT file_search_entries_fts.path
                FROM file_search_entries_fts
                WHERE file_search_entries_fts MATCH ?
                    AND path IN (
                        SELECT path
                        FROM file_search_entries
                        WHERE root = ?
                    )
                ORDER BY rank
                LIMIT ?
            )"
            .to_string(),
        );
        values.push(Value::Text(fts_terms.join(" AND ")));
        values.push(Value::Text(root.to_string()));
        values.push(Value::Integer(candidate_limit as i64));
    }

    let sql = format!(
        r#"
        SELECT name, path, parent, extension, is_dir
        FROM file_search_entries
        WHERE {}
        ORDER BY is_dir DESC, length(name) ASC, name ASC
        LIMIT ?
        "#,
        clauses.join(" AND ")
    );
    values.push(Value::Integer(candidate_limit as i64));

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

pub(super) fn search_index_db_with_trigram(
    connection: &Connection,
    root: &str,
    terms: &[String],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];
    let mut trigram_terms = Vec::new();

    for term in terms {
        if term.starts_with('.') {
            clauses.push("extension = ?".to_string());
            values.push(Value::Text(term.clone()));
            continue;
        }

        trigram_terms.push(format!(
            "(name:{0} OR parent:{0} OR full_path:{0})",
            fts_phrase_term(term)
        ));
    }

    if !trigram_terms.is_empty() {
        clauses.push(
            "path IN (
                SELECT file_search_entries_trigram.path
                FROM file_search_entries_trigram
                WHERE file_search_entries_trigram MATCH ?
                    AND path IN (
                        SELECT path
                        FROM file_search_entries
                        WHERE root = ?
                    )
                ORDER BY rank
                LIMIT ?
            )"
            .to_string(),
        );
        values.push(Value::Text(trigram_terms.join(" AND ")));
        values.push(Value::Text(root.to_string()));
        values.push(Value::Integer(candidate_limit as i64));
    }

    let sql = format!(
        r#"
        SELECT name, path, parent, extension, is_dir
        FROM file_search_entries
        WHERE {}
        ORDER BY is_dir DESC, length(name) ASC, name ASC
        LIMIT ?
        "#,
        clauses.join(" AND ")
    );
    values.push(Value::Integer(candidate_limit as i64));

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
    candidate_limit: usize,
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
        ORDER BY is_dir DESC, length(name) ASC, name ASC
        LIMIT ?
        "#,
        clauses.join(" AND ")
    );
    values.push(Value::Integer(candidate_limit as i64));

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

pub(super) fn parse_terms(query: &str) -> Vec<String> {
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
