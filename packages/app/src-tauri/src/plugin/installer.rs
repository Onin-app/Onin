//! # 插件安装管理模块
//!
//! 负责插件的导入、下载和卸载，包括：
//! - 本地插件导入（符号链接）
//! - 从市场下载安装
//! - 插件卸载和清理

use std::path::Path;
use tauri::{Emitter, Manager, State};

use super::state::{
    get_plugin_settings_path, get_system_protected_paths, load_plugin_states, save_plugin_states,
};
use super::types::{
    find_plugin_by_id, make_plugin_dir_name, InstallSource, LoadedPlugin, PluginManifest,
    PluginState, PluginStore,
};
use crate::js_runtime;

// ============================================================================
// Tauri 命令 - 本地导入
// ============================================================================

/// 导入本地插件
///
/// 通过符号链接方式导入插件，支持开发时实时更新
///
/// # 参数
/// - `source_path`: 插件源目录路径
#[tauri::command]
pub fn import_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    source_path: String,
) -> Result<LoadedPlugin, String> {
    println!("[plugin/installer] 从 {} 导入插件", source_path);

    // 1. 验证源路径
    let source = std::path::PathBuf::from(&source_path);
    if !source.exists() || !source.is_dir() {
        return Err("无效的插件目录".to_string());
    }

    // 安全检查：规范化路径，防止路径遍历攻击
    let source = source
        .canonicalize()
        .map_err(|e| format!("解析插件路径失败: {}", e))?;

    // 安全检查：禁止导入系统敏感目录
    let forbidden_paths = get_system_protected_paths();
    for forbidden in &forbidden_paths {
        if source.starts_with(forbidden) {
            return Err(format!("无法从系统目录导入插件: {:?}", forbidden));
        }
    }

    // 2. 验证 manifest.json 是否存在
    let manifest_path = source.join("manifest.json");
    if !manifest_path.exists() {
        return Err("插件目录中未找到 manifest.json".to_string());
    }

    // 3. 读取并解析 manifest
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest 失败: {}", e))?;
    let manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("manifest 格式无效: {}", e))?;

    println!(
        "[plugin/installer] 插件 manifest 已加载: {} ({})",
        manifest.name, manifest.id
    );

    // 安全检查：验证插件 ID 格式
    if manifest.id.contains("..") || manifest.id.contains("/") || manifest.id.contains("\\") {
        return Err("插件 ID 无效: 包含非法字符".to_string());
    }

    // 检查插件是否已存在
    let (plugin_exists, existing_plugin_name) = {
        let store_lock = store.0.lock().unwrap();
        if let Some(existing) = find_plugin_by_id(&store_lock, &manifest.id) {
            (true, Some(existing.manifest.name.clone()))
        } else {
            (false, None)
        }
    };

    if plugin_exists {
        let plugin_name = existing_plugin_name.unwrap_or_else(|| manifest.id.clone());
        return Err(format!(
            "插件 '{}' (ID: {}) 已存在。\n请先卸载现有插件，然后再导入新版本。",
            plugin_name, manifest.id
        ));
    }

    // 4. 获取插件目录
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");

    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("创建 plugins 目录失败: {}", e))?;
    }

    // 5. 创建符号链接
    let dir_name = make_plugin_dir_name(&manifest.id, InstallSource::Local);
    let plugin_link_path = plugins_dir.join(&dir_name);

    // 如果已存在，先删除
    if plugin_link_path.exists() {
        remove_plugin_link(&plugin_link_path)?;
    }

    // 创建符号链接
    create_symlink(&source, &plugin_link_path)?;

    println!(
        "[plugin/installer] 已创建符号链接: {:?} -> {:?}",
        plugin_link_path, source
    );

    // 6. 加载插件到 store
    let plugin_states = load_plugin_states(&app);
    let (enabled, auto_detach) = if let Some(state) = plugin_states.get(&manifest.id) {
        (state.enabled, state.auto_detach)
    } else {
        (true, manifest.auto_detach)
    };

    let mut manifest_with_state = manifest.clone();
    manifest_with_state.auto_detach = auto_detach;

    let loaded_plugin = LoadedPlugin {
        manifest: manifest_with_state,
        dir_name: dir_name.clone(),
        enabled,
        settings: None,
        install_source: InstallSource::Local,
    };

    // 7. 添加到 store
    {
        let mut store_lock = store.0.lock().unwrap();
        store_lock.insert(dir_name.clone(), loaded_plugin.clone());
    }

    // 8. 初始化插件生命周期
    initialize_plugin_lifecycle(&app, &source, &manifest);

    println!(
        "[plugin/installer] 成功导入插件: {}",
        manifest.name
    );
    Ok(loaded_plugin)
}

