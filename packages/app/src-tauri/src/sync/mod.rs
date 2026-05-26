pub mod webdav;
pub mod zip_utils;

use tauri::{AppHandle, Emitter, Manager};
use whoami;

use crate::app_config::{AppConfigState, WebDavConfig};
use webdav::{LastSyncInfo, WebDavClient};

/// 创建 WebDAV 客户端辅助函数
fn get_webdav_client(config: &WebDavConfig) -> Result<WebDavClient, String> {
    if config.base_url.is_empty() {
        return Err("WebDAV 服务器地址不能为空".to_string());
    }
    if config.username.is_empty() {
        return Err("WebDAV 用户名不能为空".to_string());
    }
    if config.password.is_empty() {
        return Err("WebDAV 密码/应用密钥不能为空".to_string());
    }
    Ok(WebDavClient::new(
        config.base_url.clone(),
        config.username.clone(),
        config.password.clone(),
    ))
}

/// 测试 WebDAV 连接
#[tauri::command]
pub async fn test_webdav_connection(config: WebDavConfig) -> Result<(), String> {
    let client = get_webdav_client(&config)?;
    client.test_connection().await
}

/// 检查云端备份是否存在并返回时间戳
#[tauri::command]
pub async fn check_cloud_backup(config: WebDavConfig) -> Result<Option<LastSyncInfo>, String> {
    let client = get_webdav_client(&config)?;

    let folder = config
        .folder_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("onin");

    // 我们先尝试创建一次云端根目录，保证目录存在，这也是安全的
    let _ = client.create_directory(folder).await;

    let last_sync_path = format!("{}/last_sync.json", folder);

    // 拉取 last_sync.json 验证
    match client.download_file(&last_sync_path).await {
        Ok(Some(bytes)) => {
            let info: LastSyncInfo = serde_json::from_slice(&bytes)
                .map_err(|e| format!("解析云端同步元数据失败: {}", e))?;
            Ok(Some(info))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 触发 WebDAV 同步备份或恢复
///
/// mode: "backup" 或 "restore"
#[tauri::command]
pub async fn trigger_webdav_sync(
    app: AppHandle,
    mode: String,
) -> Result<Option<LastSyncInfo>, String> {
    // 1. 从内存状态中获取最新的 WebDAV 配置
    let config = {
        let state = app.state::<AppConfigState>();
        let config_lock = state.0.lock().map_err(|e| e.to_string())?;
        config_lock.webdav.clone()
    };

    let client = get_webdav_client(&config)?;

    // 获取本地 app_data 路径和临时压缩包保存路径
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用配置目录失败: {}", e))?;

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("获取缓存目录失败: {}", e))?;

    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    }

    let temp_zip_path = cache_dir.join("onin_backup_temp.zip");

    let folder = config
        .folder_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("onin");

    let backup_path = format!("{}/onin_backup.zip", folder);
    let last_sync_path = format!("{}/last_sync.json", folder);

    // 确保云端目录存在
    let _ = client.create_directory(folder).await;

    if mode == "backup" {
        println!("[sync] 开始打包本地数据...");
        // A. 打包本地数据到临时 ZIP，自动应用黑名单过滤
        zip_utils::pack_app_data_to_zip(&app_data_dir, &temp_zip_path)?;

        // B. 读取 ZIP 字节码并上传
        let zip_bytes =
            std::fs::read(&temp_zip_path).map_err(|e| format!("读取临时备份包失败: {}", e))?;

        println!("[sync] 上传备份包到云端...");
        client.upload_file(&backup_path, zip_bytes).await?;

        // C. 生成并写入本地与云端的同步信息 last_sync.json
        let sync_time = chrono::Utc::now().to_rfc3339();
        let device_name = whoami::devicename();

        let sync_info = LastSyncInfo {
            last_sync_time: sync_time.clone(),
            device_id: device_name,
        };

        let info_bytes = serde_json::to_vec_pretty(&sync_info)
            .map_err(|e| format!("序列化同步元数据失败: {}", e))?;

        client.upload_file(&last_sync_path, info_bytes).await?;

        // 清理临时 ZIP
        let _ = std::fs::remove_file(&temp_zip_path);
        println!("[sync] 备份全部完成！");

        Ok(Some(sync_info))
    } else if mode == "restore" {
        println!("[sync] 从云端下载备份包...");
        // A. 下载云端备份包到本地临时文件
        match client.download_file(&backup_path).await? {
            Some(zip_bytes) => {
                std::fs::write(&temp_zip_path, zip_bytes)
                    .map_err(|e| format!("保存云端备份包失败: {}", e))?;

                println!("[sync] 开始解压并恢复本地数据...");
                // B. 解压覆盖到本地 app_data_dir，自动应用黑名单排除本地专属设置
                zip_utils::unpack_zip_to_app_data(&temp_zip_path, &app_data_dir)?;

                // 清理临时文件
                let _ = std::fs::remove_file(&temp_zip_path);

                // C. 热重载内存配置与通知前端刷新
                println!("[sync] 热重载应用配置...");
                if let Ok(new_config) = crate::app_config::load_config(&app) {
                    if let Some(state) = app.try_state::<AppConfigState>() {
                        if let Ok(mut lock) = state.0.lock() {
                            *lock = new_config;
                        }
                    }
                }

                // 广播全局设置载入事件，通知 Svelte 前端触发配置重绘与插件重扫
                let _ = app.emit("app-config-loaded", ());

                println!("[sync] 恢复全部完成！");

                // 获取最新的云端同步信息
                if let Ok(Some(bytes)) = client.download_file(&last_sync_path).await {
                    if let Ok(info) = serde_json::from_slice::<LastSyncInfo>(&bytes) {
                        return Ok(Some(info));
                    }
                }

                Ok(None)
            }
            None => Err("云端没有任何备份文件，请在旧设备先点击“立即备份”".to_string()),
        }
    } else {
        Err(format!("未知的同步指令: {}", mode))
    }
}
