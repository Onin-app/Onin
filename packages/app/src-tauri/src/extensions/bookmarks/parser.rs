use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub browser: String,
    pub folder: String,
    pub profile: Option<String>,
}

// 全局书签缓存
static BOOKMARK_CACHE: Lazy<Mutex<Vec<BookmarkItem>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// 获取所有浏览器的书签，如果缓存为空则进行扫描
pub fn get_bookmarks(force_reload: bool) -> Vec<BookmarkItem> {
    if !force_reload {
        let cache = BOOKMARK_CACHE.lock().unwrap();
        if !cache.is_empty() {
            return cache.clone();
        }
    }

    // 缓存为空或强制重新加载，进行扫描
    let bookmarks = scan_all_bookmarks();
    let mut cache = BOOKMARK_CACHE.lock().unwrap();
    *cache = bookmarks.clone();
    bookmarks
}

/// 探测 Chromium 浏览器的用户配置（Profile）目录结构
struct ChromiumProfileInfo {
    path: PathBuf, // Bookmarks 文件的绝对路径
    name: String,  // Profile 目录的名称，如 "Default", "Profile 1" 等
}

/// 扫描所有支持的浏览器书签
fn scan_all_bookmarks() -> Vec<BookmarkItem> {
    let mut all_bookmarks = Vec::new();

    // 1. 扫描 Chromium 系浏览器书签
    let chromium_browsers = vec!["Chrome", "Edge", "Brave", "Arc", "Opera", "Vivaldi"];

    for name in chromium_browsers {
        let user_data_dirs = get_chromium_user_data_dirs(name);
        for dir in user_data_dirs {
            if dir.exists() {
                let profiles = find_chromium_profiles(&dir);
                for profile in profiles {
                    if let Ok(bookmarks) =
                        parse_chromium_bookmarks(&profile.path, name, &profile.name)
                    {
                        all_bookmarks.extend(bookmarks);
                    }
                }
            }
        }
    }

    // 2. 仅在 macOS 上扫描 Safari 书签
    #[cfg(target_os = "macos")]
    {
        let safari_paths = get_browser_bookmark_path("Safari");
        for path in safari_paths {
            if path.exists() {
                if let Ok(bookmarks) = parse_safari_bookmarks(&path) {
                    all_bookmarks.extend(bookmarks);
                }
            }
        }
    }

    // 3. 扫描 Firefox 书签
    let firefox_paths = get_browser_bookmark_path("Firefox");
    for profiles_dir in firefox_paths {
        if profiles_dir.exists() {
            // 使用 walkdir 遍历 profiles 目录以查找所有的 places.sqlite
            for entry in walkdir::WalkDir::new(&profiles_dir)
                .max_depth(3) // places.sqlite 通常在 Profiles/xxxx.default/ 这一层
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_name() == "places.sqlite" {
                    if let Ok(bookmarks) = parse_firefox_bookmarks(entry.path()) {
                        all_bookmarks.extend(bookmarks);
                    }
                }
            }
        }
    }

    all_bookmarks
}

// ============================================================================
// Chromium 书签解析与多 Profile 动态扫描 (JSON)
// ============================================================================

