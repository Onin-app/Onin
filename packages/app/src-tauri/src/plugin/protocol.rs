//! # 插件协议处理模块
//!
//! 处理 `plugin://` 自定义协议请求，包括：
//! - 解析请求路径
//! - 读取插件资源文件
//! - 注入运行时脚本

use tauri::http::{Request, Response};
use tauri::Manager;

use super::bridge::{PLUGIN_WINDOW_CONTROLS_SCRIPT, PLUGIN_WINDOW_TOPBAR_TEMPLATE};
use super::types::PluginStore;

// ============================================================================
// 协议处理
// ============================================================================

/// 处理 plugin:// 协议请求
///
/// 用于加载插件资源文件（HTML、JS、CSS、图片等）
pub fn handle_plugin_protocol<R: tauri::Runtime>(
    context: tauri::UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<std::borrow::Cow<'static, [u8]>> {
    let uri = request.uri();
    let path = uri.path();

    println!("[plugin/protocol] 请求 URI: {}", uri);
    println!("[plugin/protocol] 请求路径: {}", path);

    // 解析路径，格式为 /plugin_dir_name/file_path
    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    println!("[plugin/protocol] 路径部分: {:?}", path_parts);

    if path_parts.is_empty() || path_parts[0].is_empty() {
        println!("[plugin/protocol] 空路径");
        return Response::builder()
            .status(404)
            .body("Not Found".as_bytes().to_vec().into())
            .unwrap();
    }

    let plugin_dir_name = path_parts[0];
    let file_path = if path_parts.len() > 1 {
        path_parts[1..].join("/")
    } else {
        "index.html".to_string()
    };

    // 获取插件目录
    let data_dir = match context.app_handle().path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("[plugin/protocol] 获取应用数据目录失败: {}", e);
            return Response::builder()
                .status(500)
                .body("Internal Server Error".as_bytes().to_vec().into())
                .unwrap();
        }
    };

    let plugin_file_path = data_dir
        .join("plugins")
        .join(plugin_dir_name)
        .join(&file_path);

    println!("[plugin/protocol] 请求文件: {:?}", plugin_file_path);

    // 检查文件是否存在
    if !plugin_file_path.exists() {
        println!("[plugin/protocol] 文件不存在: {:?}", plugin_file_path);

        // 调试：列出插件目录内容
        let plugin_dir = data_dir.join("plugins").join(plugin_dir_name);
        if plugin_dir.exists() {
            println!("[plugin/protocol] 插件目录存在: {:?}", plugin_dir);
            if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
                println!("[plugin/protocol] 目录内容:");
                for entry in entries.flatten() {
                    println!("  - {:?}", entry.file_name());
                }
            }
        } else {
            println!("[plugin/protocol] 插件目录不存在: {:?}", plugin_dir);
        }

        return Response::builder()
            .status(404)
            .body(
                format!("File Not Found: {}", file_path)
                    .as_bytes()
                    .to_vec()
                    .into(),
            )
            .unwrap();
    }

    // 读取文件内容
    let mut content = match std::fs::read(&plugin_file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("[plugin/protocol] 读取文件失败: {}", e);
            return Response::builder()
                .status(500)
                .body("Failed to read file".as_bytes().to_vec().into())
                .unwrap();
        }
    };

    // 如果是 HTML 文件，需要修改资源路径并注入内容
    if plugin_file_path.extension().and_then(|s| s.to_str()) == Some("html") {
        if let Ok(html_content) = String::from_utf8(content.clone()) {
            // 将绝对路径转换为相对路径
            let mut modified_html = html_content
                .replace("src=\"/assets/", "src=\"./assets/")
                .replace("href=\"/assets/", "href=\"./assets/")
                .replace("src='/assets/", "src='./assets/")
                .replace("href='/assets/", "href='./assets/")
                .replace("href=\"/vite.svg\"", "href=\"./vite.svg\"");

            // 获取插件信息以注入 plugin ID
            let store = context.app_handle().state::<PluginStore>();
            let store_lock = store.0.lock().unwrap();
            let plugin_opt = store_lock.values().find(|p| p.dir_name == plugin_dir_name);

            if let Some(plugin) = plugin_opt {
                // 注入 plugin ID 和运行时信息（窗口模式）
                let plugin_id_script = format!(
                    r#"<script>
window.__PLUGIN_ID__ = '{}';
window.__ONIN_RUNTIME__ = {{
  mode: 'window',
  pluginId: '{}',
  version: '{}',
  mainWindowLabel: 'main'
}};
</script>"#,
                    plugin.manifest.id, plugin.manifest.id, plugin.manifest.version
                );

                let topbar_html = format!(
                    "{}{}\n<script>\n{}\n</script>",
                    plugin_id_script, PLUGIN_WINDOW_TOPBAR_TEMPLATE, PLUGIN_WINDOW_CONTROLS_SCRIPT
                );

                // 在 </head> 之前或 <body> 之后注入
                if let Some(head_pos) = modified_html.find("</head>") {
                    modified_html.insert_str(head_pos, &topbar_html);
                } else if let Some(body_pos) = modified_html.find("<body") {
                    if let Some(body_end) = modified_html[body_pos..].find('>') {
                        let insert_pos = body_pos + body_end + 1;
                        modified_html.insert_str(insert_pos, &topbar_html);
                    }
                }
            }

            content = modified_html.into_bytes();
        }
    }

    // 根据文件扩展名设置 Content-Type
    let content_type = get_content_type(&plugin_file_path);

    println!("[plugin/protocol] 响应文件，Content-Type: {}", content_type);

    Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Cache-Control", "no-cache")
        .body(content.into())
        .unwrap()
}

/// 根据文件扩展名获取 Content-Type
fn get_content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}
