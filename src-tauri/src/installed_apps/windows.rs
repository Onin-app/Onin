use base64::{engine::general_purpose, Engine as _};
use futures::StreamExt;
use image::{ImageEncoder, ImageFormat, ImageReader};
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

use windows::{
    core::{Interface, PWSTR},
    Win32::{
        Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER,
            DIB_RGB_COLORS,
        },
        System::Com::{
            CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
        },
        UI::Shell::{
            BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem,
            IShellItemImageFactory, SHGetKnownFolderItem, KF_FLAG_DEFAULT, SIGDN_NORMALDISPLAY,
            SIGDN_PARENTRELATIVEPARSING, SIIGBF_ICONONLY,
        },
    },
};

fn hbitmap_to_base64_png(hbitmap: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<String> {
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    let hdc = unsafe { CreateCompatibleDC(None) };
    if hdc.is_invalid() {
        return None;
    }

    // Get bitmap info header
    if unsafe {
        GetDIBits(
            hdc,
            hbitmap,
            0,
            0,
            None,
            &mut bmp_info as *mut _ as *mut _,
            DIB_RGB_COLORS,
        )
    } == 0
    {
        let _ = unsafe { DeleteDC(hdc) };
        return None;
    }

    let width = bmp_info.bmiHeader.biWidth;
    let height = bmp_info.bmiHeader.biHeight.abs();
    bmp_info.bmiHeader.biHeight = height; // Ensure height is positive for top-down DIB
    bmp_info.bmiHeader.biCompression = 0; // BI_RGB

    let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

    // Get bitmap data
    if unsafe {
        GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info as *mut _ as *mut _,
            DIB_RGB_COLORS,
        )
    } == 0
    {
        unsafe {
            let _ = DeleteDC(hdc);
        };
        return None;
    }

    unsafe {
        let _ = DeleteDC(hdc);
    };

    // Manually convert BGRA to RGBA and flip vertically
    let mut rgba_buffer = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let src_idx = ((height - 1 - y) * width + x) as usize * 4;
            let dest_idx = (y * width + x) as usize * 4;
            rgba_buffer[dest_idx] = buffer[src_idx + 2]; // R
            rgba_buffer[dest_idx + 1] = buffer[src_idx + 1]; // G
            rgba_buffer[dest_idx + 2] = buffer[src_idx]; // B
            rgba_buffer[dest_idx + 3] = buffer[src_idx + 3]; // A
        }
    }

    let mut png_buffer = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_buffer)
        .write_image(
            &rgba_buffer,
            width as u32,
            height as u32,
            image::ColorType::Rgba8.into(),
        )
        .ok()
        .map(|_| general_purpose::STANDARD.encode(&png_buffer))
}

fn get_apps_from_apps_folder() -> Result<Vec<AppInfo>, String> {
    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_err() {
            return Err("Failed to initialize COM".to_string());
        }
    }

    let apps_folder: Result<IShellItem, _> =
        unsafe { SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None) };

    let apps_folder = match apps_folder {
        Ok(folder) => folder,
        Err(_) => {
            unsafe { CoUninitialize() };
            return Err("Failed to get AppsFolder".to_string());
        }
    };

    let apps_folder_enum: Result<IEnumShellItems, _> =
        unsafe { apps_folder.BindToHandler(None, &BHID_EnumItems) };
    let apps_folder_enum = match apps_folder_enum {
        Ok(e) => e,
        Err(_) => {
            unsafe { CoUninitialize() };
            return Err("Failed to bind to enum items".to_string());
        }
    };

    let mut apps = Vec::new();
    let mut app_item_raw: [Option<IShellItem>; 1] = [None];
    while unsafe { apps_folder_enum.Next(&mut app_item_raw, None).is_ok() }
        && app_item_raw[0].is_some()
    {
        if let Some(app) = app_item_raw[0].take() {
            let name_pwstr: Result<PWSTR, _> = unsafe { app.GetDisplayName(SIGDN_NORMALDISPLAY) };
            let path_pwstr: Result<PWSTR, _> =
                unsafe { app.GetDisplayName(SIGDN_PARENTRELATIVEPARSING) };

            if let (Ok(name_pwstr), Ok(path_pwstr)) = (name_pwstr, path_pwstr) {
                let name = unsafe { name_pwstr.to_string().unwrap_or_default() };
                let path = unsafe { path_pwstr.to_string().unwrap_or_default() };

                let icon = unsafe {
                    app.cast::<IShellItemImageFactory>()
                        .ok()
                        .and_then(|factory| {
                            let size = windows::Win32::Foundation::SIZE { cx: 64, cy: 64 };
                            factory
                                .GetImage(size, SIIGBF_ICONONLY)
                                .ok()
                                .and_then(|hbitmap| {
                                    let b64 = hbitmap_to_base64_png(hbitmap);
                                    let _ = DeleteObject(hbitmap);
                                    b64
                                })
                        })
                };

                if !name.is_empty() && !path.is_empty() {
                    apps.push(AppInfo {
                        name: normalize_app_name(&name),
                        path: Some(format!("shell:AppsFolder\\{}", path)),
                        icon,
                        origin: Some(AppOrigin::Uwp),
                    });
                }
            }
        }
    }

    unsafe { CoUninitialize() };
    Ok(apps)
}

pub async fn get_apps() -> Result<Vec<AppInfo>, String> {
    let hkey_apps_future = get_apps_from_hkey();
    let shortcut_apps_future = get_apps_from_shortcuts();
    let apps_folder_apps = task::spawn_blocking(get_apps_from_apps_folder)
        .await
        .unwrap()?;

    let (hkey_apps_result, shortcut_apps_result) =
        tokio::join!(hkey_apps_future, shortcut_apps_future);

    let hkey_apps = hkey_apps_result?;
    let shortcut_apps = shortcut_apps_result?;

    let mut unique_apps: HashMap<String, AppInfo> = HashMap::new();

    // 1. 优先添加 AppsFolder 中的应用程序
    for app in apps_folder_apps {
        unique_apps.insert(app.name.clone(), app);
    }

    // 2. 添加注册表中的应用程序
    for app in hkey_apps {
        unique_apps.entry(app.name.clone()).or_insert(app);
    }

    // 3. 添加快捷方式中的应用程序
    for app in shortcut_apps {
        let normalized_name = normalize_app_name(&app.name);
        unique_apps.entry(normalized_name).or_insert(app);
    }

    let final_apps: Vec<AppInfo> = unique_apps.into_values().collect();

    Ok(final_apps)
}

pub fn open_app(path: &str) -> Result<(), String> {
    if path.starts_with("shell:AppsFolder\\") {
        unsafe {
            use windows::core::PCWSTR;
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;
            let operation = widestring::U16CString::from_str("open").unwrap();
            let file = widestring::U16CString::from_str("explorer.exe").unwrap();
            let params = widestring::U16CString::from_str(path).unwrap();
            ShellExecuteW(
                None,
                PCWSTR(operation.as_ptr()),
                PCWSTR(file.as_ptr()),
                PCWSTR(params.as_ptr()),
                None,
                SW_NORMAL,
            );
        }
    } else {
        Command::new("cmd")
            .args(&["/C", "start", "", path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
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

    let mut reader = ImageReader::open(&path).ok()?;

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

    Some(general_purpose::STANDARD.encode(&encoded_bytes))
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
