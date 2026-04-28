use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};

use crate::app_config::AppConfigState;

use super::types::{FileSearchOptions, PlatformFile};

pub(super) fn file_search_options(app: &AppHandle) -> FileSearchOptions {
    let config_state = app.state::<AppConfigState>();
    let Ok(config) = config_state.0.lock() else {
        return FileSearchOptions {
            roots: Vec::new(),
            excluded_paths: Vec::new(),
            include_hidden: false,
        };
    };

    FileSearchOptions {
        roots: normalize_config_paths(&config.file_search_roots),
        excluded_paths: normalize_config_paths(&config.file_search_excluded_paths),
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

pub(super) fn validate_search_result_path(app: &AppHandle, path: &str) -> Result<PathBuf, String> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err("文件路径不能为空".to_string());
    }

    let requested_path = PathBuf::from(trimmed_path);
    let canonical_path =
        fs::canonicalize(&requested_path).map_err(|_| "文件不存在或无法访问".to_string())?;
    let options = file_search_options(app);
    if !is_path_allowed_by_options(&canonical_path, &options) {
        return Err("文件不在当前文件搜索范围内".to_string());
    }

    Ok(canonical_path)
}

pub(super) fn platform_file_from_path(path: &Path) -> Option<PlatformFile> {
    let metadata = fs::metadata(path).ok()?;
    platform_file_from_path_with_kind(path, metadata.is_dir())
}

pub(super) fn platform_file_from_path_with_kind(path: &Path, is_dir: bool) -> Option<PlatformFile> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    Some(PlatformFile {
        name,
        path: path.to_string_lossy().to_string(),
        parent,
        extension,
        is_dir,
    })
}

pub(super) fn is_path_allowed_by_options(path: &Path, options: &FileSearchOptions) -> bool {
    if options.roots.is_empty() {
        return false;
    }

    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let is_in_root = options
        .roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .any(|root| path.starts_with(&root));

    if !is_in_root || should_ignore_path(&path, options) {
        return false;
    }

    !options
        .excluded_paths
        .iter()
        .filter_map(|excluded_path| fs::canonicalize(excluded_path).ok())
        .any(|excluded_path| path.starts_with(&excluded_path))
}

pub(super) fn is_path_allowed_by_options_fast(path: &Path, options: &FileSearchOptions) -> bool {
    if options.roots.is_empty() {
        return false;
    }

    let normalized_path = normalize_path_for_prefix(path);

    let is_in_root = options
        .roots
        .iter()
        .any(|root| path_has_prefix(&normalized_path, &normalize_path_for_prefix(root)));

    if !is_in_root || should_ignore_path(path, options) {
        return false;
    }

    !options.excluded_paths.iter().any(|excluded_path| {
        path_has_prefix(&normalized_path, &normalize_path_for_prefix(excluded_path))
    })
}

fn normalize_path_for_prefix(path: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        path.to_string_lossy().replace('/', "\\").to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string_lossy().to_string()
    }
}

fn path_has_prefix(path: &str, prefix: &str) -> bool {
    let prefix = prefix.trim_end_matches(['\\', '/']);
    path == prefix
        || path
            .strip_prefix(prefix)
            .map(|rest| rest.starts_with('\\') || rest.starts_with('/'))
            .unwrap_or(false)
}

pub(super) fn should_ignore_path(path: &Path, options: &FileSearchOptions) -> bool {
    if is_platform_cache_dir(path) {
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
