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
    let chromium_browsers = vec![
        ("Chrome", get_chrome_path()),
        ("Edge", get_edge_path()),
        ("Brave", get_brave_path()),
        ("Arc", get_arc_path()),
    ];

    for (name, path_opt) in chromium_browsers {
        if let Some(path) = path_opt {
            if path.exists() {
                if let Ok(bookmarks) = parse_chromium_bookmarks(&path, name) {
                    all_bookmarks.extend(bookmarks);
                }
            }
        }
    }

    // 2. 仅在 macOS 上扫描 Safari 书签
    #[cfg(target_os = "macos")]
    {
        if let Some(safari_path) = get_safari_path() {
            if safari_path.exists() {
                if let Ok(bookmarks) = parse_safari_bookmarks(&safari_path) {
                    all_bookmarks.extend(bookmarks);
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

fn get_chrome_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        get_home_dir()
            .map(|h| h.join("Library/Application Support/Google/Chrome/Default/Bookmarks"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|l| PathBuf::from(l).join("Google/Chrome/User Data/Default/Bookmarks"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn get_edge_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        get_home_dir()
            .map(|h| h.join("Library/Application Support/Microsoft Edge/Default/Bookmarks"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|l| PathBuf::from(l).join("Microsoft/Edge/User Data/Default/Bookmarks"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn get_brave_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        get_home_dir().map(|h| {
            h.join("Library/Application Support/BraveSoftware/Brave-Browser/Default/Bookmarks")
        })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok().map(|l| {
            PathBuf::from(l).join("BraveSoftware/Brave-Browser/User Data/Default/Bookmarks")
        })
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn get_arc_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        get_home_dir()
            .map(|h| h.join("Library/Application Support/Arc/User Data/Default/Bookmarks"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(|l| PathBuf::from(l).join("Arc/User Data/Default/Bookmarks"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn get_safari_path() -> Option<PathBuf> {
    get_home_dir().map(|h| h.join("Library/Safari/Bookmarks.plist"))
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
