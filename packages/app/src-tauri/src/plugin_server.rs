use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Plugin HTTP server state
pub struct PluginServerState {
    pub plugins_dir: std::path::PathBuf,
    pub port: u16,
}

/// Start the plugin HTTP server
pub async fn start_plugin_server(
    plugins_dir: std::path::PathBuf,
) -> Result<u16, Box<dyn std::error::Error>> {
    // Try to bind to a port starting from 3456
    let mut port = 3456;
    let max_attempts = 10;

    for _ in 0..max_attempts {
        match try_start_server(plugins_dir.clone(), port).await {
            Ok(_) => {
                println!("[plugin_server] Started on port {}", port);
                return Ok(port);
            }
            Err(_) => {
                port += 1;
            }
        }
    }

    Err("Failed to start plugin server: no available ports".into())
}

async fn try_start_server(
    plugins_dir: std::path::PathBuf,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(PluginServerState { plugins_dir, port });

    // 只允许本地访问
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

    println!("[plugin_server] Listening on http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[plugin_server] Server error: {}", e);
        }
    });

    Ok(())
}

use axum::extract::Query;
use std::collections::HashMap;

/// Serve plugin files
async fn serve_plugin_file(
    State(state): State<Arc<PluginServerState>>,
    Path((plugin_id, file_path)): Path<(String, String)>,
    Query(_params): Query<HashMap<String, String>>,
) -> Response {
    // Construct the file path
    let file_path = if file_path.is_empty() {
        "index.html".to_string()
    } else {
        file_path
    };

    let full_path = state.plugins_dir.join(&plugin_id).join(&file_path);

    // Check if file exists
    if !full_path.exists() {
        // Check if this looks like a development mode file
        if file_path.starts_with("src/")
            || file_path.ends_with(".ts")
            || file_path.ends_with(".tsx")
        {
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

        return (
            StatusCode::NOT_FOUND,
            format!("File not found: {}", file_path),
        )
            .into_response();
    }

    // Read file content
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

    // Determine content type
    let content_type = match full_path.extension().and_then(|s| s.to_str()) {
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
    };

    // 对于 HTML 文件，始终注入 Tauri 桥接脚本和插件 ID
    // 同时修复 Vite 构建插件的绝对路径为相对路径
    let final_content = if full_path.extension().and_then(|s| s.to_str()) == Some("html") {
        if let Ok(html) = String::from_utf8(content.clone()) {
            // 首先，修复绝对路径为相对路径
            // Vite 构建通常使用 /assets/... 这样的绝对路径，在我们的服务器上无法正常工作
            let fixed_html = fix_asset_paths(&html);
            // 然后注入 Tauri 桥接脚本
            inject_tauri_bridge(&fixed_html, &plugin_id).into_bytes()
        } else {
            content
        }
    } else {
        content
    };

    ([(header::CONTENT_TYPE, content_type)], final_content).into_response()
}

/// 修复 HTML 中的绝对路径为相对路径
/// 
/// Vite 构建的插件使用绝对路径（如 /assets/...），但在我们的插件服务器中：
/// - 插件 HTML 的 URL 是：http://127.0.0.1:3457/plugin/plugin-id/dist/index.html
/// - 如果 HTML 中引用 /assets/style.css，浏览器会解析为 http://127.0.0.1:3457/assets/style.css（错误）
/// - 实际文件路径应该是：http://127.0.0.1:3457/plugin/plugin-id/dist/assets/style.css
/// 
/// 因此需要将绝对路径 / 转换为相对路径 ./，让浏览器相对于 HTML 文件所在目录解析资源
fn fix_asset_paths(html: &str) -> String {
    html.replace("=\"/", "=\"./")
        .replace("='/", "='./")
}

/// Inject Tauri API bridge for inline plugins
fn inject_tauri_bridge(html: &str, plugin_id: &str) -> String {
    let bridge_script = format!(
        r#"
<script>
(function() {{
  // Get plugin ID from URL parameters or injected value
  const urlParams = new URLSearchParams(window.location.search);
  const pluginIdFromUrl = urlParams.get('plugin_id');
  const pluginIdFromInjection = '{}';
  
  window.__PLUGIN_ID__ = pluginIdFromUrl || pluginIdFromInjection;
  globalThis.__PLUGIN_ID__ = window.__PLUGIN_ID__;
  
  const createProxy = (command) => {{
    return (...args) => {{
      return new Promise((resolve, reject) => {{
        const messageId = 'tauri_' + Math.random().toString(36).substring(7) + '_' + Date.now();
        
        const handleResponse = (event) => {{
          if (event.data && event.data.messageId === messageId) {{
            window.removeEventListener('message', handleResponse);
            if (event.data.error) {{
              reject(new Error(event.data.error));
            }} else {{
              resolve(event.data.result);
            }}
          }}
        }};
        
        window.addEventListener('message', handleResponse);
        
        window.parent.postMessage({{
          type: 'plugin-tauri-call',
          messageId,
          command,
          args
        }}, '*');
        
        setTimeout(() => {{
          window.removeEventListener('message', handleResponse);
          reject(new Error('Tauri call timeout'));
        }}, 30000);
      }});
    }};
  }};
  
  const invokeProxy = createProxy('invoke');
  
  window.__TAURI__ = {{
    core: {{ invoke: invokeProxy }},
    event: {{
      emit: createProxy('emit'),
      listen: createProxy('listen')
    }},
    invoke: invokeProxy
  }};
  
  window.__TAURI_INVOKE__ = invokeProxy;
}})();
</script>
"#,
        plugin_id
    );

    if html.contains("<head>") {
        html.replace("<head>", &format!("<head>{}", bridge_script))
    } else if html.contains("<html>") {
        html.replace("<html>", &format!("<html><head>{}</head>", bridge_script))
    } else {
        format!("{}{}", bridge_script, html)
    }
}
