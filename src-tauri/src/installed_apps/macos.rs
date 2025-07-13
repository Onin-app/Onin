use super::AppInfo;
use base64::{engine::general_purpose, Engine as _};
use icns::{IconFamily, IconType};
use plist::Value;
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
            info!("[ICON] Icon from plist does not exist: {:?}. Will search directory.", path);
            icon_path = None;
        }
    }

    // 3. If we still don't have a path, try common names.
    if icon_path.is_none() {
        info!("[ICON] No valid icon from plist, trying common names.");
        let app_name = Path::new(app_path).file_stem().and_then(|s| s.to_str()).unwrap_or_default();
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
            error!("[ICON] Failed to open icon file {:?}: {}", final_icon_path, e);
            return None;
        }
    };
    let icon_family = match IconFamily::read(&mut file) {
        Ok(family) => family,
        Err(e) => {
            error!("[ICON] Failed to read icon family from {:?}: {}", final_icon_path, e);
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
            error!("[ICON] Failed to read png from icon element for {:?}: {}", final_icon_path, e);
            return None;
        }
    };
    let mut png_data = Vec::new();
    if let Err(e) = image.write_png(&mut png_data) {
        error!("[ICON] Failed to write png data for {:?}: {}", final_icon_path, e);
        return None;
    }

    let base64_icon = general_purpose::STANDARD.encode(&png_data);
    info!("[ICON] Successfully processed icon for {}", app_path);
    Some(base64_icon)
}

pub async fn get_apps() -> Result<Vec<AppInfo>, String> {
    async_runtime::spawn_blocking(|| {
        let mut apps = vec![];
        if let Ok(entries) = fs::read_dir("/Applications") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.ends_with(".app") {
                            let path = format!("/Applications/{}", name);
                            let icon = get_app_icon(&path);
                            apps.push(AppInfo {
                                name,
                                path: Some(path),
                                icon,
                                origin: None,
                            });
                        }
                    }
                }
            }
        }
        Ok(apps)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn open_app(path: &str) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
