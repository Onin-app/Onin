use std::fs;

use rusqlite::Connection;

use super::db::{indexed_path_exists, initialize_index_db, save_index_batch};
use super::path_utils::is_path_allowed_by_options;
use super::search::{
    parse_terms, search_index_db_with_connection, search_index_db_with_trigram, SearchKind,
    SearchTerm,
};
use super::types::{FileSearchOptions, IndexedFile, SEARCH_CANDIDATE_LIMIT};

fn indexed_file(name: &str, path: &str, extension: &str) -> IndexedFile {
    IndexedFile {
        name: name.to_string(),
        path: path.to_string(),
        parent: "C:/root/docs".to_string(),
        extension: extension.to_string(),
        is_dir: false,
    }
}

fn indexed_folder(name: &str, path: &str) -> IndexedFile {
    IndexedFile {
        name: name.to_string(),
        path: path.to_string(),
        parent: "C:/root".to_string(),
        extension: String::new(),
        is_dir: true,
    }
}

fn setup_search_db() -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize_index_db(&connection, true).unwrap();
    save_index_batch(
        &mut connection,
        "C:/root",
        "scan-test",
        &[
            indexed_file("ProjectNotes.md", "C:/root/docs/ProjectNotes.md", ".md"),
            indexed_file("Project Plan.md", "C:/root/docs/Project Plan.md", ".md"),
            indexed_file("image.png", "C:/root/docs/image.png", ".png"),
            indexed_folder("Projects", "C:/root/Projects"),
        ],
    )
    .unwrap();
    connection
}

#[test]
fn search_uses_fts_prefix_matches() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("projectnotes"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "ProjectNotes.md");
}

#[test]
fn search_falls_back_to_like_for_substring_matches() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("jectnotes"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "ProjectNotes.md");
}

#[test]
fn trigram_search_matches_filename_substrings() {
    let connection = setup_search_db();
    let results = search_index_db_with_trigram(
        &connection,
        "C:/root",
        &parse_terms("jectnotes"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "ProjectNotes.md");
}

#[test]
fn search_combines_fts_and_extension_filters() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("projectnotes .md"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "ProjectNotes.md");
}

#[test]
fn parse_terms_supports_filters_and_quoted_phrases() {
    assert_eq!(
        parse_terms("\"project plan\" ext:md type:file"),
        vec![
            SearchTerm::Text("project plan".to_string()),
            SearchTerm::Extension(".md".to_string()),
            SearchTerm::Kind(SearchKind::File),
        ]
    );
}

#[test]
fn search_supports_extension_filter_syntax() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("ext:png"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "image.png");
}

#[test]
fn search_supports_kind_filter_syntax() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("type:folder proj"),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Projects");
    assert!(results[0].is_dir);
}

#[test]
fn search_supports_quoted_phrase_queries() {
    let connection = setup_search_db();
    let results = search_index_db_with_connection(
        &connection,
        "C:/root",
        &parse_terms("\"project plan\""),
        SEARCH_CANDIDATE_LIMIT,
    )
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Project Plan.md");
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
