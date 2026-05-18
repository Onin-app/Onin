use futures::StreamExt;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;
use tokio::sync::Mutex;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    downloaded: u64,
    total: Option<u64>,
    percent: Option<f64>,
}

/// 包装下载取消令牌
pub struct UpdateCancelToken(pub Arc<Mutex<Option<oneshot::Sender<()>>>>);

/// 下载最新安装包并自动执行就地覆盖升级/热替换
#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    client: State<'_, reqwest::Client>,
    cancel_token: State<'_, UpdateCancelToken>,
    url: String,
) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();

    // 确定下载文件后缀名
    let extension = if url.contains(".deb") {
        "deb"
    } else if url.contains(".dmg") {
        "dmg"
    } else if url.contains(".AppImage") {
        "AppImage"
    } else {
        "msi"
    };

    let temp_file_path = temp_dir.join(format!("onin_setup.{}", extension));

    // 解决 5：开始下载前，主动检测并清理上一次可能遗留的旧临时安装包，实现自我净化
    if temp_file_path.exists() {
        let _ = std::fs::remove_file(&temp_file_path);
    }

    // 解决 7：创建 Cancellation Token 并更新到全局状态中
    let (tx, mut rx) = oneshot::channel::<()>();
    {
        let mut token = cancel_token.0.lock().await;
        *token = Some(tx);
    }

    // 解决 6：重用托管的全局 reqwest::Client
    let response = client
        .get(&url)
        .header("User-Agent", "Onin-Updater")
        .send()
        .await
        .map_err(|e| {
            let _ = cancel_token.0.try_lock().map(|mut t| *t = None);
            format!("请求下载链接失败: {}", e)
        })?;

    let total_size = response.content_length();
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let mut file = File::create(&temp_file_path).map_err(|e| {
        let _ = cancel_token.0.try_lock().map(|mut t| *t = None);
        format!("创建临时文件失败: {}", e)
    })?;

    while let Some(item) = stream.next().await {
        // 解决 7：高频检测取消状态，如用户取消则清理文件并退回
        if rx.try_recv().is_ok() {
            drop(file);
            let _ = std::fs::remove_file(&temp_file_path);
            return Err("下载已被用户取消".to_string());
        }

        let chunk = item.map_err(|e| format!("下载数据流出错: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = total_size.map(|total| (downloaded as f64 / total as f64) * 100.0);
        let _ = app.emit(
            "update-progress",
            ProgressPayload {
                downloaded,
                total: total_size,
                percent,
            },
        );
    }

    // 解决 4：遵循 idiomatic 规范，删除多余的 drop(file)，当文件离开 scope 后自动由析构释放。
    // 解决 3：将事件名称重构为 "update-downloaded"，以精确反映“下载完毕、即将拉起安装”的实际物理状态
    let _ = app.emit("update-downloaded", ());

    // 清除取消令牌
    {
        let mut token = cancel_token.0.lock().await;
        *token = None;
    }

    // ================= Windows 下就地覆盖安装 =================
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("msiexec");
        cmd.arg("/i")
            .arg(&temp_file_path)
            .arg("/passive")
            .arg("/norestart");

        match cmd.spawn() {
            Ok(_) => {
                app.exit(0);
            }
            Err(e) => {
                return Err(format!("启动更新程序失败: {}", e));
            }
        }
    }

    // ================= Linux 下热替换 / deb 安装 =================
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(appimage_path_str) = std::env::var("APPIMAGE") {
            let appimage_path = std::path::PathBuf::from(appimage_path_str);
            let backup_path = appimage_path.with_extension("old");

            let _ = std::fs::rename(&appimage_path, &backup_path);
            if let Err(e) = std::fs::copy(&temp_file_path, &appimage_path) {
                let _ = std::fs::rename(&backup_path, &appimage_path);
                return Err(format!("覆盖 AppImage 文件失败: {}", e));
            }

            // 拷贝完成，立即清除临时文件以防残留
            let _ = std::fs::remove_file(&temp_file_path);

            if let Ok(metadata) = std::fs::metadata(&appimage_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&appimage_path, perms);
            }

            match std::process::Command::new(&appimage_path).spawn() {
                Ok(_) => {
                    app.exit(0);
                }
                Err(e) => {
                    let _ = std::fs::rename(&backup_path, &appimage_path);
                    return Err(format!("启动新版 AppImage 失败: {}", e));
                }
            }
        } else if extension == "deb" {
            let mut cmd = std::process::Command::new("pkexec");
            cmd.arg("dpkg").arg("-i").arg(&temp_file_path);

            match cmd.spawn() {
                Ok(_) => {
                    app.exit(0);
                }
                Err(e) => {
                    return Err(format!(
                        "启动提权覆盖升级失败: {}。您也可以手动运行: sudo dpkg -i {:?}",
                        e, temp_file_path
                    ));
                }
            }
        } else {
            return Err(
                "未检测到 AppImage 环境变量，且当前包非 deb 格式。请手动替换更新。".to_string(),
            );
        }
    }

    // ================= macOS 自动挂载 =================
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&temp_file_path)
            .spawn();
        app.exit(0);
    }

    Ok(())
}

/// 解决 7：新增的主动取消下载指令
#[tauri::command]
pub async fn cancel_update(cancel_token: State<'_, UpdateCancelToken>) -> Result<(), String> {
    let mut token = cancel_token.0.lock().await;
    if let Some(tx) = token.take() {
        let _ = tx.send(()); // 发送取消信号，高频循环体会立即捕获并优雅中止下载
    }
    Ok(())
}
