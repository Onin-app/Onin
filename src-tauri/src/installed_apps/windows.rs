use base64;
use futures::StreamExt;
use image::{io::Reader, ImageFormat};
use lnk::ShellLink;
use num_cpus;
use regex::Regex;
use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};
use tokio::task;
use walkdir::WalkDir;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::{RegKey, HKEY};

use super::exe_to_icon::extract_icon_from_exe;
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
#[tracing::instrument]
async fn get_apps_from_hkey() -> Result<Vec<AppInfo>, String> {
    let mut futures: Vec<tokio::task::JoinHandle<Result<Option<AppInfo>, String>>> = Vec::new(); // 已经有这个类型注解了，保持不变

    for (path, hive) in UNINSTALL_PATHS {
        let root = RegKey::predef(*hive);
        if let Ok(uninstall_key) = root.open_subkey(path) {
            for item in uninstall_key.enum_keys().filter_map(Result::ok) {
                let subkey = match uninstall_key.open_subkey(&item) {
                    Ok(sk) => sk,
                    Err(_) => continue,
                };

                let display_name: Result<String, _> = subkey.get_value("DisplayName");
                if display_name.is_err() {
                    continue;
                }
                let system_component: Result<u32, _> = subkey.get_value("SystemComponent");
                if matches!(system_component, Ok(1)) {
                    continue;
                }
                let has_parent_key = subkey.get_value::<String, _>("ParentKeyName").is_ok();
                let has_parent_display = subkey.get_value::<String, _>("ParentDisplayName").is_ok();
                if has_parent_key || has_parent_display {
                    continue;
                }

                let display_name_val = display_name.unwrap();
                let display_icon_val = subkey.get_value::<String, _>("DisplayIcon").ok();
                let install_location_val = subkey.get_value::<String, _>("InstallLocation").ok();
                let uninstall_string_val = subkey.get_value::<String, _>("UninstallString").ok();

                let app_path_candidate = extract_exe_path(
                    display_icon_val.clone(),
                    install_location_val.clone(),
                    uninstall_string_val.clone(),
                );

                if let Some(ref p) = app_path_candidate {
                    if !p.to_lowercase().contains("msiexec.exe") {
                        let name_cloned = display_name_val.clone();
                        let path_cloned = p.clone();
                        let icon_path_cloned = display_icon_val.clone();

                        futures.push(task::spawn_blocking(move || {
                            let icon_base64 =
                                extract_icon_from_exe_or_image(&icon_path_cloned, &path_cloned);
                            // <<-- 修复点：将 Ok(AppInfo { ... }) 改为 Ok(Some(AppInfo { ... }))
                            Ok(Some(AppInfo {
                                // <<-- 加上 Some()
                                name: normalize_app_name(&name_cloned),
                                path: Some(path_cloned),
                                icon: icon_base64,
                                origin: Some(AppOrigin::Hkey),
                            }))
                        }));
                    }
                }
            }
        }
    }

    // 这一部分代码不需要修改，它已经正确处理了 Result<Option<AppInfo>, String>
    let apps: Vec<AppInfo> = futures::stream::iter(futures)
        .buffer_unordered(num_cpus::get() * 2)
        .filter_map(|res| async move {
            match res {
                Ok(Ok(Some(app))) => Some(app),
                Ok(Ok(None)) => None,
                Ok(Err(e)) => {
                    tracing::error!("get_apps_from_hkey: Blocking task failed: {:?}", e);
                    None
                }
                Err(e) => {
                    tracing::error!("get_apps_from_hkey: Join error: {:?}", e);
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(apps)
}

fn normalize_app_name(name: &str) -> String {
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

    let re = Regex::new(r"\s*\d+(\.\d+)+").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"\([^)]*\)").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"\[[^\]]*\]").unwrap();
    result = re.replace_all(&result, "").to_string();

    result.trim().to_string()
}

fn extract_exe_path(
    display_icon: Option<String>,
    install_location: Option<String>,
    uninstall_string: Option<String>,
) -> Option<String> {
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

    if let Some(loc) = &install_location {
        let guessed = std::path::Path::new(loc).join("start.exe");
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

    if let Some(uninstall) = uninstall_string {
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

    None
}

fn get_all_shortcuts_sync() -> Vec<PathBuf> {
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
        for entry in WalkDir::new(base)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map(|s| s == "lnk")
                .unwrap_or(false)
            {
                shortcuts.push(entry.path().to_path_buf());
            }
        }
    }

    shortcuts
}

// 从快捷方式中获取应用列表
#[tracing::instrument]
async fn get_apps_from_shortcuts() -> Result<Vec<AppInfo>, String> {
    let shortcut_paths = get_all_shortcuts_sync();
    let mut futures: Vec<tokio::task::JoinHandle<Result<Option<AppInfo>, String>>> = Vec::new();

    for shortcut_path in shortcut_paths {
        // 重命名为 shortcut_path 更清晰
        // 不需要在这里克隆，因为 PathBuf 会在 move 闭包中被所有权转移
        futures.push(task::spawn_blocking(move || {
            // 在这里打开 ShellLink 一次
            let shell_link = match ShellLink::open(&shortcut_path) {
                Ok(link) => link,
                Err(e) => {
                    // 记录无法打开快捷方式的错误，并跳过
                    tracing::warn!("Failed to open shortcut {:?}: {:?}", shortcut_path, e);
                    return Ok(None);
                }
            };

            // 1. 过滤快捷方式，使用已打开的 shell_link
            if should_filter_shortcut_with_link(&shell_link) {
                // <<-- 新增函数，使用已打开的 link
                return Ok(None);
            }

            let mut app_info: Option<AppInfo> = None;
            // 2. 解析快捷方式目标路径，使用已打开的 shell_link
            if let Some(target_path) = extract_target_from_lnk_with_link(&shell_link) {
                // <<-- 新增函数，使用已打开的 link
                let app_name = shortcut_path // 使用原始 shortcut_path 获取文件名
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                // 3. 提取图标，使用已打开的 shell_link
                let icon = extract_icon_from_shortcut_with_link(&shell_link, &target_path); // <<-- 新增函数，使用已打开的 link 和 target_path

                if target_path.to_lowercase().ends_with(".exe")
                    || target_path.to_lowercase().ends_with(".msi")
                    || target_path.to_lowercase().ends_with(".bat")
                {
                    app_info = Some(AppInfo {
                        name: normalize_app_name(&app_name),
                        path: Some(target_path.clone()),
                        icon: icon,
                        origin: Some(AppOrigin::Shortcut),
                    });
                }
            }
            Ok(app_info)
        }));
    }

    let apps: Vec<AppInfo> = futures::stream::iter(futures)
        .buffer_unordered(num_cpus::get() * 2)
        .filter_map(|res| async move {
            match res {
                Ok(Ok(Some(app))) => Some(app),
                Ok(Ok(None)) => None,
                Ok(Err(e)) => {
                    tracing::error!("get_apps_from_shortcuts: Blocking task failed: {:?}", e);
                    None
                }
                Err(e) => {
                    tracing::error!("get_apps_from_shortcuts: Join error: {:?}", e);
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(apps)
}

// should_filter_shortcut 的新版本，接受 ShellLink 引用
// 保持同步，因为会在 spawn_blocking 内部调用
fn should_filter_shortcut_with_link(link: &ShellLink) -> bool {
    // 逻辑不变，直接使用传入的 link
    is_uninstall_path(link) // is_uninstall_path 已经接受 ShellLink 引用
}

// extract_target_from_lnk 的新版本，接受 ShellLink 引用
// 保持同步，因为会在 spawn_blocking 内部调用
fn extract_target_from_lnk_with_link(link: &ShellLink) -> Option<String> {
    let link_info = link.link_info().as_ref()?;
    let local_path = link_info.local_base_path().clone()?;

    if Path::new(&local_path).exists() {
        Some(local_path)
    } else {
        None
    }
}

// extract_icon_from_shortcut 的新版本，接受 ShellLink 引用和 target_path
// 保持同步，因为会在 spawn_blocking 内部调用
fn extract_icon_from_shortcut_with_link(link: &ShellLink, target_path: &str) -> Option<String> {
    // 尝试获取快捷方式的图标路径
    if let Some(icon_location) = link.icon_location() {
        let path = icon_location.to_string();
        if Path::new(&path).exists() {
            if let Some(ext) = Path::new(&path).extension().and_then(|s| s.to_str()) {
                let ext = ext.to_lowercase();
                if matches!(ext.as_str(), "ico" | "png" | "jpg" | "jpeg" | "bmp") {
                    return convert_image_to_base64(&path);
                }
            }
            if path.to_lowercase().ends_with(".exe") {
                return extract_icon_from_exe(&path);
            }
        }
    }
    // 从目标EXE提取图标 (如果上面没有成功)
    if target_path.to_lowercase().ends_with(".exe") {
        return extract_icon_from_exe(target_path);
    }
    None
}

pub async fn get_apps() -> Result<Vec<AppInfo>, String> {
    let hkey_apps = get_apps_from_hkey().await?;
    let shortcut_apps = get_apps_from_shortcuts().await?;

    // 使用 HashMap 来存储唯一的应用程序信息，键为标准化后的应用程序名称
    // 这样可以确保去重，并允许我们根据优先级进行选择。
    // key: String (normalized app name), value: AppInfo
    let mut unique_apps: HashMap<String, AppInfo> = HashMap::new();

    // 1. 优先添加注册表中的应用程序
    for app in hkey_apps {
        // 使用应用程序的标准化名称作为 HashMap 的键
        unique_apps.insert(app.name.clone(), app); // clone name for key, move app for value
    }

    // 2. 添加快捷方式中的应用程序，但只添加 HashMap 中不存在的
    for app in shortcut_apps {
        let normalized_name = normalize_app_name(&app.name); // 确保快捷方式的名称也标准化

        // 如果 HashMap 中还没有这个应用程序，或者你想更新它（例如，如果快捷方式提供了更好的图标），
        // 可以在这里插入。
        // 这里采用你的优先级策略：如果注册表中已有，则快捷方式的跳过。
        unique_apps.entry(normalized_name).or_insert(app); // 如果键不存在，则插入 app
    }

    let mut final_apps: Vec<AppInfo> = unique_apps.into_values().collect();
    // 确保所有 icon 都有默认值
    for app_info in &mut final_apps {
        // 遍历 final_apps
        if app_info.icon.is_none() {
            app_info.icon = Some(DEFAULT_APP_ICON.to_string());
        }
    }

    // 3. 将 HashMap 中的所有值收集到 Vec 中返回
    Ok(final_apps)
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

fn convert_image_to_base64(image_path: &str) -> Option<String> {
    let path = PathBuf::from(image_path);

    let mut reader = Reader::open(&path).ok()?;

    let format_guess = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext_str| match ext_str.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "ico" => Some(ImageFormat::Ico),
            "bmp" => Some(ImageFormat::Bmp),
            _ => None,
        });

    if let Some(fmt) = format_guess {
        reader.set_format(fmt);
    }

    let img = reader.decode().ok()?;

    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Png).ok()?;

    let encoded_bytes = buffer.into_inner();

    Some(base64::encode(&encoded_bytes))
}

fn extract_icon_from_exe_or_image(
    display_icon_path: &Option<String>,
    exe_path: &String,
) -> Option<String> {
    // 1. 尝试从 DisplayIcon 提取
    if let Some(icon_str) = display_icon_path {
        let clean_path = if let Some((path, _)) = icon_str.split_once(',') {
            path.trim_matches('"').to_string()
        } else {
            icon_str.trim_matches('"').to_string() // <<-- 修复：移除这里的分号
        };

        let path = Path::new(&clean_path);
        if path.exists() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            if let Some(ext) = ext {
                if matches!(ext.as_str(), "ico" | "png" | "jpg" | "jpeg" | "bmp") {
                    if let Some(base64_data) = convert_image_to_base64(&clean_path) {
                        return Some(base64_data);
                    }
                }
            }

            if clean_path.to_lowercase().ends_with(".exe") {
                if let Some(base64_data) = extract_icon_from_exe(&clean_path) {
                    return Some(base64_data);
                }
            }
        }
    }

    if let Some(base64_data) = extract_icon_from_exe(exe_path) {
        return Some(base64_data);
    }

    None
}
