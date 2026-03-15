//! 插件 HTTP 服务器
//!
//! 为插件提供本地 HTTP 服务，用于加载插件资源文件。

use axum::{
    extract::{Path, Query, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ============================================================================
// 服务器状态和配置
// ============================================================================

/// Plugin HTTP server state
pub struct PluginServerState {
    pub plugins_dir: std::path::PathBuf,
}

/// 启动端口范围
const START_PORT: u16 = 3456;
const MAX_PORT_ATTEMPTS: u16 = 10;

// ============================================================================
// 服务器启动
// ============================================================================

/// Start the plugin HTTP server
pub async fn start_plugin_server(
    plugins_dir: std::path::PathBuf,
) -> Result<u16, Box<dyn std::error::Error>> {
    for port in START_PORT..(START_PORT + MAX_PORT_ATTEMPTS) {
        if try_start_server(plugins_dir.clone(), port).await.is_ok() {
            return Ok(port);
        }
    }
    Err("Failed to start plugin server: no available ports".into())
}

async fn try_start_server(
    plugins_dir: std::path::PathBuf,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(PluginServerState { plugins_dir });

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:1420"
                .parse::<header::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET])
        .allow_headers(vec![header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/plugin/:plugin_id/*path", get(serve_plugin_file))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[plugin_server] Server error: {}", e);
        }
    });

    Ok(())
}

// ============================================================================
// 文件服务
// ============================================================================

/// Serve plugin files
async fn serve_plugin_file(
    State(state): State<Arc<PluginServerState>>,
    Path((plugin_id, file_path)): Path<(String, String)>,
    Query(_params): Query<HashMap<String, String>>,
) -> Response {
    let file_path = if file_path.is_empty() {
        "index.html".to_string()
    } else {
        file_path
    };

    let full_path = state.plugins_dir.join(&plugin_id).join(&file_path);

    // 检查文件是否存在
    if !full_path.exists() {
        return handle_file_not_found(&file_path);
    }

    // 读取文件内容
    let content = match tokio::fs::read(&full_path).await {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read file: {}", e),
            )
                .into_response();
        }
    };

    // 确定 content type
    let extension = full_path.extension().and_then(|s| s.to_str());
    let content_type = get_content_type(extension);

    // 处理 HTML 文件
    let final_content = if extension == Some("html") {
        process_html_content(content, &plugin_id)
    } else {
        content
    };

    ([(header::CONTENT_TYPE, content_type)], final_content).into_response()
}

/// 处理文件不存在的情况
fn handle_file_not_found(file_path: &str) -> Response {
    // 检测是否是开发模式文件
    if file_path.starts_with("src/") || file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        return (
            StatusCode::NOT_FOUND,
            format!(
                "Plugin not built: The file '{}' suggests this plugin is in development mode. \
                Please run 'npm run build' or 'pnpm build' in the plugin directory.",
                file_path
            ),
        )
            .into_response();
    }

    (
        StatusCode::NOT_FOUND,
        format!("File not found: {}", file_path),
    )
        .into_response()
}

/// 根据文件扩展名获取 MIME 类型
fn get_content_type(extension: Option<&str>) -> &'static str {
    match extension {
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

// ============================================================================
// HTML 处理
// ============================================================================

/// 处理 HTML 内容：修复路径并注入 Tauri 桥接
fn process_html_content(content: Vec<u8>, plugin_id: &str) -> Vec<u8> {
    if let Ok(html) = String::from_utf8(content.clone()) {
        let fixed_html = fix_asset_paths(&html);
        inject_tauri_bridge(&fixed_html, plugin_id).into_bytes()
    } else {
        content
    }
}

/// 修复 HTML 中的绝对路径为相对路径
///
/// Vite 构建的插件使用绝对路径（如 /assets/...），需要转换为相对路径
fn fix_asset_paths(html: &str) -> String {
    html.replace("=\"/", "=\"./").replace("='/", "='./")
}

// ============================================================================
// Tauri 桥接注入
// ============================================================================

/// Runtime initialization script template
const TAURI_BRIDGE_SCRIPT: &str = r#"
<script>
(function() {
  const pluginIdFromInjection = '__PLUGIN_ID__';
  const urlParams = new URLSearchParams(window.location.search);
  const pluginIdFromUrl = urlParams.get('plugin_id');

  window.__PLUGIN_ID__ = pluginIdFromUrl || pluginIdFromInjection;
  globalThis.__PLUGIN_ID__ = window.__PLUGIN_ID__;
})();
</script>
"#;

/// 注入运行时初始化脚本到 HTML
fn inject_tauri_bridge(html: &str, plugin_id: &str) -> String {
    let bridge_script = TAURI_BRIDGE_SCRIPT.replace("__PLUGIN_ID__", plugin_id);

    if html.contains("<head>") {
        html.replace("<head>", &format!("<head>{}", bridge_script))
    } else if html.contains("<html>") {
        html.replace("<html>", &format!("<html><head>{}</head>", bridge_script))
    } else {
        format!("{}{}", bridge_script, html)
    }
}