// ============================================================================
// Tauri 命令 - 卸载
// ============================================================================

/// 卸载插件
///
/// 从系统中完全移除插件，包括文件、状态和运行时
#[tauri::command]
pub fn uninstall_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    println!("[plugin/installer] 卸载插件: {}", plugin_id);

    // 1. 从 store 中获取插件信息
    let (dir_name, actual_key) = {
        let store_lock = store.0.lock().unwrap();
        if let Some(plugin) = find_plugin_by_id(&store_lock, &plugin_id) {
            let key = store_lock
                .iter()
                .find(|(_, p)| p.manifest.id == plugin_id)
                .map(|(k, _)| k.clone());
            (Some(plugin.dir_name.clone()), key)
        } else {
            (None, None)
        }
    };

    // 2. 从 store 中移除
    let plugin_was_in_store = {
        let mut store_lock = store.0.lock().unwrap();
        if let Some(key) = actual_key {
            store_lock.remove(&key).is_some()
        } else {
            false
        }
    };

    // 3. 删除插件目录
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");
    let dir_to_remove = dir_name.clone().unwrap_or_else(|| plugin_id.clone());
    let plugin_link_path = plugins_dir.join(&dir_to_remove);

    println!(
        "[plugin/installer] 尝试删除插件目录: {:?}",
        plugin_link_path
    );

    if plugin_link_path.exists() {
        remove_plugin_directory(&plugin_link_path)?;
    } else if dir_name.is_none() {
        // 扫描目录查找
        scan_and_remove_plugin(&plugins_dir, &plugin_id)?;
    }

    // 4. 清理插件状态
    let plugin_states = load_plugin_states(&app);
    let mut new_states = plugin_states.clone();
    new_states.remove(&plugin_id);
    save_plugin_states(&app, &new_states)?;

    // 5. 清理插件设置
    if let Ok(settings_path) = get_plugin_settings_path(&app, &plugin_id) {
        if settings_path.exists() {
            if let Err(e) = std::fs::remove_file(&settings_path) {
                eprintln!("[plugin/installer] 删除设置文件失败: {}", e);
            }
        }
    }

    // 6. 清理 JavaScript 运行时
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("创建运行时失败: {}", e))?;

    rt.block_on(async {
        if let Err(e) = js_runtime::clear_plugin_runtime(&plugin_id).await {
            eprintln!("[plugin/installer] 清理 JS 运行时失败: {}", e);
        }
    });

    if !plugin_was_in_store && !plugin_link_path.exists() {
        return Err(format!("插件未找到: {}", plugin_id));
    }

    println!("[plugin/installer] 成功卸载插件: {}", plugin_id);
    
    // 发送事件通知前端插件已卸载
    let _ = app.emit("plugin-uninstalled", &plugin_id);
    
    Ok(())
}

// ============================================================================
// Tauri 命令 - 市场下载安装
// ============================================================================

