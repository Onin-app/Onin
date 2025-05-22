use lnk::ShellLink;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::{RegKey, HKEY};

use super::AppInfo;

const UNINSTALL_PATHS: &[(&str, HKEY)] = &[
    (
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
    ),
    (
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_CURRENT_USER,
    ),
    (
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        HKEY_LOCAL_MACHINE,
    ),
];

// 从注册表中获取应用列表
pub fn get_apps_from_hkey() -> Result<Vec<AppInfo>, String> {
    let mut apps = vec![];

    for (path, hive) in UNINSTALL_PATHS {
        let root = RegKey::predef(*hive);
        if let Ok(uninstall_key) = root.open_subkey(path) {
            for item in uninstall_key.enum_keys().filter_map(Result::ok) {
                if let Ok(subkey) = uninstall_key.open_subkey(&item) {
                    // Skip if no DisplayName
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    if display_name.is_err() {
                        continue;
                    }

                    // Skip if SystemComponent == 1
                    let system_component: Result<u32, _> = subkey.get_value("SystemComponent");
                    if matches!(system_component, Ok(1)) {
                        continue;
                    }

                    // Skip if ParentKeyName or ParentDisplayName exists
                    let has_parent_key = subkey.get_value::<String, _>("ParentKeyName").is_ok();
                    let has_parent_display =
                        subkey.get_value::<String, _>("ParentDisplayName").is_ok();
                    if has_parent_key || has_parent_display {
                        continue;
                    }

                    let display_name = display_name.unwrap();
                    let display_icon = subkey.get_value::<String, _>("DisplayIcon").ok();
                    let install_location = subkey.get_value::<String, _>("InstallLocation").ok();
                    let uninstall_string = subkey.get_value::<String, _>("UninstallString").ok();

                    let path = extract_exe_path(display_icon, install_location, uninstall_string);

                    // 只有在path不为None时才添加到列表
                    if path.is_some() {
                        apps.push(AppInfo {
                            name: normalize_app_name(&display_name),
                            path,
                        });
                    }
                }
            }
        }
    }

    Ok(apps)
}

fn normalize_app_name(name: &str) -> String {
    // 特殊应用处理
    // if name.contains("Mozilla Firefox") {
    //     return "Firefox".to_string();
    // }

    // 常见需要移除的模式
    let patterns = [
        " (x86)",
        " (x64)",
        " (x86_64)",
        " [x86]",
        " [x64]",
        " [x86_64]",
        " en-US",
        " zh-CN",
    ];

    let mut result = name.to_string();
    for pattern in patterns {
        result = result.replace(pattern, "");
    }

    // 移除版本号 (匹配类似 1.06.2412050 的格式)
    let re = Regex::new(r"\s*\d+(\.\d+)+").unwrap();
    result = re.replace_all(&result, "").to_string();

    // 移除括号及其内容
    let re = Regex::new(r"\([^)]*\)").unwrap();
    result = re.replace_all(&result, "").to_string();

    // 移除方括号及其内容
    let re = Regex::new(r"\[[^\]]*\]").unwrap();
    result = re.replace_all(&result, "").to_string();

    result.trim().to_string()
}

fn extract_exe_path(
    display_icon: Option<String>,
    install_location: Option<String>,
    uninstall_string: Option<String>,
) -> Option<String> {
    // 1. 尝试清洗 DisplayIcon
    if let Some(icon) = display_icon {
        if icon.to_lowercase().ends_with(".exe") {
            return Some(icon);
        }
        if icon.to_lowercase().contains(".exe,") {
            let parts: Vec<&str> = icon.split(",").collect();
            if parts[0].to_lowercase().ends_with(".exe") {
                return Some(parts[0].to_string());
            }
        }
    }

    // 2. 从 InstallLocation 猜测路径
    if let Some(loc) = &install_location {
        let guessed = std::path::Path::new(loc).join("start.exe"); // 默认启动文件名示意
        if guessed.exists() {
            return Some(guessed.to_string_lossy().to_string());
        }
    }

    // 3. 尝试清洗 UninstallString
    if let Some(uninstall) = uninstall_string {
        // 抽取包含 .exe 的路径（排除带 uninstall 字样的）
        if uninstall.to_lowercase().contains(".exe") {
            let re = Regex::new("\"?([^\"]+?\\.exe)\"?").unwrap();
            if let Some(cap) = re.captures(&uninstall) {
                let path = &cap[1];
                if !path.to_lowercase().contains("uninstall") {
                    return Some(path.to_string());
                }
            }
        }
    }

    // 4. fallback：返回 None
    None
}

fn extract_target_from_lnk(path: &Path) -> Option<String> {
    let shell_link = ShellLink::open(path).ok()?;
    let link_info = shell_link.link_info().as_ref()?;
    let local_path = link_info.local_base_path().clone()?; // 这里添加.clone()

    // 确保路径存在
    if Path::new(&local_path).exists() {
        Some(local_path)
    } else {
        None
    }
}

fn get_all_shortcuts() -> Vec<PathBuf> {
    let mut shortcuts = Vec::new();
    let user_profile = std::env::var("USERPROFILE").unwrap();

    println!("user_profile -> {:?}", user_profile);

    let start_menu_paths = vec![
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs".to_string(),
        format!(
            "{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs",
            user_profile
        ),
        format!("{}\\Desktop", user_profile),
    ];

    for base in start_menu_paths {
        if !Path::new(&base).exists() {
            continue;
        }
        for entry in WalkDir::new(base).into_iter().filter_map(|e| e.ok()) {
            if entry
                .path()
                .extension()
                .map(|s| s == "lnk")
                .unwrap_or(false)
            {
                if !should_filter_shortcut(entry.path()) {
                    shortcuts.push(entry.path().to_path_buf());
                }
            }
        }
    }

    shortcuts
}

// 从快捷方式中获取应用列表
pub fn get_apps_from_shortcuts() -> Result<Vec<AppInfo>, String> {
    let shortcuts = get_all_shortcuts();
    let mut apps = Vec::new();

    for shortcut in shortcuts {
        if let Some(target_path) = extract_target_from_lnk(&shortcut) {
            let app_name = shortcut
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            apps.push(AppInfo {
                name: app_name,
                path: Some(target_path),
            });
        }
    }

    Ok(apps)
}

// 合并并去重
pub fn get_apps() -> Result<Vec<AppInfo>, String> {
    let mut hkey_apps = get_apps_from_hkey()?;
    let shortcut_apps = get_apps_from_shortcuts()?;

    // 合并并去重
    for app in shortcut_apps {
        if !hkey_apps.iter().any(|a| a.name == app.name) {
            hkey_apps.push(app);
        }
    }
    Ok(hkey_apps)
}

pub fn open_app(path: &str) -> Result<(), String> {
    Command::new("cmd")
        .args(&["/C", "start", "", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn is_uninstall_path(link: &ShellLink) -> bool {
    if let Some(link_info) = link.link_info() {
        if let Some(local_path) = link_info.local_base_path() {
            let path_str = local_path.to_lowercase();
            return path_str.contains("uninstall") || path_str.contains("卸载");
        }
    }
    false
}

fn should_filter_shortcut(shortcut: &Path) -> bool {
    if let Ok(link) = ShellLink::open(shortcut) {
        // 检查路径和参数
        let is_uninstall = is_uninstall_path(&link);
        is_uninstall
    } else {
        false
    }
}
