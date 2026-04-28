use std::{cmp::Ordering, collections::HashSet, path::PathBuf, process::Command};

use serde::Deserialize;
use tauri::AppHandle;

use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

use super::{
    path_utils::{file_search_options, is_path_allowed_by_options, platform_file_from_path},
    types::{FileSearchOptions, PlatformFile, DEFAULT_RESULT_LIMIT},
};

pub(super) fn backend_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows Search"
    }
    #[cfg(target_os = "macos")]
    {
        "Spotlight"
    }
    #[cfg(target_os = "linux")]
    {
        "locate/plocate"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unsupported"
    }
}

pub(super) fn backend_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        true
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("mdfind").arg("-version").output().is_ok()
    }
    #[cfg(target_os = "linux")]
    {
        command_exists("plocate") || command_exists("locate")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

pub(super) fn search_platform_files(
    query: String,
    limit: Option<usize>,
    app: &AppHandle,
) -> Result<Vec<LaunchableItem>, String> {
    let query = query.trim().to_lowercase();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }

    let terms = parse_terms(&query);
    if terms.text.is_empty() {
        return Ok(Vec::new());
    }

    let limit = limit.unwrap_or(DEFAULT_RESULT_LIMIT).clamp(1, 100);
    let options = file_search_options(app);
    let mut files = platform_search(&terms.text_query(), limit.saturating_mul(10).max(100))?;
    files = filter_files(files, &terms, &options);
    files.sort_by(|a, b| compare_files(a, b, &terms));
    files.dedup_by(|a, b| paths_equal(&a.path, &b.path));

    Ok(files
        .into_iter()
        .take(limit)
        .map(launchable_item_from_file)
        .collect())
}

#[derive(Default)]
struct ParsedTerms {
    text: Vec<String>,
    extension: Option<String>,
    kind: Option<SearchKind>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchKind {
    File,
    Folder,
}

impl ParsedTerms {
    fn text_query(&self) -> String {
        self.text.join(" ")
    }
}

fn parse_terms(query: &str) -> ParsedTerms {
    let mut terms = ParsedTerms::default();

    for token in split_query_tokens(query) {
        if let Some(extension) = token
            .strip_prefix("ext:")
            .or_else(|| token.strip_prefix("extension:"))
            .and_then(normalize_extension)
        {
            terms.extension = Some(extension);
            continue;
        }

        if token.starts_with('.') {
            if let Some(extension) = normalize_extension(&token) {
                terms.extension = Some(extension);
            }
            continue;
        }

        if let Some(kind) = token
            .strip_prefix("type:")
            .or_else(|| token.strip_prefix("kind:"))
            .and_then(parse_kind)
        {
            terms.kind = Some(kind);
            continue;
        }

        terms.text.push(token);
    }

    terms
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

fn normalize_extension(extension: &str) -> Option<String> {
    let extension = extension.trim().trim_start_matches('.');
    if extension.is_empty() {
        return None;
    }

    Some(format!(".{}", extension.to_lowercase()))
}

fn parse_kind(kind: &str) -> Option<SearchKind> {
    match kind.trim() {
        "file" | "files" => Some(SearchKind::File),
        "folder" | "folders" | "dir" | "dirs" | "directory" | "directories" => {
            Some(SearchKind::Folder)
        }
        _ => None,
    }
}

fn filter_files(
    files: Vec<PlatformFile>,
    terms: &ParsedTerms,
    options: &FileSearchOptions,
) -> Vec<PlatformFile> {
    let mut seen = HashSet::new();

    files
        .into_iter()
        .filter(|file| seen.insert(normalize_path_key(&file.path)))
        .filter(|file| match terms.kind {
            Some(SearchKind::File) => !file.is_dir,
            Some(SearchKind::Folder) => file.is_dir,
            None => true,
        })
        .filter(|file| {
            terms
                .extension
                .as_ref()
                .map(|extension| &file.extension == extension)
                .unwrap_or(true)
        })
        .filter(|file| {
            let path = PathBuf::from(&file.path);
            is_path_allowed_by_options(&path, options)
        })
        .filter(|file| score_file(file, terms).is_some())
        .collect()
}

fn compare_files(a: &PlatformFile, b: &PlatformFile, terms: &ParsedTerms) -> Ordering {
    let score_a = score_file(a, terms).unwrap_or_default();
    let score_b = score_file(b, terms).unwrap_or_default();

    score_b
        .cmp(&score_a)
        .then_with(|| b.is_dir.cmp(&a.is_dir))
        .then_with(|| a.name.len().cmp(&b.name.len()))
        .then_with(|| a.name.cmp(&b.name))
}

fn score_file(file: &PlatformFile, terms: &ParsedTerms) -> Option<i32> {
    let name = file.name.to_lowercase();
    let parent = file.parent.to_lowercase();
    let path = file.path.to_lowercase();
    let mut score = if file.is_dir { 10 } else { 0 };

    for text in &terms.text {
        if name == *text {
            score += 100;
        } else if name.starts_with(text) {
            score += 75;
        } else if name.contains(text) {
            score += 45;
        } else if parent.contains(text) {
            score += 15;
        } else if path.contains(text) {
            score += 5;
        } else {
            return None;
        }
    }

    Some(score)
}

fn launchable_item_from_file(file: PlatformFile) -> LaunchableItem {
    LaunchableItem {
        name: file.name.clone(),
        description: Some(file.parent.clone()),
        keywords: vec![CommandKeyword {
            name: file.name,
            disabled: None,
            is_default: Some(true),
        }],
        path: file.path,
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

fn platform_search(query: &str, limit: usize) -> Result<Vec<PlatformFile>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_os = "windows")]
    {
        search_windows(query, limit)
    }
    #[cfg(target_os = "macos")]
    {
        search_macos(query, limit)
    }
    #[cfg(target_os = "linux")]
    {
        search_linux(query, limit)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (query, limit);
        Err("当前平台没有可用的系统文件搜索后端".to_string())
    }
}

#[cfg(target_os = "windows")]
#[derive(Deserialize)]
struct WindowsSearchRow {
    #[serde(rename = "Url")]
    url: String,
}

#[cfg(target_os = "windows")]
fn search_windows(query: &str, limit: usize) -> Result<Vec<PlatformFile>, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$query = $env:ONIN_FILE_SEARCH_QUERY
$limit = [Math]::Max(1, [Math]::Min(1000, [int]$env:ONIN_FILE_SEARCH_LIMIT))
$like = $query.Replace("'", "''").Replace('[', '[[]').Replace('%', '[%]').Replace('_', '[_]')
$connection = New-Object System.Data.OleDb.OleDbConnection("Provider=Search.CollatorDSO;Extended Properties='Application=Windows';")
$connection.Open()
try {
  $command = $connection.CreateCommand()
  $command.CommandText = "SELECT TOP $limit System.ItemUrl FROM SYSTEMINDEX WHERE System.FileName LIKE '%$like%'"
  $adapter = New-Object System.Data.OleDb.OleDbDataAdapter($command)
  $table = New-Object System.Data.DataTable
  [void]$adapter.Fill($table)
  $rows = foreach ($row in $table.Rows) {
    [PSCustomObject]@{ Url = [string]$row.'System.ItemUrl' }
  }
  ConvertTo-Json -InputObject @($rows) -Compress
} finally {
  $connection.Close()
}
"#;