/// 下载并安装插件
///
/// 从市场下载 ZIP 包，解压并安装
#[tauri::command]
pub async fn download_and_install_plugin(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    download_url: String,
    _plugin_id: String,
    icon_url: Option<String>,
) -> Result<LoadedPlugin, String> {
    println!("[plugin/installer] 从 {} 下载插件", download_url);

    // 1. 下载 ZIP 文件
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("创建临时目录失败: {}", e))?;
    let zip_path = temp_dir.path().join("plugin.zip");

    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("下载插件失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败，状态码: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    std::fs::write(&zip_path, &bytes)
        .map_err(|e| format!("写入 zip 文件失败: {}", e))?;

    println!("[plugin/installer] 已下载 {} 字节", bytes.len());

    // 2. 解压
    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("创建解压目录失败: {}", e))?;

    extract_zip(&zip_path, &extract_dir)?;

    // 3. 查找插件根目录
    let plugin_root = find_plugin_root(&extract_dir)?;

    // 4. 读取 manifest
    let manifest_path = plugin_root.join("manifest.json");
    if !manifest_path.exists() {
        return Err("解压文件中未找到 manifest.json".to_string());
    }

    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest 失败: {}", e))?;
    let mut manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("manifest 格式无效: {}", e))?;

    // 强制禁用开发模式
    if manifest.dev_mode {
        println!(
            "[plugin/installer] 插件 {} 的 devMode=true，强制设为 false",
            manifest.id
        );
        manifest.dev_mode = false;
        manifest.dev_server = None;
    }

    // 使用市场提供的图标
    if let Some(icon) = icon_url {
        manifest.icon = Some(icon);
    }

    println!(
        "[plugin/installer] 插件 manifest 已加载: {} ({})",
        manifest.name, manifest.id
    );

    // 5. 复制到 plugins 目录
    let dir_name = make_plugin_dir_name(&manifest.id, InstallSource::Marketplace);
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");
    let target_dir = plugins_dir.join(&dir_name);

    if target_dir.exists() {
        return Err(format!(
            "插件市场版本 '{}' 已存在。\n请先卸载现有版本，然后再安装。",
            manifest.name
        ));
    }

    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir)
            .map_err(|e| format!("创建 plugins 目录失败: {}", e))?;
    }

    copy_dir_all(&plugin_root, &target_dir)?;

    // 写入更新后的 manifest
    let target_manifest_path = target_dir.join("manifest.json");
    let updated_manifest_content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("序列化 manifest 失败: {}", e))?;
    std::fs::write(&target_manifest_path, updated_manifest_content)
        .map_err(|e| format!("写入更新后的 manifest 失败: {}", e))?;

    println!(
        "[plugin/installer] 插件文件已复制到: {:?}",
        target_dir
    );

    // 6. 加载到 store
    let plugin_states = load_plugin_states(&app);
    let auto_detach = plugin_states
        .get(&manifest.id)
        .map(|s| s.auto_detach)
        .unwrap_or(manifest.auto_detach);

    let mut manifest_with_state = manifest.clone();
    manifest_with_state.auto_detach = auto_detach;

    let loaded_plugin = LoadedPlugin {
        manifest: manifest_with_state,
        dir_name: dir_name.clone(),
        enabled: true,
        settings: None,
        install_source: InstallSource::Marketplace,
    };

    // 添加到 store，禁用其他版本
    {
        let mut store_lock = store.0.lock().unwrap();
        
        let other_versions: Vec<String> = store_lock
            .iter()
            .filter(|(_, p)| p.manifest.id == manifest.id && p.enabled)
            .map(|(dir_name, _)| dir_name.clone())
            .collect();
        
        for other_dir_name in other_versions {
            if let Some(other_plugin) = store_lock.get_mut(&other_dir_name) {
                other_plugin.enabled = false;
                println!(
                    "[plugin/installer] 安装市场版本时自动禁用 {} 版本",
                    match other_plugin.install_source {
                        InstallSource::Local => "本地",
                        InstallSource::Marketplace => "市场",
                    }
                );
            }
        }
        
        store_lock.insert(dir_name.clone(), loaded_plugin.clone());
    }

    // 7. 初始化生命周期
    initialize_plugin_lifecycle(&app, &target_dir, &manifest);

    println!(
        "[plugin/installer] 成功安装插件: {}",
        manifest.name
    );
    
    let _ = app.emit("plugin-installed", &manifest.id);
    
    Ok(loaded_plugin)
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 移除插件符号链接
fn remove_plugin_link(path: &Path) -> Result<(), String> {
    println!("[plugin/installer] 删除现有插件链接: {:?}", path);
    
    #[cfg(windows)]
    {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|e| format!("获取符号链接元数据失败: {}", e))?;
        if metadata.is_dir() {
            std::fs::remove_dir(path)
                .map_err(|e| format!("删除插件链接失败: {}", e))?;
        } else {
            std::fs::remove_file(path)
                .map_err(|e| format!("删除插件链接失败: {}", e))?;
        }
    }
    #[cfg(not(windows))]
    {
        std::fs::remove_file(path)
            .map_err(|e| format!("删除插件链接失败: {}", e))?;
    }
    
    Ok(())
}

/// 创建符号链接
fn create_symlink(source: &Path, target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_dir(source, target) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = if e.raw_os_error() == Some(1314) {
                    "创建符号链接失败: 权限不足。\n\n\
                     请启用开发者模式:\n\
                     1. 打开 设置 > 更新和安全 > 开发者选项\n\
                     2. 启用 '开发者模式'\n\
                     3. 重启应用程序\n\n\
                     或以管理员身份运行应用程序。"
                } else {
                    return Err(format!(
                        "创建符号链接失败: {}。\n\
                         在 Windows 上，您可能需要管理员权限或启用开发者模式。",
                        e
                    ));
                };
                Err(error_msg.to_string())
            }
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("创建符号链接失败: {}", e))
    }
}

