use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataFileInfo {
    pub id: String,
    pub name: String,
    pub category: String, // "main" | "plugin" | "extension"
    pub rel_path: String,
    pub absolute_path: String,
    pub size_bytes: u64,
    pub is_json: bool,
    pub is_image: bool,
    pub is_text: bool,
}

/// 安全拼接路径，防止路径穿越 (Path Traversal)
pub fn safe_resolve_path(app: &AppHandle, rel_path: &str) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let target_path = data_dir.join(rel_path);

    // 获取并比对 canonicalized 物理路径
    if let Ok(canonical_data_dir) = std::fs::canonicalize(&data_dir) {
        if target_path.exists() {
            if let Ok(canonical_target) = std::fs::canonicalize(&target_path) {
                if !canonical_target.starts_with(&canonical_data_dir) {
                    return Err("拒绝访问：检测到非法路径穿越".to_string());
                }
            } else {
                return Err("拒绝访问：无效的路径".to_string());
            }
        } else {
            // 文件不存在时，比对它的父目录物理路径是否在 data_dir 内
            if let Some(parent) = target_path.parent() {
                if parent.exists() {
                    if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
                        if !canonical_parent.starts_with(&canonical_data_dir) {
                            return Err("拒绝访问：检测到非法路径穿越".to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(target_path)
}

/// 递归扫描指定目录下的文件
fn scan_dir_recursive(
    base_dir: &Path,
    current_dir: &Path,
    category: &str,
    prefix_label: &str,
    result: &mut Vec<AppDataFileInfo>,
) {
    if let Ok(entries) = std::fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(base_dir, &path, category, prefix_label, result);
            } else if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base_dir) {
                    if let Some(rel_str) = rel.to_str() {
                        let rel_path_unix = rel_str.replace('\\', "/");
                        // 忽略 window_states.json，因为已在静态列表中定义
                        if rel_path_unix == "plugin_data/window_states.json" {
                            continue;
                        }

                        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let extension = path
                            .extension()
                            .map_or("", |ext| ext.to_str().unwrap_or(""))
                            .to_lowercase();
                        let is_json = extension == "json";
                        let is_image = matches!(
                            extension.as_str(),
                            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
                        );
                        let is_text = is_json
                            || matches!(
                                extension.as_str(),
                                "txt" | "log" | "md" | "jsonl" | "conf" | "cfg"
                            );
                        let absolute_path = path.to_string_lossy().to_string();

                        // 格式化展示名称
                        let display_name = if rel_path_unix.starts_with("plugin_data/") {
                            let sub = rel_path_unix.trim_start_matches("plugin_data/");
                            format!("插件数据: {}", sub)
                        } else if rel_path_unix.starts_with("extensions/") {
                            let sub = rel_path_unix.trim_start_matches("extensions/");
                            format!("扩展数据: {}", sub)
                        } else {
                            format!("{}: {}", prefix_label, rel_path_unix)
                        };

                        result.push(AppDataFileInfo {
                            id: format!("{}:{}", category, rel_path_unix),
                            name: display_name,
                            category: category.to_string(),
                            rel_path: rel_path_unix,
                            absolute_path,
                            size_bytes,
                            is_json,
                            is_image,
                            is_text,
                        });
                    }
                }
            }
        }
    }
}

/// 获取应用数据目录的绝对路径
#[tauri::command]
pub fn get_app_data_dir_path(app: AppHandle) -> Result<String, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.to_string_lossy().to_string())
}

/// 列出所有可预览和管理的本地应用数据文件
#[tauri::command]
pub fn list_app_data_files(app: AppHandle) -> Result<Vec<AppDataFileInfo>, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    // 1. 静态主应用/核心配置文件定义
    let static_files = vec![
        ("app_config", "app_config.json", "应用通用配置", "main"),
        ("ai_config", "ai_config.json", "AI 能力配置", "main"),
        ("commands", "commands.json", "自定义指令配置", "main"),
        (
            "dynamic_commands",
            "dynamic_commands.json",
            "动态指令配置",
            "main",
        ),
        (
            "file_commands",
            "file_commands.json",
            "文件启动指令",
            "main",
        ),
        (
            "command_usage",
            "command_usage.json",
            "指令使用频率统计",
            "main",
        ),
        ("shortcuts", "shortcuts.json", "全局快捷键配置", "main"),
        (
            "plugin_states",
            "plugin_states.json",
            "插件启用状态配置",
            "plugin",
        ),
        (
            "window_states",
            "plugin_data/window_states.json",
            "插件窗口状态配置",
            "plugin",
        ),
    ];

    for (id, rel_path, display_name, category) in static_files {
        let full_path = data_dir.join(rel_path);
        if full_path.is_file() {
            let size_bytes = std::fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0);
            let absolute_path = full_path.to_string_lossy().to_string();
            result.push(AppDataFileInfo {
                id: id.to_string(),
                name: display_name.to_string(),
                category: category.to_string(),
                rel_path: rel_path.to_string(),
                absolute_path,
                size_bytes,
                is_json: true,
                is_image: false,
                is_text: true,
            });
        }
    }

    // 2. 动态扫描 plugin_settings 文件夹下的插件配置
    let settings_dir = data_dir.join("plugin_settings");
    if settings_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(settings_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        let plugin_id = file_name.trim_end_matches(".json");
                        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let rel_path = format!("plugin_settings/{}", file_name);
                        let absolute_path = path.to_string_lossy().to_string();
                        result.push(AppDataFileInfo {
                            id: format!("plugin_settings:{}", plugin_id),
                            name: format!("插件设置: {}", plugin_id),
                            category: "plugin".to_string(),
                            rel_path,
                            absolute_path,
                            size_bytes,
                            is_json: true,
                            is_image: false,
                            is_text: true,
                        });
                    }
                }
            }
        }
    }

    // 3. 动态扫描 plugin_data 文件夹下的自定义插件存储数据
    let plugin_data_dir = data_dir.join("plugin_data");
    if plugin_data_dir.is_dir() {
        scan_dir_recursive(
            &data_dir,
            &plugin_data_dir,
            "plugin",
            "插件数据",
            &mut result,
        );
    }

    // 4. 动态扫描 extensions 文件夹下的内置扩展数据
    let extensions_dir = data_dir.join("extensions");
    if extensions_dir.is_dir() {
        scan_dir_recursive(
            &data_dir,
            &extensions_dir,
            "extension",
            "扩展数据",
            &mut result,
        );
    }

    Ok(result)
}

/// 读取指定相对路径的数据文件内容
#[tauri::command]
pub fn read_app_data_file_content(app: AppHandle, rel_path: String) -> Result<String, String> {
    let full_path = safe_resolve_path(&app, &rel_path)?;
    if !full_path.is_file() {
        return Err(format!("文件不存在或不是普通文件: {}", rel_path));
    }

    let content =
        std::fs::read_to_string(&full_path).map_err(|e| format!("读取文件失败: {}", e))?;

    Ok(content)
}