/// 获取 Chromium 系列浏览器的用户数据（User Data）根目录
///
/// 注意：对于 Arc 浏览器，其在 macOS 上的用户数据存放在 `Arc/User Data` 目录下，
/// 而其它 Chromium 浏览器通常直接存放在各自的应用支持根目录下，此处针对 Arc 进行了定向适配，
/// 以保证后续 `find_chromium_profiles` 寻找 `Default` 或 `Profile *` 子目录的逻辑保持一致。
fn get_chromium_user_data_dirs(browser_name: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = get_home_dir() {
            let app_support = home.join("Library/Application Support");
            match browser_name {
                "Chrome" => dirs.push(app_support.join("Google/Chrome")),
                "Edge" => dirs.push(app_support.join("Microsoft Edge")),
                "Brave" => dirs.push(app_support.join("BraveSoftware/Brave-Browser")),
                "Arc" => dirs.push(app_support.join("Arc/User Data")),
                "Opera" => dirs.push(app_support.join("com.operasoftware.Opera")),
                "Vivaldi" => dirs.push(app_support.join("Vivaldi")),
                _ => {}
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from);
        let app_data = std::env::var("APPDATA").ok().map(PathBuf::from);

        match browser_name {
            "Chrome" => {
                if let Some(ref l) = local_app_data {
                    dirs.push(l.join("Google/Chrome/User Data"));
                }
            }
            "Edge" => {
                if let Some(ref l) = local_app_data {
                    dirs.push(l.join("Microsoft/Edge/User Data"));
                }
            }
            "Brave" => {
                if let Some(ref l) = local_app_data {
                    dirs.push(l.join("BraveSoftware/Brave-Browser/User Data"));
                }
            }
            "Arc" => {
                if let Some(ref l) = local_app_data {
                    // Arc 浏览器在 Windows 上的用户数据存放在 Arc/User Data 目录下，已通过实机验证确保路径无误
                    dirs.push(l.join("Arc/User Data"));
                }
            }
            "Opera" => {
                if let Some(ref a) = app_data {
                    dirs.push(a.join("Opera Software/Opera Stable"));
                }
                if let Some(ref l) = local_app_data {
                    dirs.push(l.join("Opera Software/Opera Stable"));
                }
            }
            "Vivaldi" => {
                if let Some(ref l) = local_app_data {
                    dirs.push(l.join("Vivaldi/User Data"));
                }
            }
            _ => {}
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = get_home_dir() {
            let config_dir = home.join(".config");
            match browser_name {
                "Chrome" => dirs.push(config_dir.join("google-chrome")),
                "Edge" => dirs.push(config_dir.join("microsoft-edge")),
                "Brave" => dirs.push(config_dir.join("BraveSoftware/Brave-Browser")),
                "Arc" => dirs.push(config_dir.join("Arc")),
                "Opera" => dirs.push(config_dir.join("opera")),
                "Vivaldi" => dirs.push(config_dir.join("vivaldi")),
                _ => {}
            }
        }
    }

    dirs
}

/// 在指定 Chromium 浏览器的 User Data 目录下动态搜寻所有 Profile 中的 Bookmarks 文件
fn find_chromium_profiles(user_data_dir: &Path) -> Vec<ChromiumProfileInfo> {
    let mut profiles = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. 检查根目录下是否有 Bookmarks (有些浏览器如 Arc 或精简单用户版会直接存放在根目录)
    let root_bookmarks = user_data_dir.join("Bookmarks");
    if root_bookmarks.exists() && root_bookmarks.is_file() {
        // seen_paths 使用 canonicalize() 规范化绝对路径实现精准去重。
        // 因前置了 root_bookmarks.exists() 检查，canonicalize() 必然成功，故不存在无法规范化导致重复解析的隐患。
        if let Ok(abs_path) = root_bookmarks.canonicalize() {
            seen_paths.insert(abs_path.clone());
            profiles.push(ChromiumProfileInfo {
                path: abs_path,
                name: "Default".to_string(),
            });
        }
    }

    // 2. 遍历一级子目录，寻找包含 Bookmarks 的合法 Profile 目录（如 Default, Profile 1, Profile 2 等）
    if let Ok(entries) = std::fs::read_dir(user_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let bookmarks_path = path.join("Bookmarks");
                if bookmarks_path.exists() && bookmarks_path.is_file() {
                    if let Ok(abs_path) = bookmarks_path.canonicalize() {
                        if seen_paths.contains(&abs_path) {
                            continue;
                        }
                        seen_paths.insert(abs_path.clone());

                        let dir_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Default")
                            .to_string();

                        // 排除一些系统和无关的内置非用户 Profile
                        if dir_name != "System Profile" && dir_name != "Guest Profile" {
                            profiles.push(ChromiumProfileInfo {
                                path: abs_path,
                                name: dir_name,
                            });
                        }
                    }
                }
            }
        }
    }

    profiles
}

