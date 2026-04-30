use std::{
    cmp::Ordering,
    collections::HashSet,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
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
    path_utils::{
        file_search_options, is_path_allowed_by_options_fast, platform_file_from_path,
        platform_file_from_path_with_kind_and_modified_time,
    },
    types::{FileSearchOptions, FileSearchResponse, PlatformFile, DEFAULT_RESULT_LIMIT},
};

const MAX_PAGE_OFFSET: usize = 5_000;
const MAX_CANDIDATE_LIMIT: usize = 5_000;

pub(super) fn backend_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        if everything_ipc_available() {
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
    offset: Option<usize>,
    app: &AppHandle,
) -> Result<FileSearchResponse, String> {
    let query = query.trim().to_lowercase();
    let limit = limit.unwrap_or(DEFAULT_RESULT_LIMIT).clamp(1, 100);
    let offset = offset.unwrap_or(0).min(MAX_PAGE_OFFSET);

    if !is_search_query_long_enough(&query) {
        return Ok(empty_search_response(offset, limit));
    }

    let terms = parse_terms(&query);
    if terms.text.is_empty() {
        return Ok(empty_search_response(offset, limit));
    }

    let options = file_search_options(app);
    // 拉大候选池，避免 Everything 按字母序返回时把相关结果切掉
    let requested_end = offset.saturating_add(limit);
    let candidate_limit = requested_end
        .saturating_mul(6)
        .max(200)
        .min(MAX_CANDIDATE_LIMIT);
    let mut files = platform_search(&terms.text_query(), candidate_limit, &options)?;
    let candidate_limit_reached = files.len() >= candidate_limit;
    files = filter_files(files, &terms, &options);
    files.sort_by(|a, b| compare_files(a, b, &terms, &options));
    files.dedup_by(|a, b| paths_equal(&a.path, &b.path));

    let total_count = files.len();
    let items = files
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(launchable_item_from_file)
        .collect();

    Ok(FileSearchResponse {
        items,
        total_count,
        total_count_is_exact: !candidate_limit_reached,
        has_more: total_count > requested_end
            || (candidate_limit_reached && total_count >= requested_end),
        offset,
        limit,
    })
}

fn empty_search_response(offset: usize, limit: usize) -> FileSearchResponse {
    FileSearchResponse {
        items: Vec::new(),
        total_count: 0,
        total_count_is_exact: true,
        has_more: false,
        offset,
        limit,
    }
}

fn is_search_query_long_enough(query: &str) -> bool {
    query.chars().count() >= 2 || query.chars().any(is_cjk_search_character)
}

fn is_cjk_search_character(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
            | '\u{3040}'..='\u{309f}'
            | '\u{30a0}'..='\u{30ff}'
            | '\u{31f0}'..='\u{31ff}'
            | '\u{ac00}'..='\u{d7af}'
    )
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
            is_path_allowed_by_options_fast(&path, options)
        })
        .filter(|file| score_file(file, terms).is_some())
        .collect()
}

fn compare_files(
    a: &PlatformFile,
    b: &PlatformFile,
    terms: &ParsedTerms,
    options: &FileSearchOptions,
) -> Ordering {
    let score_a = score_file_with_options(a, terms, options).unwrap_or_default();
    let score_b = score_file_with_options(b, terms, options).unwrap_or_default();

    score_b
        .cmp(&score_a)
        .then_with(|| b.is_dir.cmp(&a.is_dir))
        .then_with(|| a.name.len().cmp(&b.name.len()))
        .then_with(|| a.name.cmp(&b.name))
}

fn score_file(file: &PlatformFile, terms: &ParsedTerms) -> Option<i32> {
    score_file_with_options(
        file,
        terms,
        &FileSearchOptions {
            roots: Vec::new(),
            preferred_paths: Vec::new(),
            excluded_paths: Vec::new(),
            include_hidden: false,
        },
    )
}

fn score_file_with_options(
    file: &PlatformFile,
    terms: &ParsedTerms,
    options: &FileSearchOptions,
) -> Option<i32> {
    let name = file.name.to_lowercase();
    let parent = file.parent.to_lowercase();
    let path = file.path.to_lowercase();
    let stem = Path::new(&file.name)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    // 目录基础分略高：搜索 src/config 等词时优先展示目录本身
    let mut score = if file.is_dir { 15 } else { 0 };

    for text in &terms.text {
        score += match_text_score(&name, &stem, &parent, &path, text)?;
    }

    score += preferred_path_bonus(file, options);

    Some(score)
}

