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

    // 解决 4：下载完成，显式释放文件句柄（fd），确保在后续平台相关命令（如 Windows 的 msiexec，macOS 的 hdiutil/cp）执行时文件不被占用
    drop(file);

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

    // ================= macOS 自动挂载与静默覆盖升级 =================
    #[cfg(target_os = "macos")]
    {
        let mut bundle_replaced = false;

        if let Ok(current_exe) = std::env::current_exe() {
            // 回溯寻找当前运行的 .app bundle 目录（例如 /Applications/Onin.app）
            let mut current_path = current_exe.clone();
            let mut bundle_path = None;
            while let Some(parent) = current_path.parent() {
                if current_path.extension().map_or(false, |ext| ext == "app") {
                    bundle_path = Some(current_path.to_path_buf());
                    break;
                }
                current_path = parent.to_path_buf();
            }

            if let Some(bundle_path) = bundle_path {
                let current_pid = std::process::id();
                let mount_point = std::env::temp_dir().join(format!("onin_mount_{}", current_pid));

                // 1. 尝试清理之前可能残留的目录及挂载
                let _ = std::process::Command::new("hdiutil")
                    .arg("detach")
                    .arg(&mount_point)
                    .arg("-force")
                    .status();
                let _ = std::fs::remove_dir_all(&mount_point);
                let _ = std::fs::create_dir_all(&mount_point);

                // 2. 后台静默挂载 DMG 镜像（使用 -nobrowse 避免 Finder 激活弹窗）
                let mount_status = std::process::Command::new("hdiutil")
                    .arg("attach")
                    .arg("-nobrowse")
                    .arg("-readonly")
                    .arg("-mountpoint")
                    .arg(&mount_point)
                    .arg(&temp_file_path)
                    .status();

                if mount_status.as_ref().map_or(false, |s| s.success()) {
                    let mut new_app_path = None;
                    if let Ok(entries) = std::fs::read_dir(&mount_point) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.extension().map_or(false, |ext| ext == "app") {
                                new_app_path = Some(path);
                                break;
                            }
                        }
                    }

                    if let Some(new_app_path) = new_app_path {
                        let old_bundle_path = bundle_path.with_extension("old.app");
                        if old_bundle_path.exists() {
                            let _ = std::fs::remove_dir_all(&old_bundle_path);
                        }

                        // 3. 将原运行的 .app 目录重命名为 .old.app（在 inode 层直接释放原目录名，完全不影响正在运行的进程）
                        if std::fs::rename(&bundle_path, &old_bundle_path).is_ok() {
                            // 4. 复制新版 .app 目录至原位（cp -R 能原生保留 macOS 特有的软链接与文件元数据）
                            let copy_status = std::process::Command::new("cp")
                                .arg("-R")
                                .arg(&new_app_path)
                                .arg(&bundle_path)
                                .status();

                            if copy_status.as_ref().map_or(false, |s| s.success()) {
                                // 5. 卸载 DMG
                                let _ = std::process::Command::new("hdiutil")
                                    .arg("detach")
                                    .arg(&mount_point)
                                    .status();

                                // 6. 异步拉起全新升级后的应用
                                let _ =
                                    std::process::Command::new("open").arg(&bundle_path).spawn();

                                // 7. 开启独立的 shell 后台清理进程，完全防止 Shell 注入漏洞。
                                // 通过 kill -0 循环轮询旧进程 PID 直至其完全退出（最多轮询 100 次，每次 100ms 也就是最多等待 10s），然后彻底删除旧版本残留及临时挂载点。
                                let _ = std::process::Command::new("nohup")
                                    .arg("sh")
                                    .arg("-c")
                                    .arg("pid=$1; old_path=$2; mount_p=$3; i=0; while [ $i -lt 100 ] && kill -0 \"$pid\" 2>/dev/null; do sleep 0.1; i=$((i+1)); done; rm -rf \"$old_path\"; for r in 1 2 3; do hdiutil detach \"$mount_p\" -force 2>/dev/null && break; sleep 1; done; rm -rf \"$mount_p\"")
                                    .arg("--")
                                    .arg(current_pid.to_string())
                                    .arg(&old_bundle_path)
                                    .arg(&mount_point)
                                    .spawn();

                                bundle_replaced = true;
                                app.exit(0);
                            } else {
                                // 容灾双重回滚：若复制新版失败，先将可能损坏的新包 rename 移开，再原子性还原旧版，最后异步清除损坏的残留
                                let failed_copy_path = bundle_path.with_extension("failed.app");
                                if std::fs::rename(&bundle_path, &failed_copy_path).is_ok() {
                                    if std::fs::rename(&old_bundle_path, &bundle_path).is_err() {
                                        // 极其罕见灾难：恢复旧版本失败。尝试把刚刚移走的新包移回来保底，尽量避免 app 不可用
                                        let _ = std::fs::rename(&failed_copy_path, &bundle_path);
                                    } else {
                                        // 成功原子恢复了旧包！异步清除损坏的新包
                                        let _ = std::process::Command::new("nohup")
                                            .arg("sh")
                                            .arg("-c")
                                            .arg("rm -rf \"$1\"")
                                            .arg("--")
                                            .arg(&failed_copy_path)
                                            .spawn();
                                    }
                                } else {
                                    // 降级回滚方案
                                    let _ = std::fs::remove_dir_all(&bundle_path);
                                    let _ = std::fs::rename(&old_bundle_path, &bundle_path);
                                }

                                // 成功完成双重原子回滚，原应用包已完全复原，直接返回具体错误给前端且不走 fallback

                                let _ = std::process::Command::new("hdiutil")
                                    .arg("detach")
                                    .arg(&mount_point)
                                    .status();
                                let _ = std::fs::remove_dir_all(&mount_point);

                                return Err("复制新版本文件失败，已自动原子级回滚恢复当前版本。"
                                    .to_string());
                            }
                        }
                    }

                    // 如果中途出错且没有成功替换，安全卸载挂载点
                    if !bundle_replaced {
                        let _ = std::process::Command::new("hdiutil")
                            .arg("detach")
                            .arg(&mount_point)
                            .status();
                        let _ = std::fs::remove_dir_all(&mount_point);
                    }
                }
            }
        }

        // 备用降级方案：若静默覆盖失败（如无权限/非标准 bundle/开发环境），使用 open 直接打开 DMG 并退出引导用户手动拖拽
        if !bundle_replaced {
            let _ = std::process::Command::new("open")
                .arg(&temp_file_path)
                .spawn();
            app.exit(0);
        }
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
