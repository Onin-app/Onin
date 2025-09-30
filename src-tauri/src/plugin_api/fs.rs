use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs;

// 线程本地存储用于保存当前插件ID（复用 storage.rs 的逻辑）
use super::storage::{get_current_plugin_id, StorageError};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystemError {
    pub name: String,
    pub message: String,
    pub code: Option<String>,
    pub path: Option<String>,
}

impl From<String> for FileSystemError {
    fn from(message: String) -> Self {
        FileSystemError {
            name: "FileSystemError".to_string(),
            message,
            code: None,
            path: None,
        }
    }
}

impl From<&str> for FileSystemError {
    fn from(message: &str) -> Self {
        FileSystemError {
            name: "FileSystemError".to_string(),
            message: message.to_string(),
            code: None,
            path: None,
        }
    }
}

impl From<std::io::Error> for FileSystemError {
    fn from(error: std::io::Error) -> Self {
        FileSystemError {
            name: "FileSystemError".to_string(),
            message: error.to_string(),
            code: Some(format!("{:?}", error.kind())),
            path: None,
        }
    }
}

impl From<StorageError> for FileSystemError {
    fn from(error: StorageError) -> Self {
        FileSystemError {
            name: "FileSystemError".to_string(),
            message: error.message,
            code: None,
            path: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    #[serde(rename = "isFile")]
    pub is_file: bool,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    pub size: u64,
    #[serde(rename = "modifiedTime")]
    pub modified_time: u64,
    #[serde(rename = "createdTime")]
    pub created_time: u64,
}

// 获取插件文件系统根目录
fn get_plugin_fs_root(app: &AppHandle, plugin_id: &str) -> Result<PathBuf, FileSystemError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| FileSystemError::from(format!("Failed to get app data dir: {}", e)))?;

    let plugin_dir = app_data_dir.join("plugin_data").join(plugin_id);
    Ok(plugin_dir)
}

// 解析相对路径到绝对路径，确保在插件目录内
fn resolve_plugin_path(
    app: &AppHandle,
    plugin_id: &str,
    relative_path: &str,
) -> Result<PathBuf, FileSystemError> {
    let plugin_root = get_plugin_fs_root(app, plugin_id)?;
    let resolved_path = plugin_root.join(relative_path);

    // 确保插件根目录存在
    if !plugin_root.exists() {
        std::fs::create_dir_all(&plugin_root).map_err(|e| {
            FileSystemError::from(format!("Failed to create plugin directory: {}", e))
        })?;
    }

    // 安全检查：确保路径在插件目录内
    if !resolved_path.starts_with(&plugin_root) {
        return Err(FileSystemError::from("Path is outside plugin directory"));
    }

    Ok(resolved_path)
}

// 获取当前插件ID的包装函数
fn get_current_plugin_id_fs(app: &AppHandle) -> Result<String, FileSystemError> {
    get_current_plugin_id(app).map_err(FileSystemError::from)
}

#[tauri::command]
pub async fn plugin_fs_read_file(app: AppHandle, path: String) -> Result<String, FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let file_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    let content = fs::read_to_string(&file_path).await?;

    println!("[FS] Read file '{}' for plugin '{}'", path, plugin_id);
    Ok(content)
}

#[tauri::command]
pub async fn plugin_fs_write_file(
    app: AppHandle,
    path: String,
    content: String,
) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let file_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    // 确保父目录存在
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(&file_path, content).await?;

    println!("[FS] Wrote file '{}' for plugin '{}'", path, plugin_id);
    Ok(())
}

#[tauri::command]
pub async fn plugin_fs_exists(app: AppHandle, path: String) -> Result<bool, FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let file_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    let exists = file_path.exists();

    println!(
        "[FS] Check exists '{}' for plugin '{}': {}",
        path, plugin_id, exists
    );
    Ok(exists)
}

#[tauri::command]
pub async fn plugin_fs_create_dir(
    app: AppHandle,
    path: String,
    recursive: bool,
) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let dir_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    if recursive {
        fs::create_dir_all(&dir_path).await?;
    } else {
        fs::create_dir(&dir_path).await?;
    }

    println!(
        "[FS] Created dir '{}' for plugin '{}' (recursive: {})",
        path, plugin_id, recursive
    );
    Ok(())
}

#[tauri::command]
pub async fn plugin_fs_list_dir(
    app: AppHandle,
    path: String,
) -> Result<Vec<FileInfo>, FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let dir_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    let mut entries = fs::read_dir(&dir_path).await?;
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let relative_path = if path == "." || path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", path, file_name)
        };

        let modified_time = metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let created_time = metadata
            .created()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        files.push(FileInfo {
            name: file_name,
            path: relative_path,
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
            modified_time,
            created_time,
        });
    }

    println!(
        "[FS] Listed dir '{}' for plugin '{}': {} items",
        path,
        plugin_id,
        files.len()
    );
    Ok(files)
}

#[tauri::command]
pub async fn plugin_fs_delete_file(app: AppHandle, path: String) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let file_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    fs::remove_file(&file_path).await?;

    println!("[FS] Deleted file '{}' for plugin '{}'", path, plugin_id);
    Ok(())
}

#[tauri::command]
pub async fn plugin_fs_delete_dir(
    app: AppHandle,
    path: String,
    recursive: bool,
) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let dir_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    if recursive {
        fs::remove_dir_all(&dir_path).await?;
    } else {
        fs::remove_dir(&dir_path).await?;
    }

    println!(
        "[FS] Deleted dir '{}' for plugin '{}' (recursive: {})",
        path, plugin_id, recursive
    );
    Ok(())
}

#[tauri::command]
pub async fn plugin_fs_get_file_info(
    app: AppHandle,
    path: String,
) -> Result<FileInfo, FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let file_path = resolve_plugin_path(&app, &plugin_id, &path)?;

    let metadata = fs::metadata(&file_path).await?;
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let modified_time = metadata
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let created_time = metadata
        .created()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let file_info = FileInfo {
        name: file_name,
        path: path.clone(),
        is_file: metadata.is_file(),
        is_directory: metadata.is_dir(),
        size: metadata.len(),
        modified_time,
        created_time,
    };

    println!("[FS] Got file info '{}' for plugin '{}'", path, plugin_id);
    Ok(file_info)
}

#[tauri::command]
pub async fn plugin_fs_copy_file(
    app: AppHandle,
    source_path: String,
    dest_path: String,
) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let source_file_path = resolve_plugin_path(&app, &plugin_id, &source_path)?;
    let dest_file_path = resolve_plugin_path(&app, &plugin_id, &dest_path)?;

    // 确保目标目录存在
    if let Some(parent) = dest_file_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::copy(&source_file_path, &dest_file_path).await?;

    println!(
        "[FS] Copied file '{}' to '{}' for plugin '{}'",
        source_path, dest_path, plugin_id
    );
    Ok(())
}

#[tauri::command]
pub async fn plugin_fs_move_file(
    app: AppHandle,
    source_path: String,
    dest_path: String,
) -> Result<(), FileSystemError> {
    let plugin_id = get_current_plugin_id_fs(&app)?;
    let source_file_path = resolve_plugin_path(&app, &plugin_id, &source_path)?;
    let dest_file_path = resolve_plugin_path(&app, &plugin_id, &dest_path)?;

    // 确保目标目录存在
    if let Some(parent) = dest_file_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::rename(&source_file_path, &dest_file_path).await?;

    println!(
        "[FS] Moved file '{}' to '{}' for plugin '{}'",
        source_path, dest_path, plugin_id
    );
    Ok(())
}