fn match_text_score(name: &str, stem: &str, parent: &str, path: &str, text: &str) -> Option<i32> {
    if name == text {
        return Some(260);
    }

    if stem == text {
        return Some(245);
    }

    if name.starts_with(text) {
        return Some(220);
    }

    if stem.starts_with(text) {
        return Some(205);
    }

    if let Some(index) = word_boundary_index(name, text) {
        return Some(185 - index.min(25) as i32);
    }

    if let Some(index) = name.find(text) {
        return Some(160 - index.min(40) as i32);
    }

    // 规范化匹配放在 fuzzy 之前：
    // 对 stem（不含扩展名）做规范化，避免 .ts 等扩展名污染匹配结果。
    // 例如 my-project.ts 的 stem 是 my-project，规范化后是 myproject，精确命中得 130；
    // 若用 name 做规范化，myprojectts 只能走 starts_with 得 110。
    if stem.contains(['-', '_', '.']) {
        let normalized_stem = stem.replace(['-', '_', '.'], "");
        // 用户输入若也带分隔符（如 my-project），同样规范化后再比
        let normalized_text = text.replace(['-', '_'], "");
        if !normalized_text.is_empty() {
            if normalized_stem == normalized_text {
                return Some(130);
            }
            if normalized_stem.starts_with(&normalized_text) {
                return Some(110);
            }
            if let Some(index) = word_boundary_index(&normalized_stem, &normalized_text) {
                return Some(90 - index.min(20) as i32);
            }
        }
    }

    if let Some(score) = fuzzy_subsequence_score(name, text) {
        return Some(score);
    }

    if let Some(index) = parent.find(text) {
        return Some(45 - index.min(25) as i32);
    }

    if let Some(index) = path.find(text) {
        return Some(20 - index.min(15) as i32);
    }

    None
}

fn word_boundary_index(value: &str, text: &str) -> Option<usize> {
    // 注意：调用方传入的 value 和 text 均已 to_lowercase()，
    // 因此驼峰边界（大写首字符检测）在此无效，仅保留 ASCII 分隔符边界。
    value.match_indices(text).find_map(|(index, _)| {
        if index == 0 {
            return Some(index);
        }

        let prev_char = value[..index].chars().last()?;
        if matches!(prev_char, '-' | '_' | '.' | ' ' | '\\' | '/') {
            return Some(index);
        }

        None
    })
}

fn fuzzy_subsequence_score(value: &str, text: &str) -> Option<i32> {
    let mut last_index = None;
    let mut first_index = None;
    let mut gap_count = 0usize;
    let mut search_start = 0usize;

    for character in text.chars() {
        let relative_index = value[search_start..].find(character)?;
        let index = search_start + relative_index;

        if let Some(last_index) = last_index {
            gap_count += index.saturating_sub(last_index + 1);
        } else {
            first_index = Some(index);
        }

        last_index = Some(index);
        search_start = index + character.len_utf8();
    }

    Some(95 - first_index.unwrap_or(0).min(25) as i32 - gap_count.min(45) as i32)
}

fn preferred_path_bonus(file: &PlatformFile, options: &FileSearchOptions) -> i32 {
    let file_path = PathBuf::from(&file.path);

    options
        .preferred_paths
        .iter()
        .position(|preferred_path| path_is_inside(&file_path, preferred_path))
        // 上限从 35 提高到 50，确保偏好路径加成高于父目录匹配分数（最高 45）
        .map(|index| 50 - (index.min(4) as i32 * 5))
        .unwrap_or(0)
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
        modified_time: file.modified_time,
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

fn run_command_with_timeout(
    command: &mut Command,
    label: &str,
    timeout: Duration,
) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("{} 启动失败: {}", label, error))?;
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| format!("读取 {} 输出失败: {}", label, error));
            }
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait_with_output();
                return Err(format!("{} 超时，请稍后重试", label));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("等待 {} 结束失败: {}", label, error));
            }
        }
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
    let request_flags = RequestFlags::FileName
        | RequestFlags::Path
        | RequestFlags::Attributes
        | RequestFlags::DateModified;
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
            let is_dir = item
                .get_u32(RequestFlags::Attributes)
                .map(|attributes| attributes & 0x10 != 0)
                .unwrap_or(false);

            let path = PathBuf::from(parent).join(name);
            let modified_time = item
                .get_time(RequestFlags::DateModified)
                .and_then(|filetime| {
                    filetime_to_unix_millis(filetime.dwHighDateTime, filetime.dwLowDateTime)
                });

            if let Some(file) =
                platform_file_from_path_with_kind_and_modified_time(&path, is_dir, modified_time)
            {
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

    if let Err(error) = disable_everything_update_notification(path) {
        tracing::warn!(%error, "failed to disable Everything update notification before startup");
    }

    Command::new(path)
        .arg("-startup")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("启动 Everything 客户端失败: {}", error))
}

#[cfg(target_os = "windows")]
fn filetime_to_unix_millis(high: u32, low: u32) -> Option<u64> {
    const UNIX_EPOCH_FILETIME_TICKS: u64 = 116_444_736_000_000_000;
    const FILETIME_TICKS_PER_MILLISECOND: u64 = 10_000;

    let ticks = ((high as u64) << 32) | low as u64;
    ticks
        .checked_sub(UNIX_EPOCH_FILETIME_TICKS)
        .map(|unix_ticks| unix_ticks / FILETIME_TICKS_PER_MILLISECOND)
}

