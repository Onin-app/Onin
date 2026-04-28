use std::{
    cmp::Ordering,
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use serde::Deserialize;
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey, HKEY,
};

use crate::shared_types::{CommandKeyword, IconType, ItemSource, ItemType, LaunchableItem};

use super::{
    path_utils::{file_search_options, is_path_allowed_by_options, platform_file_from_path},
    types::{FileSearchOptions, PlatformFile, DEFAULT_RESULT_LIMIT},
};

pub(super) fn backend_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        if everything_installed() {
            return "Everything";
        }

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

pub(super) fn everything_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        find_everything_exe_path().is_some()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub(super) fn everything_ipc_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        everything_ipc_window_available()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub(super) fn everything_install_required() -> bool {
    #[cfg(target_os = "windows")]
    {
        !everything_installed()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub(super) fn install_everything_backend() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        install_everything_with_winget()?;

        let path = find_everything_exe_path()
            .ok_or_else(|| "Everything 安装完成后仍未找到 Everything.exe".to_string())?;
        start_everything_client(&path)?;

        for _ in 0..30 {
            thread::sleep(Duration::from_millis(200));
            if everything_ipc_window_available() {
                return Ok(());
            }
        }

        Err("Everything 已安装，但 IPC 尚未就绪，请稍后再试".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("当前平台不支持自动安装 Everything".to_string())
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
    let mut files = platform_search(
        &terms.text_query(),
        limit.saturating_mul(10).max(100),
        &options,
    )?;
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

fn platform_search(
    query: &str,
    limit: usize,
    options: &FileSearchOptions,
) -> Result<Vec<PlatformFile>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(target_os = "windows")]
    {
        search_windows(query, limit, options)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = options;
        search_macos(query, limit)
    }
    #[cfg(target_os = "linux")]
    {
        let _ = options;
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
fn search_windows(
    query: &str,
    limit: usize,
    options: &FileSearchOptions,
) -> Result<Vec<PlatformFile>, String> {
    match search_everything_ipc(query, limit, options) {
        Ok(files) => return Ok(files),
        Err(error) => {
            tracing::warn!(backend = "Everything", %error, "file search backend failed");
        }
    }

    match search_everything_cli(query, limit, options) {
        Ok(files) => return Ok(files),
        Err(error) => {
            tracing::warn!(backend = "Everything CLI", %error, "file search backend failed");
        }
    }

    search_windows_search(query, limit)
}

#[cfg(target_os = "windows")]
fn search_everything_ipc(
    query: &str,
    limit: usize,
    options: &FileSearchOptions,
) -> Result<Vec<PlatformFile>, String> {
    use everything_ipc::wm::{RequestFlags, Sort};

    let everything = everything_client_or_start()?;
    let request_flags = RequestFlags::FileName | RequestFlags::Path;
    let mut files = Vec::new();

    for search_query in everything_queries(query, options) {
        let list = everything
            .query_wait(&search_query)
            .request_flags(request_flags)
            .sort(Sort::NameAscending)
            .max_results(limit as u32)
            .timeout(std::time::Duration::from_millis(700))
            .call()
            .map_err(|error| error.to_string())?;

        for item in list.iter() {
            let Some(name) = item.get_string(RequestFlags::FileName) else {
                continue;
            };
            let Some(parent) = item.get_string(RequestFlags::Path) else {
                continue;
            };

            let path = PathBuf::from(parent).join(name);
            if let Some(file) = platform_file_from_path(&path) {
                files.push(file);
            }

            if files.len() >= limit {
                return Ok(files);
            }
        }
    }

    Ok(files)
}

#[cfg(target_os = "windows")]
fn everything_client_or_start() -> Result<everything_ipc::wm::EverythingClient, String> {
    use everything_ipc::wm::EverythingClient;

    if let Ok(client) = EverythingClient::new() {
        return Ok(client);
    }

    let Some(path) = find_everything_exe_path() else {
        return Err("Everything IPC 不可用，且未找到 Everything.exe".to_string());
    };

    start_everything_client(&path)?;

    for _ in 0..15 {
        thread::sleep(Duration::from_millis(100));
        if let Ok(client) = EverythingClient::new() {
            return Ok(client);
        }
    }

    Err(format!(
        "已启动 Everything 客户端，但 IPC 窗口仍不可用: {}",
        path.display()
    ))
}

#[cfg(target_os = "windows")]
fn start_everything_client(path: &Path) -> Result<(), String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    Command::new(path)
        .arg("-startup")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("启动 Everything 客户端失败: {}", error))
}

#[cfg(target_os = "windows")]
fn find_everything_exe_path() -> Option<PathBuf> {
    find_everything_exe_path_from_registry().or_else(|| {
        [
            r"C:\Program Files\Everything\Everything.exe",
            r"C:\Program Files (x86)\Everything\Everything.exe",
        ]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
    })
}

#[cfg(target_os = "windows")]
fn find_everything_exe_path_from_registry() -> Option<PathBuf> {
    const UNINSTALL_PATHS: &[(&str, HKEY)] = &[
        (
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            HKEY_CURRENT_USER,
        ),
        (
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            HKEY_LOCAL_MACHINE,
        ),
        (
            "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            HKEY_LOCAL_MACHINE,
        ),
    ];

    for (path, hive) in UNINSTALL_PATHS {
        let root = RegKey::predef(*hive);
        let Ok(uninstall_key) = root.open_subkey(path) else {
            continue;
        };

        for item in uninstall_key.enum_keys().filter_map(Result::ok) {
            let Ok(subkey) = uninstall_key.open_subkey(item) else {
                continue;
            };

            let display_name = subkey.get_value::<String, _>("DisplayName").ok();
            if !display_name
                .as_deref()
                .map(|name| name.to_lowercase().contains("everything"))
                .unwrap_or(false)
            {
                continue;
            }

            for value_name in ["DisplayIcon", "InstallLocation"] {
                let Some(path) = subkey
                    .get_value::<String, _>(value_name)
                    .ok()
                    .and_then(|value| everything_exe_from_registry_value(&value))
                else {
                    continue;
                };

                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn everything_exe_from_registry_value(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);
    if path.is_dir() {
        return Some(path.join("Everything.exe"));
    }

    if path
        .file_name()
        .map(|name| {
            name.to_string_lossy()
                .eq_ignore_ascii_case("Everything.exe")
        })
        .unwrap_or(false)
    {
        return Some(path);
    }

    let before_comma = trimmed.split(',').next().unwrap_or(trimmed).trim();
    let path = PathBuf::from(before_comma);
    if path
        .file_name()
        .map(|name| {
            name.to_string_lossy()
                .eq_ignore_ascii_case("Everything.exe")
        })
        .unwrap_or(false)
    {
        return Some(path);
    }

    None
}

#[cfg(target_os = "windows")]
fn search_everything_cli(
    query: &str,
    limit: usize,
    options: &FileSearchOptions,
) -> Result<Vec<PlatformFile>, String> {
    let mut files = Vec::new();
    for search_query in everything_queries(query, options) {
        let output = Command::new("es.exe")
            .args(["-n", &limit.to_string(), &search_query])
            .output()
            .map_err(|error| format!("Everything CLI 查询启动失败: {}", error))?;

        if !output.status.success() {
            return Err(format!(
                "Everything CLI 查询失败: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let path = PathBuf::from(line.trim());
            if let Some(file) = platform_file_from_path(&path) {
                files.push(file);
            }

            if files.len() >= limit {
                return Ok(files);
            }
        }
    }

    Ok(files)
}

#[cfg(target_os = "windows")]
fn search_windows_search(query: &str, limit: usize) -> Result<Vec<PlatformFile>, String> {
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
fn everything_ipc_window_available() -> bool {
    everything_ipc::IpcWindow::new()
        .map(|window| window.is_ipc_available() && window.is_db_loaded())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn install_everything_with_winget() -> Result<(), String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("winget")
        .args([
            "install",
            "--id",
            "voidtools.Everything",
            "--exact",
            "--source",
            "winget",
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| {
            format!(
                "启动 winget 失败: {}。请确认系统已安装 App Installer，或手动安装 Everything。",
                error
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "winget 安装 Everything 失败: {}{}",
        stdout.trim(),
        stderr.trim()
    ))
}

#[cfg(target_os = "windows")]
fn everything_queries(query: &str, options: &FileSearchOptions) -> Vec<String> {
    let query = query.trim();
    let roots = options
        .roots
        .iter()
        .filter(|root| root.exists())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        return vec![query.to_string()];
    }

    roots
        .into_iter()
        .map(|root| {
            let root = everything_path_scope(root);
            if root.is_empty() {
                query.to_string()
            } else {
                format!("{} {}", root, query)
            }
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn everything_path_scope(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('/', "\\");
    let normalized = normalized.trim_end_matches('\\');
    if normalized.is_empty() {
        return String::new();
    }

    let scoped = format!(r"{}\", normalized.replace('"', r#"\""#));
    if scoped.chars().any(char::is_whitespace) {
        format!("\"{}\"", scoped)
    } else {
        scoped
    }
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