    let output = Command::new(windows_powershell_path())
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("ONIN_FILE_SEARCH_QUERY", query)
        .env("ONIN_FILE_SEARCH_LIMIT", limit.to_string())
        .output()
        .map_err(|error| format!("Windows Search 查询启动失败: {}", error))?;

    if !output.status.success() {
        return Err(format!(
            "Windows Search 查询失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<WindowsSearchRow> =
        serde_json::from_str(stdout.trim()).map_err(|error| error.to_string())?;

    Ok(rows
        .into_iter()
        .filter_map(|row| path_from_windows_item_url(&row.url))
        .filter_map(|path| platform_file_from_path(&path))
        .collect())
}

#[cfg(target_os = "windows")]
fn path_from_windows_item_url(item_url: &str) -> Option<PathBuf> {
    if let Ok(url) = url::Url::parse(item_url) {
        if let Ok(path) = url.to_file_path() {
            return Some(normalize_windows_search_path(path));
        }
    }

    item_url
        .strip_prefix("file:")
        .map(|path| normalize_windows_search_path(PathBuf::from(path.replace('/', "\\"))))
}

#[cfg(target_os = "windows")]
fn normalize_windows_search_path(path: PathBuf) -> PathBuf {
    if path.exists() {
        return path;
    }

    let raw = path.to_string_lossy();
    if let Some(rest) = raw.strip_prefix(r"\C:\") {
        return PathBuf::from(format!(r"C:\{}", rest));
    }

    path
}

#[cfg(target_os = "windows")]
fn windows_powershell_path() -> PathBuf {
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let powershell_path = system_root
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");

    if powershell_path.exists() {
        powershell_path
    } else {
        PathBuf::from("powershell.exe")
    }
}

#[cfg(target_os = "macos")]
fn search_macos(query: &str, limit: usize) -> Result<Vec<PlatformFile>, String> {
    let query = format!("kMDItemFSName == '*{}*'cd", query.replace('\'', "\\'"));
    let output = Command::new("mdfind")
        .args(["-0", &query])
        .output()
        .map_err(|error| format!("Spotlight 查询启动失败: {}", error))?;

    if !output.status.success() {
        return Err(format!(
            "Spotlight 查询失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(split_nul_paths(&output.stdout)
        .into_iter()
        .take(limit)
        .filter_map(|path| platform_file_from_path(&path))
        .collect())
}

#[cfg(target_os = "linux")]
fn search_linux(query: &str, limit: usize) -> Result<Vec<PlatformFile>, String> {
    let command = if command_exists("plocate") {
        "plocate"
    } else if command_exists("locate") {
        "locate"
    } else {
        return Err("未找到 plocate 或 locate 文件搜索后端".to_string());
    };

    let output = Command::new(command)
        .args(["-0", "-i", "-l", &limit.to_string(), query])
        .output()
        .map_err(|error| format!("{} 查询启动失败: {}", command, error))?;

    if !output.status.success() {
        return Err(format!(
            "{} 查询失败: {}",
            command,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(split_nul_paths(&output.stdout)
        .into_iter()
        .filter_map(|path| platform_file_from_path(&path))
        .collect())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn split_nul_paths(bytes: &[u8]) -> Vec<PathBuf> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .filter_map(|part| std::str::from_utf8(part).ok())
        .map(PathBuf::from)
        .collect()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

fn paths_equal(a: &str, b: &str) -> bool {
    normalize_path_key(a) == normalize_path_key(b)
}

fn normalize_path_key(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        path.replace('/', "\\").to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string()
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::path_from_windows_item_url;

    #[test]
    fn windows_item_url_parses_drive_paths() {
        let path = path_from_windows_item_url("file:C:/Users/Administrator/logseq").unwrap();
        assert_eq!(path.to_string_lossy(), r"C:\Users\Administrator\logseq");
    }
}