/// 移除插件目录
fn remove_plugin_directory(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|e| format!("获取符号链接元数据失败: {}", e))?;

        let is_symlink = metadata.file_type().is_symlink();

        if is_symlink {
            // 符号链接：尝试目录删除，失败则尝试文件删除
            match std::fs::remove_dir(path) {
                Ok(_) => Ok(()),
                Err(_) => std::fs::remove_file(path)
                    .map_err(|e| format!("删除插件链接失败: {}", e)),
            }
        } else if metadata.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("删除插件目录失败: {}", e))
        } else {
            std::fs::remove_file(path)
                .map_err(|e| format!("删除插件文件失败: {}", e))
        }
    }
    #[cfg(not(windows))]
    {
        std::fs::remove_file(path)
            .map_err(|e| format!("删除插件链接失败: {}", e))
    }
}

/// 扫描目录查找并删除插件
fn scan_and_remove_plugin(plugins_dir: &Path, plugin_id: &str) -> Result<(), String> {
    println!(
        "[plugin/installer] 扫描 plugins 目录查找 ID 为 {} 的插件",
        plugin_id
    );

    if let Ok(entries) = std::fs::read_dir(plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                            if manifest.id == plugin_id {
                                println!(
                                    "[plugin/installer] 找到插件目录: {:?}",
                                    path
                                );
                                return remove_plugin_directory(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// 解压 ZIP 文件
fn extract_zip(zip_path: &Path, extract_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("打开 zip 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取 zip 归档失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| format!("创建父目录失败: {}", e))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("解压文件失败: {}", e))?;
        }
    }

    Ok(())
}

/// 查找包含 manifest.json 的插件根目录
pub fn find_plugin_root(extract_dir: &Path) -> Result<std::path::PathBuf, String> {
    if extract_dir.join("manifest.json").exists() {
        return Ok(extract_dir.to_path_buf());
    }

    let entries = std::fs::read_dir(extract_dir)
        .map_err(|e| format!("读取解压目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() && path.join("manifest.json").exists() {
            return Ok(path);
        }
    }

    Err("解压文件中未找到 manifest.json".to_string())
}

/// 递归复制目录
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("创建目录 {:?} 失败: {}", dst, e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("读取目录 {:?} 失败: {}", src, e))?
    {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let ty = entry
            .file_type()
            .map_err(|e| format!("获取文件类型失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件 {:?} 失败: {}", src_path, e))?;
        }
    }

    Ok(())
}

/// 初始化插件生命周期
fn initialize_plugin_lifecycle(app: &tauri::AppHandle, plugin_dir: &Path, manifest: &PluginManifest) {
    let entry_path = plugin_dir.join(&manifest.entry);
    if !entry_path.is_file() {
        return;
    }

    let extension = std::path::Path::new(&manifest.entry)
        .extension()
        .and_then(|s| s.to_str());

    let lifecycle_path = match extension {
        Some("js") => {
            println!(
                "[plugin/installer] 初始化 Headless 插件: {}",
                manifest.id
            );
            Some(entry_path.clone())
        }
        Some("html") => {
            let lifecycle_file = manifest
                .lifecycle
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("lifecycle.js");
            let lc_path = plugin_dir.join(lifecycle_file);

            if lc_path.is_file() {
                println!(
                    "[plugin/installer] 初始化视图插件生命周期: {} ({})",
                    manifest.id, lifecycle_file
                );
                Some(lc_path)
            } else {
                println!(
                    "[plugin/installer] 视图插件 {} 未找到生命周期文件 (查找: {})",
                    manifest.id, lifecycle_file
                );
                None
            }
        }
        _ => None,
    };

    if let Some(lc_path) = lifecycle_path {
        let app_clone = app.clone();
        let plugin_id = manifest.id.clone();
        let plugin_name = manifest.name.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                match std::fs::read_to_string(&lc_path) {
                    Ok(js_code) => {
                        match js_runtime::execute_js(&app_clone, &js_code, Some(&plugin_id)).await {
                            Ok(_) => {
                                let _ = app_clone.emit("plugin-init-success", &plugin_id);
                            }
                            Err(e) => {
                                eprintln!(
                                    "[plugin/installer] 初始化插件 {} 失败: {}",
                                    plugin_id, e
                                );
                                #[derive(serde::Serialize, Clone)]
                                struct PluginInitError {
                                    plugin_id: String,
                                    plugin_name: String,
                                    error: String,
                                }
                                let _ = app_clone.emit(
                                    "plugin-init-error",
                                    PluginInitError {
                                        plugin_id,
                                        plugin_name,
                                        error: e.to_string(),
                                    },
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[plugin/installer] 读取生命周期文件失败: {}", e);
                    }
                }
            });
        });
    }
}
