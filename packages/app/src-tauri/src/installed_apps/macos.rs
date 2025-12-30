use super::AppInfo;
use base64::{engine::general_purpose, Engine as _};
use icns::{IconFamily, IconType};
use plist::Value;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::async_runtime;
use tracing::{error, info};

fn get_app_icon(app_path: &str) -> Option<String> {
    info!("[ICON] Processing app: {}", app_path);
    let info_plist_path = Path::new(app_path).join("Contents/Info.plist");
    let info_plist = match Value::from_file(&info_plist_path) {
        Ok(plist) => plist,
        Err(e) => {
            error!("[ICON] Failed to read Info.plist for {}: {}", app_path, e);
            return None;
        }
    };
    let resources_path = Path::new(app_path).join("Contents/Resources");
    // 1. Try to get icon path from Info.plist
    let mut icon_path: Option<PathBuf> = info_plist
        .as_dictionary()
        .and_then(|dict| dict.get("CFBundleIconFile"))
        .and_then(|val| val.as_string())
        .map(|name| {
            let mut path = resources_path.join(name);
            if path.extension().is_none() {
                path.set_extension("icns");
            }
            path
        });

    // 2. Check if the path from plist actually exists. If not, reset to None.
    if let Some(path) = &icon_path {
        if !path.exists() {
            info!(
                "[ICON] Icon from plist does not exist: {:?}. Will search directory.",
                path
            );
            icon_path = None;
        }
    }

    // 3. If we still don't have a path, try common names.
    if icon_path.is_none() {
        info!("[ICON] No valid icon from plist, trying common names.");
        let app_name = Path::new(app_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let common_names = [
            format!("{}.icns", app_name),
            "AppIcon.icns".to_string(),
            "icon.icns".to_string(),
        ];

        for name in &common_names {
            let potential_path = resources_path.join(name);
            if potential_path.exists() {
                info!("[ICON] Found common icon: {:?}", potential_path);
                icon_path = Some(potential_path);
                break;
            }
        }
    }

    // 4. If still no path, fall back to the first .icns file in the directory.
    if icon_path.is_none() {
        info!("[ICON] No common icon found, searching directory for any .icns file.");
        icon_path = fs::read_dir(&resources_path)
            .ok()
            .and_then(|entries| {
                entries
                    .flatten()
                    .find(|entry| entry.path().extension().map_or(false, |ext| ext == "icns"))
            })
            .map(|entry| entry.path());
    }

    // 5. If we have a path, use it. Otherwise, fail.
    let final_icon_path = match &icon_path {
        Some(path) => {
            info!("[ICON] Using final icon path: {:?}", path);
            path.clone()
        }
        None => {
            error!("[ICON] No usable icon file found for {}", app_path);
            return None;
        }
    };
    // This log line is now redundant because of the final_icon_path log above.
    // info!("[ICON] Using icon path: {:?}", icon_path);

    let mut file = match File::open(&final_icon_path) {
        Ok(f) => f,
        Err(e) => {
            error!(
                "[ICON] Failed to open icon file {:?}: {}",
                final_icon_path, e
            );
            return None;
        }
    };
    let icon_family = match IconFamily::read(&mut file) {
        Ok(family) => family,
        Err(e) => {
            error!(
                "[ICON] Failed to read icon family from {:?}: {}",
                final_icon_path, e
            );
            return None;
        }
    };

    let best_icon = icon_family
        .elements
        .iter()
        .filter(|icon| {
            matches!(
                icon.icon_type(),
                Some(IconType::RGBA32_128x128) | Some(IconType::RGBA32_256x256)
            )
        })
        .max_by_key(|icon| {
            if let Ok(img) = icns::Image::read_png(&icon.data[..]) {
                img.width() * img.height()
            } else {
                0
            }
        });

    let icon_element = match best_icon {
        Some(icon) => icon,
        None => icon_family.elements.first()?,
    };

    let image = match icns::Image::read_png(&icon_element.data[..]) {
        Ok(img) => img,
        Err(e) => {
            error!(
                "[ICON] Failed to read png from icon element for {:?}: {}",
                final_icon_path, e
            );
            return None;
        }
    };
    let mut png_data = Vec::new();
    if let Err(e) = image.write_png(&mut png_data) {
        error!(
            "[ICON] Failed to write png data for {:?}: {}",
            final_icon_path, e
        );
        return None;
    }

    let base64_icon = general_purpose::STANDARD.encode(&png_data);
    info!("[ICON] Successfully processed icon for {}", app_path);
    Some(base64_icon)
}

fn get_system_localized_name(app_path: &Path) -> Option<String> {
    let app_path_str = app_path.to_string_lossy();

    // 使用 mdls 命令获取应用的本地化名称
    let output = Command::new("mdls")
        .arg("-name")
        .arg("kMDItemDisplayName")
        .arg("-raw")
        .arg(&*app_path_str)
        .output()
        .ok()?;

    if output.status.success() {
        let display_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !display_name.is_empty() && display_name != "(null)" {
            info!("[SYSTEM] Found system localized name: {}", display_name);
            return Some(display_name);
        }
    }

    None
}

fn get_localized_name(app_path: &Path, _base_name: &str) -> Option<String> {
    let resources_path = app_path.join("Contents/Resources");

    // 扩展语言目录列表，包括可能的中文本地化
    let lang_dirs = [
        "zh_CN.lproj",
        "zh_Hans.lproj",
        "zh.lproj",
        "Chinese.lproj",
        "en.lproj",
    ];

    for lang in &lang_dirs {
        let strings_path = resources_path.join(lang).join("InfoPlist.strings");
        if strings_path.exists() {
            info!("[LOCALIZED] Checking: {:?}", strings_path);

            // 方法1: 尝试作为 plist 文件读取
            if let Ok(localized_plist) = Value::from_file(&strings_path) {
                if let Some(dict) = localized_plist.as_dictionary() {
                    if let Some(display_name) = dict
                        .get("CFBundleDisplayName")
                        .or_else(|| dict.get("CFBundleName"))
                        .and_then(|v| v.as_string())
                    {
                        info!("[LOCALIZED] Found plist format name: {}", display_name);
                        return Some(display_name.to_string());
                    }
                }
            }

            // 方法2: 尝试作为 strings 文件读取
            if let Ok(content) = fs::read_to_string(&strings_path) {
                info!("[LOCALIZED] Reading strings file content");

                // 解析 strings 文件格式 (key = "value";)
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
                        continue;
                    }

                    // 查找 CFBundleDisplayName 或 CFBundleName
                    if line.starts_with("CFBundleDisplayName") || line.starts_with("CFBundleName") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line.rfind('"') {
                                if start < end {
                                    let display_name = &line[start + 1..end];
                                    info!(
                                        "[LOCALIZED] Found strings format name: {}",
                                        display_name
                                    );
                                    return Some(display_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 如果没有找到本地化名称，尝试从系统获取
    if let Some(localized_name) = get_system_localized_name(app_path) {
        return Some(localized_name);
    }

    None
}

pub async fn get_apps() -> Result<Vec<AppInfo>, String> {
    async_runtime::spawn_blocking(|| {
        let mut app_map = HashMap::new();
        let mut search_paths = vec![
            PathBuf::from("/System/Applications"),
            PathBuf::from("/System/Applications/Utilities"),
            PathBuf::from("/Applications"),
        ];

        if let Ok(home_dir) = env::var("HOME") {
            search_paths.push(PathBuf::from(home_dir).join("Applications"));
        }

        for path in search_paths {
            if !path.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let file_name = entry.file_name().to_string_lossy().to_string();
                            if file_name.ends_with(".app") {
                                let app_path = entry.path();
                                let app_path_str = app_path.to_string_lossy().to_string();
                                let mut keywords: Vec<String> = Vec::new();

                                let name = app_path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(&file_name)
                                    .to_string();

                                // 1. 获取本地化名称
                                if let Some(localized_name) = get_localized_name(&app_path, &name) {
                                    // 移除了 != name 的条件判断
                                    if !keywords.contains(&localized_name) {
                                        keywords.push(localized_name);
                                    }
                                }

                                // 2. 从 Info.plist 获取其他名称
                                if let Ok(plist) =
                                    Value::from_file(app_path.join("Contents/Info.plist"))
                                {
                                    if let Some(dict) = plist.as_dictionary() {
                                        if let Some(display_name) = dict
                                            .get("CFBundleDisplayName")
                                            .and_then(|v| v.as_string())
                                        {
                                            if !keywords.contains(&display_name.to_string()) {
                                                keywords.push(display_name.to_string());
                                            }
                                        }
                                        if let Some(bundle_name) =
                                            dict.get("CFBundleName").and_then(|v| v.as_string())
                                        {
                                            if !keywords.contains(&bundle_name.to_string()) {
                                                keywords.push(bundle_name.to_string());
                                            }
                                        }
                                    }
                                }

                                let icon = get_app_icon(&app_path_str);
                                let app_info = AppInfo {
                                    name: name.clone(),
                                    keywords,
                                    path: Some(app_path_str),
                                    icon,
                                    origin: None,
                                };
                                app_map.insert(name, app_info);
                            }
                        }
                    }
                }
            }
        }

        let mut apps: Vec<AppInfo> = app_map.into_values().collect();
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(apps)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn open_app(path: &str) -> Result<(), String> {
    // 在 macOS 上，使用 open 命令本身就会创建独立的进程
    // 但为了确保进程完全分离，使用 nohup 和后台运行
    Command::new("sh")
        .arg("-c")
        .arg(format!("nohup open '{}' > /dev/null 2>&1 &", path))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