fn parse_chromium_bookmarks(
    path: &Path,
    browser_name: &str,
    profile_name: &str,
) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let val: serde_json::Value = serde_json::from_str(&content)?;
    let mut bookmarks = Vec::new();

    if let Some(roots) = val.get("roots") {
        // Chromium 书签通常有 bookmark_bar, other, synced 等根节点
        let root_keys = vec!["bookmark_bar", "other", "synced"];
        for key in root_keys {
            if let Some(root_node) = roots.get(key) {
                traverse_chromium_node(root_node, "", browser_name, profile_name, &mut bookmarks);
            }
        }
    }

    Ok(bookmarks)
}

fn traverse_chromium_node(
    node: &serde_json::Value,
    current_folder: &str,
    browser_name: &str,
    profile_name: &str,
    results: &mut Vec<BookmarkItem>,
) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let name = node
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    if node_type == "url" {
        if let Some(url) = node.get("url").and_then(|u| u.as_str()) {
            results.push(BookmarkItem {
                title: name,
                url: url.to_string(),
                browser: browser_name.to_string(),
                folder: current_folder.to_string(),
                profile: Some(profile_name.to_string()),
            });
        }
    } else if node_type == "folder" {
        let new_folder = if current_folder.is_empty() {
            name
        } else {
            format!("{}/{}", current_folder, name)
        };

        if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
            for child in children {
                traverse_chromium_node(child, &new_folder, browser_name, profile_name, results);
            }
        }
    }
}

// ============================================================================
// Safari 书签解析 (Binary Plist, macOS 专属)
// ============================================================================

#[cfg(target_os = "macos")]
fn parse_safari_bookmarks(path: &Path) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
    let plist_val = plist::Value::from_file(path)?;
    let mut bookmarks = Vec::new();
    traverse_safari_node(&plist_val, "", &mut bookmarks);
    Ok(bookmarks)
}

#[cfg(target_os = "macos")]
fn traverse_safari_node(
    node: &plist::Value,
    current_folder: &str,
    results: &mut Vec<BookmarkItem>,
) {
    if let Some(dict) = node.as_dictionary() {
        let type_str = dict
            .get("WebBookmarkType")
            .and_then(|t| t.as_string())
            .unwrap_or("");

        if type_str == "WebBookmarkTypeLeaf" {
            // 叶子节点 (书签)
            let title = dict
                .get("URIDictionary")
                .and_then(|uri_dict| uri_dict.as_dictionary())
                .and_then(|d| d.get("title"))
                .and_then(|t| t.as_string())
                .or_else(|| dict.get("title").and_then(|t| t.as_string()))
                .unwrap_or("")
                .to_string();

            let url = dict
                .get("URLString")
                .and_then(|u| u.as_string())
                .unwrap_or("");

            if !url.is_empty() {
                results.push(BookmarkItem {
                    title,
                    url: url.to_string(),
                    browser: "Safari".to_string(),
                    folder: current_folder.to_string(),
                    // Safari 在较新 macOS 下虽然增加了概念，但其数据依然集中存储在 Bookmarks.plist 根下，
                    // 不支持主流 Chromium/Firefox 概念的多分身目录，此处统一设定为 "Default" 以保持系统一致性。
                    profile: Some("Default".to_string()),
                });
            }
        } else if type_str == "WebBookmarkTypeFolder" {
            // 文件夹节点
            let title = dict
                .get("Title")
                .and_then(|t| t.as_string())
                .unwrap_or("")
                .to_string();
            // 忽略一些系统内置大分类夹的名称显示以美化路径
            let new_folder =
                if title.is_empty() || title == "BookmarksBar" || title == "BookmarksMenu" {
                    current_folder.to_string()
                } else if current_folder.is_empty() {
                    title
                } else {
                    format!("{}/{}", current_folder, title)
                };

            if let Some(children) = dict.get("Children").and_then(|c| c.as_array()) {
                for child in children {
                    traverse_safari_node(child, &new_folder, results);
                }
            }
        }
    }
}

