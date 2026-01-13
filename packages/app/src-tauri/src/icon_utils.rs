#[cfg(target_os = "windows")]
use crate::installed_apps::exe_to_icon;

#[cfg(target_os = "windows")]
pub fn extract_icon_from_path(path: &str) -> Option<String> {
    if path.to_lowercase().ends_with(".exe") {
        exe_to_icon::extract_icon_from_exe(path)
    } else {
        // Use SHGetFileInfo to get icon for any file type
        exe_to_icon::extract_icon_from_file(path)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn extract_icon_from_path(_path: &str) -> Option<String> {
    // TODO: Implement icon extraction for other platforms
    None
}