#[cfg(target_os = "windows")]
fn disable_everything_update_notification(path: &Path) -> Result<(), String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new(path);
    command
        .arg("-disable-update-notification")
        .creation_flags(CREATE_NO_WINDOW);

    let output = run_command_with_timeout(
        &mut command,
        "Everything 禁用启动更新提示",
        Duration::from_secs(2),
    )?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Everything 禁用启动更新提示失败: {}{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
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
        let mut command = Command::new("es.exe");
        command.args(["-n", &limit.to_string(), &search_query]);
        let output =
            run_command_with_timeout(&mut command, "Everything CLI 查询", Duration::from_secs(2))?;

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

    let mut command = Command::new(windows_powershell_path());
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("ONIN_FILE_SEARCH_QUERY", query)
        .env("ONIN_FILE_SEARCH_LIMIT", limit.to_string());
    let output =
        run_command_with_timeout(&mut command, "Windows Search 查询", Duration::from_secs(5))?;

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
    let mut command = Command::new("mdfind");
    command.args(["-0", &query]);
    let output = run_command_with_timeout(&mut command, "Spotlight 查询", Duration::from_secs(3))?;

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

    let mut search_command = Command::new(command);
    search_command.args(["-0", "-i", "-l", &limit.to_string(), query]);
    let output = run_command_with_timeout(
        &mut search_command,
        &format!("{} 查询", command),
        Duration::from_secs(3),
    )?;

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

fn path_is_inside(path: &Path, prefix: &Path) -> bool {
    let path = normalize_path_key(&path.to_string_lossy());
    let prefix = normalize_path_key(&prefix.to_string_lossy());
    let prefix = prefix.trim_end_matches(['\\', '/']);

    path == prefix
        || path
            .strip_prefix(prefix)
            .map(|rest| rest.starts_with('\\') || rest.starts_with('/'))
            .unwrap_or(false)
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

#[cfg(test)]
mod scoring_tests {
    use std::{cmp::Ordering, path::PathBuf};

    use super::{
        compare_files, is_search_query_long_enough, score_file_with_options, FileSearchOptions,
        ParsedTerms, PlatformFile,
    };

    fn file(path: &str, name: &str, parent: &str) -> PlatformFile {
        PlatformFile {
            name: name.to_string(),
            path: path.to_string(),
            parent: parent.to_string(),
            extension: PathBuf::from(name)
                .extension()
                .map(|extension| format!(".{}", extension.to_string_lossy().to_lowercase()))
                .unwrap_or_default(),
            is_dir: false,
            modified_time: None,
        }
    }

    fn terms(text: &str) -> ParsedTerms {
        ParsedTerms {
            text: vec![text.to_string()],
            extension: None,
            kind: None,
        }
    }

    fn options(preferred_paths: Vec<PathBuf>) -> FileSearchOptions {
        FileSearchOptions {
            roots: Vec::new(),
            preferred_paths,
            excluded_paths: Vec::new(),
            include_hidden: false,
        }
    }

    #[test]
    fn filename_exact_and_prefix_matches_rank_above_path_matches() {
        let terms = terms("config");
        let options = options(Vec::new());
        let exact = file("/project/config.ts", "config.ts", "/project");
        let prefix = file("/project/configuration.ts", "configuration.ts", "/project");
        let path_only = file("/project/config/app.ts", "app.ts", "/project/config");

        assert!(
            score_file_with_options(&exact, &terms, &options)
                > score_file_with_options(&prefix, &terms, &options)
        );
        assert!(
            score_file_with_options(&prefix, &terms, &options)
                > score_file_with_options(&path_only, &terms, &options)
        );
    }

    #[test]
    fn consecutive_matches_rank_above_sparse_fuzzy_matches() {
        let terms = terms("con");
        let options = options(Vec::new());
        let consecutive = file("/project/config.ts", "config.ts", "/project");
        let sparse = file("/project/c_o_n_file.ts", "c_o_n_file.ts", "/project");

        assert!(
            score_file_with_options(&consecutive, &terms, &options)
                > score_file_with_options(&sparse, &terms, &options)
        );
    }

    #[test]
    fn preferred_paths_can_break_close_matches_without_usage_storage() {
        let terms = terms("notes");
        let options = options(vec![PathBuf::from("/home/me/Documents")]);
        let preferred = file(
            "/home/me/Documents/notes.txt",
            "notes.txt",
            "/home/me/Documents",
        );
        let other = file("/tmp/notes.txt", "notes.txt", "/tmp");

        assert_eq!(
            compare_files(&preferred, &other, &terms, &options),
            Ordering::Less
        );
    }

    #[test]
    fn cjk_queries_can_search_with_one_character() {
        assert!(is_search_query_long_enough("文"));
        assert!(is_search_query_long_enough("あ"));
        assert!(is_search_query_long_enough("ア"));
        assert!(is_search_query_long_enough("한"));
    }

    #[test]
    fn ascii_queries_still_need_two_characters() {
        assert!(!is_search_query_long_enough("a"));
        assert!(!is_search_query_long_enough("1"));
        assert!(is_search_query_long_enough("ab"));
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