// ============================================================================
// Firefox 书签解析 (SQLite, places.sqlite)
// ============================================================================

fn parse_firefox_bookmarks(path: &Path) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
    // 提取 profile 文件夹名（例如 "xxxx.default-release"）
    // 基于 walkdir 遍历时设置的 max_depth(3) 边界条件限制，places.sqlite 必然位于 profile 根目录
    // 或其一级直接子目录下（如 Firefox/Profiles/xxxx.default/places.sqlite）。
    // 在此有限的探测深度下，提取 path.parent() 的 file_name() 作为 profile_name 是绝对安全且无歧义的。
    let mut profile_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Default")
        .to_string();

    // 规范化 Firefox 的默认 Profile，使之与 Chromium 的 "Default" 保持一致，从而在前端优雅隐藏
    if profile_name.contains(".default") {
        profile_name = "Default".to_string();
    }

    // 拷贝到临时文件，避免数据库文件锁冲突
    let temp_dir = std::env::temp_dir();
    let temp_db_path = temp_dir.join(format!(
        "onin_firefox_places_{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    std::fs::copy(path, &temp_db_path)?;

    let conn = rusqlite::Connection::open(&temp_db_path)?;
    // 在火狐的 places.sqlite 中，
    // - type = 1 代表 URL 书签
    // - b.parent 关联父书签目录
    // - h.url 是 URL 链接，排除 'place:%' 开头的内部查询 URL
    let mut stmt = conn.prepare(
        "SELECT b.title, h.url, p.title 
         FROM moz_bookmarks b
         JOIN moz_places h ON b.fk = h.id
         LEFT JOIN moz_bookmarks p ON b.parent = p.id
         WHERE b.type = 1 AND h.url IS NOT NULL AND h.url NOT LIKE 'place:%'",
    )?;

    let profile_name_clone = profile_name.clone();
    let rows = stmt.query_map([], move |row| {
        let title: Option<String> = row.get(0)?;
        let url: String = row.get(1)?;
        let folder: Option<String> = row.get(2)?;

        Ok(BookmarkItem {
            title: title.unwrap_or_default(),
            url,
            browser: "Firefox".to_string(),
            folder: folder.unwrap_or_default(),
            profile: Some(profile_name_clone.clone()),
        })
    })?;

    let mut bookmarks = Vec::new();
    for row in rows {
        if let Ok(bookmark) = row {
            bookmarks.push(bookmark);
        }
    }

    // 尝试清理临时文件
    let _ = std::fs::remove_file(&temp_db_path);

    Ok(bookmarks)
}

// ============================================================================
// 浏览器路径探测工具 (Firefox / Safari 兼容)
// ============================================================================

#[cfg(target_os = "macos")]
fn get_home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home))
    } else {
        None
    }
}

fn get_browser_bookmark_path(browser_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = get_home_dir() {
            let app_support = home.join("Library/Application Support");
            match browser_name {
                "Safari" => paths.push(home.join("Library/Safari/Bookmarks.plist")),
                "Firefox" => paths.push(app_support.join("Firefox/Profiles")),
                _ => {}
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA").ok().map(PathBuf::from);

        match browser_name {
            "Firefox" => {
                if let Some(ref a) = app_data {
                    paths.push(a.join("Mozilla/Firefox/Profiles"));
                }
            }
            _ => {}
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_bookmarks() {
        println!("==> Starting scanning bookmarks...");
        let bookmarks = scan_all_bookmarks();
        println!(
            "==> Scan finished! Total bookmarks found: {}",
            bookmarks.len()
        );
        for (i, b) in bookmarks.iter().take(5).enumerate() {
            let prof = b.profile.as_deref().unwrap_or("None");
            println!(
                "  [{}] {} ({}) - Profile: {} - Folder: {}",
                i, b.title, b.browser, prof, b.folder
            );
        }
    }
}
