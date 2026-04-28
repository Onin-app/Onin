use std::fs;

use super::{path_utils::is_path_allowed_by_options, types::FileSearchOptions};

#[test]
fn path_allowed_by_options_accepts_roots_and_rejects_excludes() {
    let temp_dir = tempfile::Builder::new()
        .prefix("file-search-test")
        .tempdir_in(std::env::current_dir().unwrap())
        .unwrap();
    let root = temp_dir.path().join("root");
    let excluded = root.join("excluded");
    let allowed_file = root.join("allowed.txt");
    let excluded_file = excluded.join("blocked.txt");
    fs::create_dir_all(&excluded).unwrap();
    fs::write(&allowed_file, "allowed").unwrap();
    fs::write(&excluded_file, "blocked").unwrap();

    let options = FileSearchOptions {
        roots: vec![root],
        preferred_paths: Vec::new(),
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

#[test]
fn path_allowed_by_options_respects_hidden_paths() {
    let temp_dir = tempfile::Builder::new()
        .prefix("file-search-test")
        .tempdir_in(std::env::current_dir().unwrap())
        .unwrap();
    let root = temp_dir.path().join("root");
    let hidden_dir = root.join(".hidden");
    let hidden_file = hidden_dir.join("secret.txt");
    fs::create_dir_all(&hidden_dir).unwrap();
    fs::write(&hidden_file, "hidden").unwrap();

    let options = FileSearchOptions {
        roots: vec![root.clone()],
        preferred_paths: Vec::new(),
        excluded_paths: Vec::new(),
        include_hidden: false,
    };
    assert!(!is_path_allowed_by_options(
        &fs::canonicalize(&hidden_file).unwrap(),
        &options
    ));

    let options = FileSearchOptions {
        roots: vec![root],
        preferred_paths: Vec::new(),
        excluded_paths: Vec::new(),
        include_hidden: true,
    };
    assert!(is_path_allowed_by_options(
        &fs::canonicalize(hidden_file).unwrap(),
        &options
    ));
}

#[cfg(target_os = "windows")]
#[test]
fn path_allowed_by_options_accepts_uncanonicalized_windows_paths() {
    let home = std::env::var_os("USERPROFILE").unwrap();
    let home = std::path::PathBuf::from(home);
    let options = FileSearchOptions {
        roots: vec![home.clone()],
        preferred_paths: Vec::new(),
        excluded_paths: Vec::new(),
        include_hidden: false,
    };

    assert!(is_path_allowed_by_options(&home, &options));
}
