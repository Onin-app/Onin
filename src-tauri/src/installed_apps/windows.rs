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
use super::AppOrigin;

const DEFAULT_APP_ICON: &str = "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";

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

                    let path =
                        extract_exe_path(display_icon.clone(), install_location, uninstall_string);
                    let icon = extract_icon_path(display_icon);

                    if let Some(ref p) = path {
                        if !p.to_lowercase().contains("msiexec.exe") {
                            apps.push(AppInfo {
                                name: normalize_app_name(&display_name),
                                path,
                                icon: icon.or_else(|| Some(DEFAULT_APP_ICON.to_string())),
                                origin: Some(AppOrigin::Hkey),
                            });
                        }
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
        let lower_icon = icon.to_lowercase();
        if lower_icon.ends_with(".exe")
            || lower_icon.ends_with(".msi")
            || lower_icon.ends_with(".bat")
        {
            return Some(icon);
        }
        if lower_icon.contains(".exe,")
            || lower_icon.contains(".msi,")
            || lower_icon.contains(".bat,")
        {
            let parts: Vec<&str> = icon.split(",").collect();
            let first_part = parts[0].to_lowercase();
            if first_part.ends_with(".exe")
                || first_part.ends_with(".msi")
                || first_part.ends_with(".bat")
            {
                return Some(parts[0].to_string());
            }
        }
    }

    // 2. 从 InstallLocation 猜测路径
    if let Some(loc) = &install_location {
        let guessed = std::path::Path::new(loc).join("start.exe"); // 默认启动文件名示意
        if guessed.exists() {
            let guessed_str = guessed.to_string_lossy().to_string();
            if guessed_str.to_lowercase().ends_with(".exe")
                || guessed_str.to_lowercase().ends_with(".msi")
                || guessed_str.to_lowercase().ends_with(".bat")
            {
                return Some(guessed_str);
            }
        }
    }

    // 3. 尝试清洗 UninstallString
    if let Some(uninstall) = uninstall_string {
        // 抽取包含 .exe/.msi/.bat 的路径（排除带 uninstall 字样的）
        if uninstall.to_lowercase().contains(".exe")
            || uninstall.to_lowercase().contains(".msi")
            || uninstall.to_lowercase().contains(".bat")
        {
            let re = Regex::new("\"?([^\"]+?\\.(exe|msi|bat))\"?").unwrap();
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

fn extract_icon_path(display_icon: Option<String>) -> Option<String> {
    if let Some(icon) = display_icon {
        // 处理包含逗号的情况 (如 "path.exe,1")
        let clean_path = if let Some((path, _)) = icon.split_once(',') {
            path.trim_matches('"').to_string()
        } else {
            icon.trim_matches('"').to_string()
        };

        // 检查是否是有效的图片文件
        let ext = Path::new(&clean_path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());

        if let Some(ext) = ext {
            if matches!(ext.as_str(), "ico" | "png" | "jpg" | "jpeg" | "bmp") {
                return convert_image_to_base64(&clean_path);
            }
        }

        // 如果是exe文件，尝试提取图标
        if clean_path.to_lowercase().ends_with(".exe") {
            return extract_icon_from_exe(&clean_path);
        }
    }
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
            let icon = extract_icon_from_shortcut(&shortcut);

            if target_path.to_lowercase().ends_with(".exe")
                || target_path.to_lowercase().ends_with(".msi")
                || target_path.to_lowercase().ends_with(".bat")
            {
                apps.push(AppInfo {
                    name: app_name,
                    path: Some(target_path.clone()),
                    icon: icon.or_else(|| Some(DEFAULT_APP_ICON.to_string())),
                    origin: Some(AppOrigin::Shortcut),
                });
            }
        }
    }

    Ok(apps)
}

// 合并并去重
pub fn get_apps() -> Result<Vec<AppInfo>, String> {
    let mut hkey_apps = get_apps_from_hkey()?;
    let shortcut_apps = get_apps_from_shortcuts()?;

    // 合并并去重
    // for app in shortcut_apps {
    //     if !hkey_apps.iter().any(|a| a.name == app.name) {
    //         hkey_apps.push(app);
    //     }
    // }
    for app in shortcut_apps {
        hkey_apps.push(app);
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

fn extract_icon_from_shortcut(shortcut: &Path) -> Option<String> {
    if let Ok(link) = ShellLink::open(shortcut) {
        // 尝试获取快捷方式的图标路径
        if let Some(icon_location) = link.icon_location() {
            let path = icon_location.to_string();
            if Path::new(&path).exists() {
                // 如果是图片文件，转换为base64
                if let Some(ext) = Path::new(&path).extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if matches!(ext.as_str(), "ico" | "png" | "jpg" | "jpeg" | "bmp") {
                        return convert_image_to_base64(&path);
                    }
                }
                // 如果是exe文件则提取图标
                if path.to_lowercase().ends_with(".exe") {
                    return extract_icon_from_exe(&path);
                }
            }
        }
        // 从目标EXE提取图标
        if let Some(target_path) = extract_target_from_lnk(shortcut) {
            if target_path.to_lowercase().ends_with(".exe") {
                return extract_icon_from_exe(&target_path);
            }
        }
    }
    None
}

fn convert_image_to_base64(image_path: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "[Convert]::ToBase64String([System.IO.File]::ReadAllBytes('{}'))",
                image_path
            ),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let base64 = String::from_utf8(output.stdout).ok()?;
        Some(base64.trim().to_string())
    } else {
        None
    }
}

fn extract_icon_from_exe(exe_path: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(&["-Command", 
               &format!("Add-Type -AssemblyName System.Drawing; $icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{}'); $ms = New-Object System.IO.MemoryStream; $icon.ToBitmap().Save($ms, [System.Drawing.Imaging.ImageFormat]::Png); [Convert]::ToBase64String($ms.ToArray())", 
               exe_path)])
        .output()
        .ok()?;

    if output.status.success() {
        let base64 = String::from_utf8(output.stdout).ok()?;
        Some(base64.trim().to_string())
    } else {
        None
    }
}
