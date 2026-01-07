//! 文件系统 API 处理器

use crate::js_runtime::types::InvokeResult;
use crate::plugin_api;
use tauri::AppHandle;

/// 读取文件内容
pub async fn handle_read_file(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_read_file(app_handle, path).await {
        Ok(content) => super::ok_value(serde_json::json!(content)),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 写入文件内容
pub async fn handle_write_file(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let content = match super::require_str(&arg, "content") {
        Ok(c) => c,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_write_file(app_handle, path, content).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 检查文件/目录是否存在
pub async fn handle_exists(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_exists(app_handle, path).await {
        Ok(exists) => super::ok_value(serde_json::json!(exists)),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 创建目录
pub async fn handle_create_dir(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let recursive = super::get_bool(&arg, "recursive", true);

    match plugin_api::fs::plugin_fs_create_dir(app_handle, path, recursive).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 列出目录内容
pub async fn handle_list_dir(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_list_dir(app_handle, path).await {
        Ok(files) => super::ok_value(serde_json::json!(files)),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 删除文件
pub async fn handle_delete_file(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_delete_file(app_handle, path).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 删除目录
pub async fn handle_delete_dir(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let recursive = super::get_bool(&arg, "recursive", false);

    match plugin_api::fs::plugin_fs_delete_dir(app_handle, path, recursive).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 获取文件信息
pub async fn handle_get_file_info(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let path = match super::require_str(&arg, "path") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_get_file_info(app_handle, path).await {
        Ok(info) => super::ok_value(serde_json::json!(info)),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 复制文件
pub async fn handle_copy_file(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let source_path = match super::require_str(&arg, "sourcePath") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let dest_path = match super::require_str(&arg, "destPath") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_copy_file(app_handle, source_path, dest_path).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}

/// 移动文件
pub async fn handle_move_file(app_handle: AppHandle, arg: serde_json::Value) -> InvokeResult {
    let source_path = match super::require_str(&arg, "sourcePath") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let dest_path = match super::require_str(&arg, "destPath") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match plugin_api::fs::plugin_fs_move_file(app_handle, source_path, dest_path).await {
        Ok(_) => super::ok_null(),
        Err(e) => super::err_fmt("File system", e),
    }
}
