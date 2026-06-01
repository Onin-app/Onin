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

/// 扫描所有支持的浏览器书签
fn scan_all_bookmarks() -> Vec<BookmarkItem> {
    let mut all_bookmarks = Vec::new();

    // 1. 扫描 Chromium 系浏览器书签
    let chromium_browsers = vec!["Chrome", "Edge", "Brave", "Arc", "Opera", "Vivaldi"];

    for name in chromium_browsers {
        let paths = get_browser_bookmark_path(name);
        for path in paths {
            if path.exists() {
                if let Ok(bookmarks) = parse_chromium_bookmarks(&path, name) {
                    all_bookmarks.extend(bookmarks);
                    break; // 如果成功扫描了某个路径，则跳过此浏览器的其他备用路径
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
// Chromium 书签解析 (JSON)
// ============================================================================

fn parse_chromium_bookmarks(
    path: &Path,
    browser_name: &str,
) -> Result<Vec<BookmarkItem>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let val: serde_json::Value = serde_json::from_str(&content)?;
    let mut bookmarks = Vec::new();

    if let Some(roots) = val.get("roots") {
        // Chromium 书签通常有 bookmark_bar, other, synced 等根节点
        let root_keys = vec!["bookmark_bar", "other", "synced"];
        for key in root_keys {
            if let Some(root_node) = roots.get(key) {
                traverse_chromium_node(root_node, "", browser_name, &mut bookmarks);
            }
        }
    }

    Ok(bookmarks)
}

fn traverse_chromium_node(
    node: &serde_json::Value,
    current_folder: &str,
    browser_name: &str,
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
                traverse_chromium_node(child, &new_folder, browser_name, results);
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

    let rows = stmt.query_map([], |row| {
        let title: Option<String> = row.get(0)?;
        let url: String = row.get(1)?;
        let folder: Option<String> = row.get(2)?;

        Ok(BookmarkItem {
            title: title.unwrap_or_default(),
            url,
            browser: "Firefox".to_string(),
            folder: folder.unwrap_or_default(),
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
// 浏览器路径探测工具
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
                "Chrome" => paths.push(app_support.join("Google/Chrome/Default/Bookmarks")),
                "Edge" => paths.push(app_support.join("Microsoft Edge/Default/Bookmarks")),
                "Brave" => {
                    paths.push(app_support.join("BraveSoftware/Brave-Browser/Default/Bookmarks"))
                }
                "Arc" => paths.push(app_support.join("Arc/User Data/Default/Bookmarks")),
                "Safari" => paths.push(home.join("Library/Safari/Bookmarks.plist")),
                "Opera" => paths.push(app_support.join("com.operasoftware.Opera/Bookmarks")),
                "Vivaldi" => paths.push(app_support.join("Vivaldi/Default/Bookmarks")),
                "Firefox" => paths.push(app_support.join("Firefox/Profiles")),
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
                    paths.push(l.join("Google/Chrome/User Data/Default/Bookmarks"));
                }
            }
            "Edge" => {
                if let Some(ref l) = local_app_data {
                    paths.push(l.join("Microsoft/Edge/User Data/Default/Bookmarks"));
                }
            }
            "Brave" => {
                if let Some(ref l) = local_app_data {
                    paths.push(l.join("BraveSoftware/Brave-Browser/User Data/Default/Bookmarks"));
                }
            }
            "Arc" => {
                if let Some(ref l) = local_app_data {
                    paths.push(l.join("Arc/User Data/Bookmarks"));
                    paths.push(l.join("Arc/User Data/Default/Bookmarks"));
                }
            }
            "Opera" => {
                if let Some(ref a) = app_data {
                    paths.push(a.join("Opera Software/Opera Stable/Bookmarks"));
                }
                if let Some(ref l) = local_app_data {
                    paths.push(l.join("Opera Software/Opera Stable/Bookmarks"));
                }
            }
            "Vivaldi" => {
                if let Some(ref l) = local_app_data {
                    paths.push(l.join("Vivaldi/User Data/Default/Bookmarks"));
                }
            }
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
            println!(
                "  [{}] {} ({}) - Folder: {}",
                i, b.title, b.browser, b.folder
            );
        }
    }
}
