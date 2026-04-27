use rusqlite::{params_from_iter, types::Value, Connection};
use tauri::AppHandle;

use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

use super::db::open_index_db;
use super::path_utils::file_search_options;
use super::types::{IndexedFile, DEFAULT_RESULT_LIMIT, SEARCH_CANDIDATE_LIMIT};
use super::utils::{escape_like_term, fts_phrase_term, fts_prefix_term};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SearchTerm {
    Text(String),
    Extension(String),
    Kind(SearchKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SearchKind {
    File,
    Folder,
}

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
    terms: &[SearchTerm],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let uses_fts = terms.iter().any(|term| matches!(term, SearchTerm::Text(_)));
    let can_use_trigram = terms
        .iter()
        .filter_map(|term| match term {
            SearchTerm::Text(text) => Some(text),
            _ => None,
        })
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
    terms: &[SearchTerm],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];
    let mut fts_terms = Vec::new();

    for term in terms {
        match term {
            SearchTerm::Text(text) => {
                let fts_term = fts_text_term(text);
                fts_terms.push(format!(
                    "(name:{0} OR parent:{0} OR full_path:{0})",
                    fts_term
                ));
            }
            SearchTerm::Extension(extension) => {
                clauses.push("extension = ?".to_string());
                values.push(Value::Text(extension.clone()));
            }
            SearchTerm::Kind(kind) => {
                clauses.push("is_dir = ?".to_string());
                values.push(Value::Integer(if matches!(kind, SearchKind::Folder) {
                    1
                } else {
                    0
                }));
            }
        }
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
    terms: &[SearchTerm],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];
    let mut trigram_terms = Vec::new();

    for term in terms {
        match term {
            SearchTerm::Text(text) => {
                trigram_terms.push(format!(
                    "(name:{0} OR parent:{0} OR full_path:{0})",
                    fts_phrase_term(text)
                ));
            }
            SearchTerm::Extension(extension) => {
                clauses.push("extension = ?".to_string());
                values.push(Value::Text(extension.clone()));
            }
            SearchTerm::Kind(kind) => {
                clauses.push("is_dir = ?".to_string());
                values.push(Value::Integer(if matches!(kind, SearchKind::Folder) {
                    1
                } else {
                    0
                }));
            }
        }
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
    terms: &[SearchTerm],
    candidate_limit: usize,
) -> Result<Vec<IndexedFile>, String> {
    let mut clauses = vec!["root = ?".to_string()];
    let mut values = vec![Value::Text(root.to_string())];

    for term in terms {
        match term {
            SearchTerm::Text(text) => {
                let like_term = format!("%{}%", escape_like_term(text));
                clauses.push(
                    "(name LIKE ? ESCAPE '\\' OR parent LIKE ? ESCAPE '\\' OR path LIKE ? ESCAPE '\\')"
                        .to_string(),
                );
                values.push(Value::Text(like_term.clone()));
                values.push(Value::Text(like_term.clone()));
                values.push(Value::Text(like_term));
            }
            SearchTerm::Extension(extension) => {
                clauses.push("extension = ?".to_string());
                values.push(Value::Text(extension.clone()));
            }
            SearchTerm::Kind(kind) => {
                clauses.push("is_dir = ?".to_string());
                values.push(Value::Integer(if matches!(kind, SearchKind::Folder) {
                    1
                } else {
                    0
                }));
            }
        }
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

pub(super) fn parse_terms(query: &str) -> Vec<SearchTerm> {
    split_query_tokens(query)
        .into_iter()
        .filter_map(|token| parse_query_token(&token))
        .collect()
}

fn split_query_tokens(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for character in query.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                if !in_quotes && !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            character if character.is_whitespace() && !in_quotes => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

fn parse_query_token(token: &str) -> Option<SearchTerm> {
    let token = token.trim().to_lowercase();
    if token.is_empty() {
        return None;
    }

    if let Some(extension) = token
        .strip_prefix("ext:")
        .or_else(|| token.strip_prefix("extension:"))
    {
        return normalize_extension_filter(extension).map(SearchTerm::Extension);
    }

    if token.starts_with('.') {
        return normalize_extension_filter(&token).map(SearchTerm::Extension);
    }

    if let Some(kind) = token
        .strip_prefix("type:")
        .or_else(|| token.strip_prefix("kind:"))
        .and_then(parse_kind_filter)
    {
        return Some(SearchTerm::Kind(kind));
    }

    Some(SearchTerm::Text(token))
}

fn normalize_extension_filter(extension: &str) -> Option<String> {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() {
        return None;
    }

    Some(format!(".{}", extension))
}

fn parse_kind_filter(kind: &str) -> Option<SearchKind> {
    match kind.trim() {
        "file" | "files" => Some(SearchKind::File),
        "folder" | "folders" | "dir" | "dirs" | "directory" | "directories" => {
            Some(SearchKind::Folder)
        }
        _ => None,
    }
}

fn fts_text_term(term: &str) -> String {
    if term.chars().any(char::is_whitespace) {
        fts_phrase_term(term)
    } else {
        fts_prefix_term(term)
    }
}

fn score_file(file: &IndexedFile, terms: &[SearchTerm]) -> Option<i32> {
    let name = file.name.to_lowercase();
    let parent = file.parent.to_lowercase();
    let path = file.path.to_lowercase();
    let mut score = if file.is_dir { 10 } else { 0 };

    for term in terms {
        match term {
            SearchTerm::Extension(extension) => {
                if file.extension == *extension {
                    score += 30;
                    continue;
                }
                return None;
            }
            SearchTerm::Kind(SearchKind::Folder) => {
                if file.is_dir {
                    score += 20;
                    continue;
                }
                return None;
            }
            SearchTerm::Kind(SearchKind::File) => {
                if !file.is_dir {
                    score += 20;
                    continue;
                }
                return None;
            }
            SearchTerm::Text(text) if name == *text => {
                score += 100;
            }
            SearchTerm::Text(text) if name.starts_with(text) => {
                score += 75;
            }
            SearchTerm::Text(text) if name.contains(text) => {
                score += 45;
            }
            SearchTerm::Text(text) if parent.contains(text) => {
                score += 15;
            }
            SearchTerm::Text(text) if path.contains(text) => {
                score += 5;
            }
            SearchTerm::Text(_) => {
                return None;
            }
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
