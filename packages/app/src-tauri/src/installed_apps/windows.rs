use base64::{engine::general_purpose, Engine as _};
use futures::StreamExt;
use image::{ImageEncoder, ImageFormat, ImageReader};
use lnk::ShellLink;
use num_cpus;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    panic::{catch_unwind, AssertUnwindSafe},
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

// ---------------------------------------------------------------------------
// 卸载器识别（参考 Flow Launcher 的 HideUninstallersFilter）
// 关键教训：`uninst.exe` / `unins000.exe` 并不包含连续字符串 `uninstall`，
// 只匹配 `uninstall` 会漏掉最常见的卸载器。这里用「精确文件名 + 多语言前缀」
// 双模式，并同时作用于「名称」与「目标路径」两个维度。
// ---------------------------------------------------------------------------
const UNINSTALLER_EXACT_NAMES: &[&str] = &[
    "uninst.exe",
    "uninst000.exe",
    "unins000.exe",
    "uninstall.exe",
    "uninstall.lnk",
    "uninst.lnk",
];

const UNINSTALLER_NAME_PREFIXES: &[&str] = &[
    "uninstall",        // en
    "uninst",           // 缩写，覆盖 uninst.exe / uninst000.exe
    "unins",            // Inno Setup unins000.exe
    "卸载",             // zh-cn
    "卸載",             // zh-tw
    "アンインストール", // ja
    "삭제",             // ko
    "kaldır",           // tr
    "desinstaller",     // fr
    "désinstaller",
    "deinstall",  // de
    "deinstalar", // es/pt
    "הסרה",       // he
];

// 永远不能作为「应用启动目标」的系统程序。
// 这些是执行器/脚本宿主，若被当作 DisplayIcon 或快捷方式目标，几乎一定是解析错误。
const NON_APP_EXECUTABLES: &[&str] = &[
    "rundll32.exe",
    "msiexec.exe",
    "cmd.exe",
    "regsvr32.exe",
    "powershell.exe",
    "pwsh.exe",
    "wscript.exe",
    "cscript.exe",
];

// 安装器文件名。仅在 UninstallString 兜底解析时需要排除（那里出现 setup.exe
// 通常是"卸载器旁的安装器"误抓）；但它们完全可能是合法应用的真实启动目标，
// 因此不能在 DisplayIcon / 快捷方式 / 注册表来源里全局过滤。
const INSTALLER_EXECUTABLES: &[&str] = &["setup.exe", "install.exe", "installer.exe"];

/// 去掉路径/名称外层的引号与首尾空白。
fn trim_quotes(value: &str) -> &str {
    value.trim().trim_matches('"').trim_matches('\'').trim()
}

/// 判断一个「名称」（文件茎或显示名）是否看起来像卸载器。
fn is_uninstaller_name(name: &str) -> bool {
    let cleaned = trim_quotes(name);
    let lower = cleaned.to_lowercase();
    if UNINSTALLER_EXACT_NAMES.contains(&lower.as_str()) {
        return true;
    }
    // 去掉扩展名后再做前缀匹配，使 `卸载钉钉` / `uninst` 这类茎也能命中。
    let stem = lower
        .strip_suffix(".lnk")
        .or_else(|| lower.strip_suffix(".exe"))
        .unwrap_or(&lower);
    UNINSTALLER_NAME_PREFIXES
        .iter()
        .any(|prefix| stem.starts_with(prefix))
}

/// 取路径的文件名（去引号；无文件名时回退到整个字符串）。
fn file_name_of(path: &str) -> &str {
    let cleaned = trim_quotes(path);
    Path::new(cleaned)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(cleaned)
}

/// 判断一个路径指向的文件是否为卸载器。
/// 只看文件名，避免误伤安装路径中恰好含 "uninstall" 目录的合法应用。
fn looks_like_uninstaller(path: &str) -> bool {
    is_uninstaller_name(file_name_of(path))
}

/// 路径的文件名（小写）是否在给定名单中。
fn file_name_in(path: &str, names: &[&str]) -> bool {
    let file_name = file_name_of(path).to_lowercase();
    names.contains(&file_name.as_str())
}

fn is_non_app_executable(path: &str) -> bool {
    file_name_in(path, NON_APP_EXECUTABLES)
}

/// 路径指向的文件是否为安装器（仅在 UninstallString 兜底时排除）。
fn is_installer_executable(path: &str) -> bool {
    file_name_in(path, INSTALLER_EXECUTABLES)
}

/// 统一的"可作为启动目标"判断：非卸载器、非系统执行器。
fn is_launchable_target(path: &str) -> bool {
    !looks_like_uninstaller(path) && !is_non_app_executable(path)
}

