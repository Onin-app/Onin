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
