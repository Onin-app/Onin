use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// 检查路径是否匹配黑名单
fn is_blacklisted(relative_path: &str) -> bool {
    let normalized = relative_path.replace('\\', "/");
    let blacklist = vec![
        "plugin_data/window_states.json",
        "extensions/clipboard/",
        "logs/",
    ];

    for item in blacklist {
        if normalized.starts_with(item) || normalized == item {
            return true;
        }
    }
    false
}

/// 打包应用数据目录到 ZIP
pub fn pack_app_data_to_zip(app_data_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| format!("无法创建 ZIP 文件: {}", e))?;
    let mut zip = ZipWriter::new(file);

    // ZIP 文件选项，配置为标准存储或普通压缩
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walkdir = WalkDir::new(app_data_dir);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .strip_prefix(app_data_dir)
            .map_err(|e| format!("路径前缀剥离失败: {}", e))?;

        let name_str = name.to_string_lossy();
        if name_str.is_empty() {
            continue;
        }

        // 路径黑名单校验
        if is_blacklisted(&name_str) {
            println!("[sync/zip] 匹配黑名单，跳过打包: {}", name_str);
            continue;
        }

        if path.is_file() {
            println!("[sync/zip] 打包文件: {}", name_str);
            zip.start_file(name_str.replace('\\', "/"), options)
                .map_err(|e| format!("ZIP 开始文件失败: {}", e))?;

            let mut f = File::open(path).map_err(|e| format!("无法打开源文件: {}", e))?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)
                .map_err(|e| format!("读取源文件失败: {}", e))?;

            zip.write_all(&buffer)
                .map_err(|e| format!("写入 ZIP 包失败: {}", e))?;
        } else if path.is_dir() {
            // 写入目录条目
            zip.add_directory(name_str.replace('\\', "/"), options)
                .map_err(|e| format!("ZIP 添加目录失败: {}", e))?;
        }
    }

    zip.finish().map_err(|e| format!("完成 ZIP 失败: {}", e))?;
    Ok(())
}

/// 从 ZIP 解压并覆盖本地应用数据目录
pub fn unpack_zip_to_app_data(zip_path: &Path, app_data_dir: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("无法打开 ZIP 文件: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("读取 ZIP 包失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("无法获取 ZIP 索引文件: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath_str = outpath.to_string_lossy();

        // 路径黑名单校验
        if is_blacklisted(&outpath_str) {
            println!("[sync/zip] 匹配黑名单，跳过恢复: {}", outpath_str);
            continue;
        }

        let file_path = app_data_dir.join(&outpath);

        if file.name().ends_with('/') {
            // 创建目录
            std::fs::create_dir_all(&file_path).map_err(|e| format!("解压创建目录失败: {}", e))?;
        } else {
            // 创建父目录
            if let Some(p) = file_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| format!("解压创建父目录失败: {}", e))?;
                }
            }

            println!("[sync/zip] 恢复文件: {}", outpath_str);
            let mut outfile =
                File::create(&file_path).map_err(|e| format!("无法在本地创建文件: {}", e))?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("解压文件失败: {}", e))?;

            outfile
                .write_all(&buffer)
                .map_err(|e| format!("写入解压文件失败: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_blacklisted() {
        assert!(is_blacklisted("plugin_data/window_states.json"));
        assert!(is_blacklisted("plugin_data\\window_states.json"));
        assert!(is_blacklisted("logs/app.log"));
        assert!(is_blacklisted("extensions/clipboard/cache.bin"));

        assert!(!is_blacklisted("app_config.json"));
        assert!(!is_blacklisted("plugins/hello/main.js"));
    }

    #[test]
    fn test_zip_pack_and_unpack_with_blacklist() {
        let src_dir = tempdir().unwrap();
        let src_path = src_dir.path();

        // 1. 创建普通文件
        let normal_file = src_path.join("app_config.json");
        fs::write(&normal_file, "{}").unwrap();

        // 创建子目录中的普通文件
        let plugin_dir = src_path.join("plugins").join("my_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        let plugin_file = plugin_dir.join("config.json");
        fs::write(&plugin_file, "plugin-data").unwrap();

        // 2. 创建属于黑名单的文件/目录
        let window_states_dir = src_path.join("plugin_data");
        fs::create_dir_all(&window_states_dir).unwrap();
        fs::write(window_states_dir.join("window_states.json"), "states").unwrap();

        let log_dir = src_path.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        fs::write(log_dir.join("onin.log"), "logs").unwrap();

        let clipboard_dir = src_path.join("extensions").join("clipboard");
        fs::create_dir_all(&clipboard_dir).unwrap();
        fs::write(clipboard_dir.join("history.json"), "clipboard").unwrap();

        // 3. 打包
        let dest_dir = tempdir().unwrap();
        let zip_path = dest_dir.path().join("backup.zip");
        pack_app_data_to_zip(src_path, &zip_path).unwrap();

        // 4. 解包到新的临时目录
        let unpack_dir = tempdir().unwrap();
        let unpack_path = unpack_dir.path();
        unpack_zip_to_app_data(&zip_path, unpack_path).unwrap();

        // 5. 校验结果
        // 普通文件应该存在且内容正确
        assert!(unpack_path.join("app_config.json").exists());
        assert_eq!(
            fs::read_to_string(unpack_path.join("app_config.json")).unwrap(),
            "{}"
        );

        assert!(unpack_path.join("plugins/my_plugin/config.json").exists());
        assert_eq!(
            fs::read_to_string(unpack_path.join("plugins/my_plugin/config.json")).unwrap(),
            "plugin-data"
        );

        // 黑名单中的文件和目录应该被排除，不应存在
        assert!(!unpack_path.join("plugin_data/window_states.json").exists());
        assert!(!unpack_path.join("logs/onin.log").exists());
        assert!(!unpack_path
            .join("extensions/clipboard/history.json")
            .exists());
    }
}