/// 从任意字符串（如 DisplayIcon / UninstallString）中提取所有
/// exe/msi/bat 路径 token，忽略引号、图标索引（`,0`）与命令行参数。
fn extract_executable_tokens(value: &str) -> Vec<String> {
    let Ok(re) = Regex::new(r#"(?i)"?([^"]+?\.(?:exe|msi|bat))"?"#) else {
        return Vec::new();
    };
    re.captures_iter(value)
        .filter_map(|cap| cap.get(1).map(|m| trim_quotes(m.as_str()).to_string()))
        .collect()
}

fn extract_executable_token(value: &str) -> Option<String> {
    extract_executable_tokens(value).into_iter().next()
}

/// 展开 lnk / 注册表中偶尔保留的 `%VAR%` 环境变量。
fn expand_env_vars(value: &str) -> String {
    let mut result = value.to_string();
    const VARS: &[&str] = &[
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "WINDIR",
        "SystemRoot",
        "ProgramData",
    ];
    for key in VARS {
        if let Ok(val) = std::env::var(key) {
            result = result.replace(&format!("%{}%", key), &val);
            result = result.replace(&format!("%{}%", key.to_lowercase()), &val);
            result = result.replace(&format!("%{}%", key.to_uppercase()), &val);
        }
    }
    result
}

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

                // 过滤 1：显示名本身看起来就是卸载器（如 "卸载 XXX" / "Uninstall XXX"）。
                if is_uninstaller_name(&display_name_val) {
                    continue;
                }

                let display_icon_val = subkey.get_value::<String, _>("DisplayIcon").ok();
                let install_location_val = subkey.get_value::<String, _>("InstallLocation").ok();
                let uninstall_string_val = subkey.get_value::<String, _>("UninstallString").ok();

                let app_path_candidate = extract_exe_path(
                    display_icon_val.clone(),
                    install_location_val.clone(),
                    uninstall_string_val.clone(),
                );

                if let Some(ref p) = app_path_candidate {
                    // 过滤 2：最终解析出的启动路径不能是卸载器或系统执行器。
                    if is_launchable_target(p) {
                        let name_cloned = display_name_val.clone();
                        let path_cloned = p.clone();
                        let icon_path_cloned = display_icon_val.clone();

                        futures.push(task::spawn_blocking(move || {
                            let icon_base64 =
                                extract_icon_from_exe_or_image(&icon_path_cloned, &path_cloned);
                            Ok(Some(AppInfo {
                                name: normalize_app_name(&name_cloned),
                                keywords: vec![],
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

pub(crate) fn normalize_app_name(name: &str) -> String {
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

    // 全角括号（中文应用名常见），如 `钉钉（桌面版）`。
    let re = Regex::new(r"（[^）]*）").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"【[^】]*】").unwrap();
    result = re.replace_all(&result, "").to_string();

    result.trim().to_string()
}

/// 从一个候选路径中解析出可用于「启动」的可执行文件。
/// 注意：DisplayIcon 很多时候只是图标（可能是 .dll / .ico），不能直接当启动目标。
fn extract_exe_path(
    display_icon: Option<String>,
    install_location: Option<String>,
    uninstall_string: Option<String>,
) -> Option<String> {
    // 优先级 1：DisplayIcon 中的可执行文件路径（去引号、去 `,索引`、去参数）。
    if let Some(icon) = display_icon {
        if let Some(token) = extract_executable_token(&icon) {
            let token = expand_env_vars(&token);
            // 图标路径里指向卸载器时不能作为启动目标。
            if is_launchable_target(&token) {
                return Some(token);
            }
        }
    }

    // 优先级 2：InstallLocation 下的 start.exe（历史兼容）。
    if let Some(loc) = &install_location {
        let loc = expand_env_vars(trim_quotes(loc));
        let guessed = Path::new(&loc).join("start.exe");
        if guessed.exists() {
            let guessed_str = guessed.to_string_lossy().to_string();
            if is_launchable_target(&guessed_str) {
                return Some(guessed_str);
            }
        }
    }

    // 优先级 3：UninstallString 兜底。这里只取「第一个合法的 exe」，
    // 并跳过卸载器 / 系统安装器 / rundll32 之类（例如 MsiExec /X{GUID}）。
    // 注意：许多应用只会写 UninstallString，这是它们能出现在列表里的唯一途径。
    if let Some(uninstall) = uninstall_string {
        for token in extract_executable_tokens(&uninstall) {
            let token = expand_env_vars(&token);
            // 兜底来源里额外排除安装器：UninstallString 中出现的 setup.exe
            // 往往是卸载器旁误抓的安装程序，不是用来启动应用的。
            if !is_launchable_target(&token) || is_installer_executable(&token) {
                continue;
            }
            return Some(token);
        }
    }

    None
}

fn get_all_shortcuts_sync() -> Vec<PathBuf> {
    let mut shortcuts = Vec::new();
    let user_profile = match std::env::var("USERPROFILE") {
        Ok(profile) => profile,
        Err(e) => {
            tracing::warn!(
                "Failed to read USERPROFILE while scanning shortcuts: {:?}",
                e
            );
            return shortcuts;
        }
    };

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

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        message.to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn is_valid_lnk_file(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut header = [0u8; 20];
    if file.read_exact(&mut header).is_err() {
        return false;
    }

    // 1. Check HeaderSize: 0x0000004C (Little Endian: [0x4C, 0x00, 0x00, 0x00])
    if header[0..4] != [0x4C, 0x00, 0x00, 0x00] {
        return false;
    }

    // 2. Check LinkCLSID: 00021401-0000-0000-C000-000000000046
    // Bytes representation: [01 14 02 00 00 00 00 00 C0 00 00 00 00 00 00 46]
    let expected_clsid = [
        0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x46,
    ];
    if header[4..20] != expected_clsid {
        return false;
    }

    true
}

fn app_from_shortcut(shortcut_path: PathBuf) -> Result<Option<AppInfo>, String> {
    // 设计说明：
    // 由于 `lnk` 库目前仅公开了 `ShellLink::open` 这一个文件读取和解析接口，
    // 未提供直接从内存字节/自定义 Reader 解析的公开 API。为了在调用 `ShellLink::open` 前完成防 panic 的预校验，
    // 我们必须在 `is_valid_lnk_file` 中先打开并读取前 20 字节。
    // 虽存在二次文件打开，但在 Windows 的系统文件缓存（File Cache）机制下，第二次打开与读取是极速的纯内存操作，
    // 对启动扫描的性能没有实质影响。
    if !is_valid_lnk_file(&shortcut_path) {
        tracing::warn!(
            "Skipping invalid or corrupted shortcut: {:?}",
            shortcut_path
        );
        return Ok(None);
    }

    let shell_link = match ShellLink::open(&shortcut_path, lnk::encoding::WINDOWS_1252) {
        Ok(link) => link,
        Err(e) => {
            tracing::warn!("Failed to open shortcut {:?}: {:?}", shortcut_path, e);
            return Ok(None);
        }
    };

    let app_name = shortcut_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 目标只解析一次，供过滤与后续构造共用（避免重复 env 展开 / 路径拼接）。
    let target_path = extract_target_from_lnk_with_link(&shell_link);

    // 同时检查「快捷方式文件名」与「目标路径」两轴（对齐 Flow Launcher）：
    // `卸载钉钉.lnk` 的目标 `uninst.exe` 即使名字不含 uninstall，也会被拦住。
    if should_filter_shortcut(&app_name, &shell_link, target_path.as_deref()) {
        return Ok(None);
    }

    let mut app_info: Option<AppInfo> = None;
    if let Some(target_path) = target_path {
        let icon = extract_icon_from_shortcut_with_link(&shell_link, &target_path);

        if target_path.to_lowercase().ends_with(".exe")
            || target_path.to_lowercase().ends_with(".msi")
            || target_path.to_lowercase().ends_with(".bat")
        {
            app_info = Some(AppInfo {
                name: normalize_app_name(&app_name),
                keywords: vec![],
                path: Some(target_path.clone()),
                icon,
                origin: Some(AppOrigin::Shortcut),
            });
        }
    }

    Ok(app_info)
}

// 从快捷方式中获取应用列表
#[tracing::instrument]
async fn get_apps_from_shortcuts() -> Result<Vec<AppInfo>, String> {
    let shortcut_paths = get_all_shortcuts_sync();
    let mut futures: Vec<tokio::task::JoinHandle<Result<Option<AppInfo>, String>>> = Vec::new();

    for shortcut_path in shortcut_paths {
        futures.push(task::spawn_blocking(move || {
            let shortcut_path_for_log = shortcut_path.clone();
            match catch_unwind(AssertUnwindSafe(|| app_from_shortcut(shortcut_path))) {
                Ok(result) => result,
                Err(payload) => {
                    tracing::warn!(
                        "get_apps_from_shortcuts: panic while processing shortcut {:?}: {}",
                        shortcut_path_for_log,
                        panic_message(payload)
                    );
                    Ok(None)
                }
            }
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

/// 快捷方式过滤器：双轴检查。
/// 1) 快捷方式自身的名字（`卸载钉钉.lnk` —— Flow Launcher 也是这么做的）；
/// 2) 目标路径（`target_path` 由调用方解析一次后传入，避免重复解析）。
fn should_filter_shortcut(
    shortcut_name: &str,
    link: &ShellLink,
    target_path: Option<&str>,
) -> bool {
    if is_uninstaller_name(shortcut_name) {
        return true;
    }
    if let Some(target) = target_path {
        return !is_launchable_target(target);
    }
    // 目标无法解析时，仍对原始 local_base_path 做一次宽松检查。
    if let Some(link_info) = link.link_info().as_ref() {
        if let Some(local_path) = link_info.local_base_path() {
            if looks_like_uninstaller(local_path) {
                return true;
            }
        }
    }
    false
}

/// 拼接 lnk 的 `local_base_path` 与 `common_path_suffix`。
///
/// `common_path_suffix` 常带前导反斜杠（如 `\DingTalk.exe`）。这类「有根
/// 无前缀」的路径传给 `PathBuf::push` 时，Rust 会保留盘符但丢掉 base 目录
/// （`C:\Program Files\DingTalk` + `\DingTalk.exe` → `C:\DingTalk.exe`），
/// 因此这里手动去掉两端分隔符后再拼接。
fn join_link_target(base: &str, suffix: &str) -> String {
    let clean_suffix = suffix.trim_start_matches(['\\', '/']);
    if clean_suffix.is_empty() {
        return base.to_string();
    }
    let clean_base = base.trim_end_matches(['\\', '/']);
    format!("{}\\{}", clean_base, clean_suffix)
}

/// 解析 lnk 目标：拼接 `local_base_path` + `common_path_suffix`（关键修复）。
/// 原实现只取 `local_base_path`，当快捷方式把目标拆成「目录 + 文件名」时，
/// 得到的是目录，随后被扩展名检查丢弃 —— 这正是钉钉「只剩卸载项」的成因。
fn extract_target_from_lnk_with_link(link: &ShellLink) -> Option<String> {
    let link_info = link.link_info().as_ref()?;

    // Unicode 变体优先（中文路径在 WINDOWS_1252 下会乱码）。
    let base = link_info
        .local_base_path_unicode()
        .as_ref()
        .map(|s| s.to_string())
        .or_else(|| link_info.local_base_path().map(|s| s.to_string()))?;
    let base = expand_env_vars(trim_quotes(&base));

    let suffix = link_info
        .common_path_suffix_unicode()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| link_info.common_path_suffix().to_string());
    let suffix = trim_quotes(&suffix);

    // 组合完整目标：base 本身已是完整路径时 suffix 通常为空。
    let combined = join_link_target(&base, &suffix);

    if Path::new(&combined).exists() {
        Some(combined)
    } else if Path::new(&base).exists() {
        // 兼容：某些 lnk 的 suffix 含参数，整体不存在时退回 base。
        Some(base)
    } else {
        None
    }
}

// extract_icon_from_shortcut 的新版本，接受 ShellLink 引用和 target_path
// 保持同步，因为会在 spawn_blocking 内部调用
fn extract_icon_from_shortcut_with_link(link: &ShellLink, target_path: &str) -> Option<String> {
    // 尝试获取快捷方式的图标路径
    if let Some(icon_location) = link.string_data().icon_location().as_ref() {
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
                            // Per user feedback, requesting 32x32 is crucial.
                            // It seems to trigger a different Windows API path that returns the icon
                            // without the tile background, which is exactly what we need.
                            let size = windows::Win32::Foundation::SIZE { cx: 32, cy: 32 };
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
                    let mut keywords = vec![];
                    // Heuristic to detect UWP apps: their path often contains an underscore
                    // followed by a publisher hash. This is not foolproof but better than nothing.
                    if path.contains('_') && path.contains('!') {
                        if let Some(keyword) = path
                            .split('_')
                            .next()
                            .and_then(|s| s.split('.').last())
                            .map(|s| s.to_lowercase())
                        {
                            keywords.push(keyword);
                        }
                    }
                    apps.push(AppInfo {
                        name: normalize_app_name(&name),
                        keywords,
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

    // 三个来源统一合并：按规范化名称去重，但合并元数据而不是简单丢弃后来者。
    // 关键：来源过滤已在各 scan 阶段剔除卸载器，因此这里存活的都是合法启动项；
    // 合并时按来源质量决定 path/name 的取舍，并补齐图标与关键词。
    let mut all_apps = Vec::new();
    all_apps.extend(hkey_apps);
    all_apps.extend(shortcut_apps);
    all_apps.extend(apps_folder_apps);

    Ok(deduplicate_apps(all_apps))
}

fn origin_rank(app: &AppInfo) -> u8 {
    match app.origin {
        Some(AppOrigin::Uwp) => 3, // AppsFolder：Shell 策展的「所有应用」，最可信
        Some(AppOrigin::Shortcut) => 2, // 开始菜单快捷方式
        Some(AppOrigin::Hkey) => 1, // 注册表（路径可能是卸载器，已过滤）
        None => 0,
    }
}

/// 把新条目合并进已存在的同名条目：
/// - 图标：低优先级来源先出现时，高优先级来源的图标要能覆盖它；高优先级
///   来源没有图标时则保留已有图标；
/// - 关键词：取并集；
/// - path/name：来源质量更高者胜出，避免坏路径遮住好路径。
fn merge_app(existing: &mut AppInfo, incoming: AppInfo) {
    if origin_rank(&incoming) > origin_rank(existing) {
        existing.path = incoming.path;
        existing.name = incoming.name;
        existing.origin = incoming.origin;
        if incoming.icon.is_some() {
            existing.icon = incoming.icon.clone();
        }
    } else if existing.icon.is_none() && incoming.icon.is_some() {
        existing.icon = incoming.icon.clone();
    }
    for kw in incoming.keywords.iter() {
        if !existing.keywords.contains(kw) {
            existing.keywords.push(kw.clone());
        }
    }
}

/// 按应用身份去重（纯函数，便于测试）。
/// 身份键 = 规范化后的名称（小写），保留「一个应用一条」的交互习惯；
/// 与旧实现不同点：来源之间不再互相吞掉，而是合并元数据 + 择优取路径。
pub(crate) fn deduplicate_apps(apps: Vec<AppInfo>) -> Vec<AppInfo> {
    let mut unique_apps: HashMap<String, AppInfo> = HashMap::new();
    for app in apps {
        // 最后一道防线：即使某个来源漏过了过滤，这里再按名称 + 路径双轴剔除卸载器。
        if is_uninstaller_name(&app.name) {
            continue;
        }
        if let Some(ref path) = app.path {
            if !is_launchable_target(path) {
                continue;
            }
        }
        let key = normalize_app_name(&app.name).to_lowercase();
        match unique_apps.get_mut(&key) {
            Some(existing) => merge_app(existing, app),
            None => {
                unique_apps.insert(key, app);
            }
        }
    }

    let mut result: Vec<AppInfo> = unique_apps.into_values().collect();
    // HashMap 顺序不确定，排序保证结果稳定。
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
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
        // 使用 CREATE_NEW_PROCESS_GROUP 和 DETACHED_PROCESS 标志让进程独立运行
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        Command::new("cmd")
            .args(&["/C", "start", "", path])
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_valid_lnk_file_non_existent() {
        let path = PathBuf::from("this_file_does_not_exist.lnk");
        assert!(!is_valid_lnk_file(&path));
    }

    #[test]
    fn test_is_valid_lnk_file_too_small() {
        let mut file = NamedTempFile::new().unwrap();
        // size 10 bytes (minimum required is 20)
        file.write_all(b"1234567890").unwrap();
        assert!(!is_valid_lnk_file(file.path()));
    }

    #[test]
    fn test_is_valid_lnk_file_invalid_header_size() {
        let mut file = NamedTempFile::new().unwrap();
        let mut header = [0u8; 20];
        // Wrong HeaderSize: [0x4D, 0x00, 0x00, 0x00]
        header[0..4].copy_from_slice(&[0x4D, 0x00, 0x00, 0x00]);
        // Correct LinkCLSID
        header[4..20].copy_from_slice(&[
            0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x46,
        ]);
        file.write_all(&header).unwrap();
        assert!(!is_valid_lnk_file(file.path()));
    }

    #[test]
    fn test_is_valid_lnk_file_invalid_clsid() {
        let mut file = NamedTempFile::new().unwrap();
        let mut header = [0u8; 20];
        // Correct HeaderSize
        header[0..4].copy_from_slice(&[0x4C, 0x00, 0x00, 0x00]);
        // Incorrect LinkCLSID
        header[4..20].copy_from_slice(&[
            0x99, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x46,
        ]);
        file.write_all(&header).unwrap();
        assert!(!is_valid_lnk_file(file.path()));
    }

    #[test]
    fn test_is_valid_lnk_file_valid() {
        let mut file = NamedTempFile::new().unwrap();
        let mut header = [0u8; 20];
        // Correct HeaderSize
        header[0..4].copy_from_slice(&[0x4C, 0x00, 0x00, 0x00]);
        // Correct LinkCLSID
        header[4..20].copy_from_slice(&[
            0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x46,
        ]);
        file.write_all(&header).unwrap();
        assert!(is_valid_lnk_file(file.path()));
    }

    #[test]
    fn test_app_from_shortcut_guard_invalid_file() {
        // Test 1: Non-existent file
        let non_existent_path = PathBuf::from("this_file_does_not_exist_at_all.lnk");
        let res = app_from_shortcut(non_existent_path);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());

        // Test 2: File is too small (e.g. 10 bytes)
        let mut too_small_file = NamedTempFile::new().unwrap();
        too_small_file.write_all(b"1234567890").unwrap();
        let res = app_from_shortcut(too_small_file.path().to_path_buf());
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());

        // Test 3: Corrupted header (e.g. matching the Sentry crash pattern with size 1556 bytes)
        let mut corrupted_file = NamedTempFile::new().unwrap();
        let mut content = vec![0u8; 1556];
        // Header contains invalid values mimicking Sentry crash where offset starts out of bounds
        content[0..4].copy_from_slice(&[0x5C, 0x43, 0x44, 0x45]); // '\CDE'
        corrupted_file.write_all(&content).unwrap();

        // Should return Ok(None) safely instead of panicking
        let res = app_from_shortcut(corrupted_file.path().to_path_buf());
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    // -----------------------------------------------------------------------
    // 卸载器识别：三个真实案例覆盖
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_uninstaller_name_covers_all_common_forms() {
        // 大小写不敏感；uninst / unins000 / uninstall 三种常见文件名都要命中
        for name in [
            "uninst.exe",
            "UNINST.EXE",
            "uninst000.exe",
            "unins000.exe",
            "Uninstall.exe",
            "uninstall.exe",
            "卸载百度网盘",
            "卸载钉钉",
            "卸載騰訊",
            "卸载钉钉.lnk",
            "Uninstall DingTalk.lnk",
        ] {
            assert!(
                is_uninstaller_name(name),
                "expected '{}' to be recognized as uninstaller",
                name
            );
        }
    }

    #[test]
    fn test_is_uninstaller_name_keeps_normal_apps() {
        for name in [
            "百度网盘",
            "钉钉",
            "小米电脑管家",
            "XiaomiPCManager",
            "Google Chrome",
            "visual studio code",
        ] {
            assert!(
                !is_uninstaller_name(name),
                "normal app '{}' must NOT be flagged as uninstaller",
                name
            );
        }
    }

    #[test]
    fn test_looks_like_uninstaller_checks_only_filename() {
        // 路径中的目录名字含 "uninstall" 不应误伤合法应用
        assert!(!looks_like_uninstaller(
            r"C:\Program Files\uninstall-tools\MyApp\myapp.exe"
        ));
        // 但文件名本身就是卸载器，要拦住
        assert!(looks_like_uninstaller(
            r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\Uninstall.exe"
        ));
        assert!(looks_like_uninstaller(
            r"C:\Users\zwj\AppData\Roaming\baidu\BaiduNetdisk\uninst.exe"
        ));
        assert!(looks_like_uninstaller(r"C:\Program Files\Foo\unins000.exe"));
    }

    #[test]
    fn test_is_non_app_executable() {
        assert!(is_non_app_executable(r"C:\Windows\System32\msiexec.exe"));
        assert!(is_non_app_executable(r"C:\Windows\System32\rundll32.exe"));
        assert!(is_non_app_executable(r"C:\Windows\System32\cmd.exe"));
        // setup.exe 不再是全局非应用执行器：它可能是合法应用的启动目标
        assert!(!is_non_app_executable(r"C:\App\setup.exe"));
        assert!(!is_non_app_executable(r"C:\Program Files\App\app.exe"));
    }

    #[test]
    fn test_installer_only_excluded_in_uninstall_string_fallback() {
        // 安装器判断只用于 UninstallString 兜底，不影响 DisplayIcon / 快捷方式来源
        assert!(is_installer_executable(r"C:\App\setup.exe"));
        assert!(is_installer_executable(r"C:\App\INSTALL.EXE"));
        assert!(!is_installer_executable(r"C:\Program Files\App\app.exe"));
        // 合法的 setup.exe 作为 DisplayIcon 应被保留（范围蔓延修复）
        assert_eq!(
            extract_exe_path(Some(r"C:\Program Files\MyApp\setup.exe".into()), None, None)
                .as_deref(),
            Some(r"C:\Program Files\MyApp\setup.exe")
        );
        // 但 UninstallString 兜底里出现的 setup.exe 应被跳过
        assert_eq!(
            extract_exe_path(None, None, Some(r"C:\App\setup.exe /S".into())),
            None
        );
    }

    #[test]
    fn test_is_launchable_target() {
        assert!(is_launchable_target(r"C:\Program Files\App\app.exe"));
        assert!(is_launchable_target(r"C:\Program Files\App\setup.exe"));
        assert!(!is_launchable_target(r"C:\Program Files\App\uninst.exe"));
        assert!(!is_launchable_target(r"C:\Windows\System32\cmd.exe"));
    }

    // -----------------------------------------------------------------------
    // extract_exe_path：DisplayIcon / UninstallString 解析
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_exe_path_from_display_icon_variants() {
        // 普通 exe
        assert_eq!(
            extract_exe_path(Some(r"C:\Program Files\App\app.exe".into()), None, None).as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
        // 带引号
        assert_eq!(
            extract_exe_path(
                Some("\"C:\\Program Files\\App\\app.exe\"".into()),
                None,
                None
            )
            .as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
        // 带图标索引 `,0`
        assert_eq!(
            extract_exe_path(Some(r"C:\Program Files\App\app.exe,0".into()), None, None).as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
        // 带引号 + 图标索引
        assert_eq!(
            extract_exe_path(
                Some("\"C:\\Program Files\\App\\app.exe\",0".into()),
                None,
                None
            )
            .as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
    }

    #[test]
    fn test_extract_exe_path_rejects_uninstaller_display_icon() {
        // 小米电脑管家案例：DisplayIcon 指向 Uninstall.exe，不能作为启动目标
        assert_eq!(
            extract_exe_path(
                Some(r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\Uninstall.exe".into()),
                None,
                None,
            ),
            None
        );
    }

    #[test]
    fn test_extract_exe_path_uninstall_string_fallback() {
        // 百度网盘案例：UninstallString 指向 uninst.exe，回退分支必须跳过
        assert_eq!(
            extract_exe_path(
                None,
                None,
                Some(r"C:\Users\zwj\AppData\Roaming\baidu\BaiduNetdisk\uninst.exe".into()),
            ),
            None
        );
        // MSI 卸载：抓到 msiexec.exe 也要跳过
        assert_eq!(
            extract_exe_path(
                None,
                None,
                Some("MsiExec.exe /I{12345678-1234-1234-1234-123456789012}".into()),
            ),
            None
        );
        // rundll32 也跳过
        assert_eq!(
            extract_exe_path(
                None,
                None,
                Some(r"rundll32.exe C:\App\uninst.dll,Uninstall".into()),
            ),
            None
        );
        // 合法路径（无其它来源时）仍可作为兜底
        assert_eq!(
            extract_exe_path(None, None, Some(r"C:\Program Files\App\app.exe /U".into()))
                .as_deref(),
            Some(r"C:\Program Files\App\app.exe")
        );
    }

    // -----------------------------------------------------------------------
    // 名称归一化
    // -----------------------------------------------------------------------

    #[test]
    fn test_normalize_app_name_fullwidth_brackets() {
        assert_eq!(normalize_app_name("钉钉（桌面版）"), "钉钉");
        assert_eq!(normalize_app_name("微信【PC版】"), "微信");
        assert_eq!(normalize_app_name("Google Chrome (x86)"), "Google Chrome");
        assert_eq!(normalize_app_name("App 1.2.3"), "App");
    }

    // -----------------------------------------------------------------------
    // 快捷方式目标拼接
    // -----------------------------------------------------------------------

    #[test]
    fn test_join_link_target_suffix_with_leading_backslash() {
        // 回归用例：common_path_suffix 的前导反斜杠不能把 base 目录冲掉。
        assert_eq!(
            join_link_target(r"C:\Program Files\DingTalk", r"\DingTalk.exe"),
            r"C:\Program Files\DingTalk\DingTalk.exe"
        );
    }

    #[test]
    fn test_join_link_target_normalizes_redundant_separators() {
        assert_eq!(
            join_link_target(r"C:\Program Files\DingTalk\", r"\DingTalk.exe"),
            r"C:\Program Files\DingTalk\DingTalk.exe"
        );
        assert_eq!(
            join_link_target(r"C:\Program Files\DingTalk", "DingTalk.exe"),
            r"C:\Program Files\DingTalk\DingTalk.exe"
        );
    }

    #[test]
    fn test_join_link_target_returns_base_for_empty_suffix() {
        assert_eq!(
            join_link_target(r"C:\Program Files\DingTalk\DingTalk.exe", ""),
            r"C:\Program Files\DingTalk\DingTalk.exe"
        );
        assert_eq!(
            join_link_target(r"C:\Program Files\DingTalk\DingTalk.exe", r"\"),
            r"C:\Program Files\DingTalk\DingTalk.exe"
        );
    }

    // -----------------------------------------------------------------------
    // 去重：坏条目不能压掉好条目
    // -----------------------------------------------------------------------

    fn make_app(name: &str, path: &str, origin: AppOrigin, icon: Option<String>) -> AppInfo {
        AppInfo {
            name: name.to_string(),
            keywords: vec![],
            path: Some(path.to_string()),
            icon,
            origin: Some(origin),
        }
    }

    #[test]
    fn test_deduplicate_prefers_shortcut_over_registry_and_merges_icon() {
        // 同名两条：注册表（低优先级，带图标）与快捷方式（高优先级，无图标）。
        // 结果应保留快捷方式的正确 path/origin，同时合并注册表的图标。
        let apps = vec![
            make_app(
                "小米电脑管家",
                r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\XiaomiPCManager.exe",
                AppOrigin::Shortcut,
                None,
            ),
            make_app(
                "小米电脑管家",
                r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\XiaomiPCManager.exe",
                AppOrigin::Hkey,
                Some("BASE64_ICON".into()),
            ),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 1, "同名应用应合并为一条");
        assert_eq!(
            result[0].path.as_deref(),
            Some(r"C:\Program Files\MI\XiaomiPCManager\5.5.2.10\XiaomiPCManager.exe"),
            "快捷方式路径应胜出"
        );
        assert_eq!(
            result[0].icon.as_deref(),
            Some("BASE64_ICON"),
            "图标应被合并"
        );
        assert_eq!(result[0].origin, Some(AppOrigin::Shortcut));
    }

    #[test]
    fn test_deduplicate_higher_origin_replaces_lower_quality_icon() {
        // 注册表先出现（rank=1，低质量图标），随后 AppsFolder（rank=3）应覆盖
        // path/name/origin 以及图标。
        let apps = vec![
            make_app(
                "小米电脑管家",
                r"C:\Program Files\MI\XiaomiPCManager\XiaomiPcManager.exe",
                AppOrigin::Hkey,
                Some("LOW_QUALITY_ICON".into()),
            ),
            make_app(
                "小米电脑管家",
                "shell:AppsFolder\\XiaomiPCManager",
                AppOrigin::Uwp,
                Some("HIGH_QUALITY_ICON".into()),
            ),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].icon.as_deref(),
            Some("HIGH_QUALITY_ICON"),
            "高优先级来源的图标应覆盖低质量图标"
        );
        assert_eq!(
            result[0].path.as_deref(),
            Some("shell:AppsFolder\\XiaomiPCManager")
        );
        assert_eq!(result[0].origin, Some(AppOrigin::Uwp));
    }

    #[test]
    fn test_deduplicate_higher_origin_without_icon_keeps_existing_icon() {
        // 高优先级来源没有图标时，不能把已有的图标清空。
        let apps = vec![
            make_app(
                "钉钉",
                r"C:\Program Files\DingTalk\DingTalk.exe",
                AppOrigin::Hkey,
                Some("EXISTING_ICON".into()),
            ),
            make_app("钉钉", "shell:AppsFolder\\DingTalk", AppOrigin::Uwp, None),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].icon.as_deref(), Some("EXISTING_ICON"));
        assert_eq!(
            result[0].path.as_deref(),
            Some("shell:AppsFolder\\DingTalk")
        );
    }

    #[test]
    fn test_deduplicate_appsfolder_path_wins() {
        let apps = vec![
            make_app(
                "钉钉",
                r"C:\Program Files\DingTalk\DingTalk.exe",
                AppOrigin::Shortcut,
                None,
            ),
            make_app("钉钉", "shell:AppsFolder\\DingTalk", AppOrigin::Uwp, None),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].path.as_deref(),
            Some("shell:AppsFolder\\DingTalk"),
            "AppsFolder 来源优先级最高"
        );
        assert_eq!(result[0].origin, Some(AppOrigin::Uwp));
    }

    #[test]
    fn test_deduplicate_uninstaller_entries_dropped() {
        // 最后一道防线：漏网的卸载器条目（名称或路径）在去重阶段被剔除
        let apps = vec![
            make_app(
                "百度网盘",
                r"C:\App\BaiduNetdisk\BaiduNetdisk.exe",
                AppOrigin::Shortcut,
                None,
            ),
            make_app(
                "卸载百度网盘",
                r"C:\App\BaiduNetdisk\uninst.exe",
                AppOrigin::Shortcut,
                None,
            ),
            make_app(
                "某应用",
                r"C:\App\SomeApp\Uninstall.exe",
                AppOrigin::Hkey,
                None,
            ),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 1, "两条卸载器都应被剔除，只剩正常应用");
        assert_eq!(result[0].name, "百度网盘");
        assert!(result.iter().all(|a| !is_uninstaller_name(&a.name)));
    }

    #[test]
    fn test_deduplicate_keeps_distinct_apps() {
        let apps = vec![
            make_app(
                "百度网盘",
                r"C:\App\BaiduNetdisk\app.exe",
                AppOrigin::Shortcut,
                None,
            ),
            make_app(
                "钉钉",
                r"C:\App\DingTalk\app.exe",
                AppOrigin::Shortcut,
                None,
            ),
        ];
        let result = deduplicate_apps(apps);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "百度网盘");
        assert_eq!(result[1].name, "钉钉");
    }
}
