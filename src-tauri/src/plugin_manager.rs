use crate::js_runtime;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::http::{Request, Response};
use tauri::{Manager, State, WebviewWindowBuilder};

pub struct PluginStore(pub Mutex<HashMap<String, LoadedPlugin>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandManifest {
    pub code: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<PluginCommandKeyword>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandKeyword {
    pub name: String,
    #[serde(rename = "type")]
    pub keyword_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default, rename = "allowUrls")]
    pub allow_urls: Vec<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default, rename = "maxRetries")]
    pub max_retries: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoragePermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub local: bool,
    #[serde(default)]
    pub session: bool,
    #[serde(default, rename = "maxSize")]
    pub max_size: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub sound: bool,
    #[serde(default)]
    pub badge: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandPermission {
    #[serde(default)]
    pub enable: bool,
    #[serde(default, rename = "allowCommands")]
    pub allow_commands: Vec<String>,
    #[serde(default, rename = "maxExecutionTime")]
    pub max_execution_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginPermissions {
    #[serde(default)]
    pub http: Option<HttpPermission>,
    #[serde(default)]
    pub storage: Option<StoragePermission>,
    #[serde(default)]
    pub notification: Option<NotificationPermission>,
    #[serde(default)]
    pub command: Option<CommandPermission>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry: String,
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
    #[serde(default)]
    pub commands: Vec<PluginCommandManifest>,
    pub permissions: Option<PluginPermissions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadedPlugin {
    #[serde(flatten)]
    pub manifest: PluginManifest,
    pub dir_name: String,
}

fn load_plugins_internal(
    app: &tauri::AppHandle,
    store: &State<PluginStore>,
    clear_existing: bool,
) -> Result<Vec<LoadedPlugin>, String> {
    println!("[plugin_manager] Loading plugins...");
    let data_dir = app.path().app_data_dir().map_err(|e| {
        println!("[plugin_manager] Error getting data dir: {}", e);
        e.to_string()
    })?;

    let plugins_dir = data_dir.join("plugins");
    println!("[plugin_manager] Plugins dir: {:?}", plugins_dir);

    if !plugins_dir.is_dir() {
        println!("[plugin_manager] Plugins dir not found.");
        return Ok(Vec::new());
    }

    let mut store_lock = store.0.lock().unwrap();
    if clear_existing {
        store_lock.clear(); // Clear old plugins
        println!("[plugin_manager] Cleared existing plugins from store.");
    }

    for entry in std::fs::read_dir(plugins_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.is_file() {
                let manifest_content =
                    std::fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
                let manifest: PluginManifest =
                    serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;

                let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();

                let loaded_plugin = LoadedPlugin {
                    manifest: manifest.clone(),
                    dir_name,
                };

                println!(
                    "[plugin_manager] Loaded plugin: {} from {}",
                    manifest.name, loaded_plugin.dir_name
                );
                store_lock.insert(manifest.id.clone(), loaded_plugin);
            }
        }
    }

    let plugins = store_lock.values().cloned().collect();
    println!("[plugin_manager] Loaded {} plugins.", store_lock.len());
    Ok(plugins)
}

#[tauri::command]
pub fn load_plugins(
    app: tauri::AppHandle,
    store: State<PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    load_plugins_internal(&app, &store, true)
}

#[tauri::command]
pub async fn refresh_plugins(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
) -> Result<Vec<LoadedPlugin>, String> {
    println!("[plugin_manager] Refreshing plugins...");
    
    // 清除 JavaScript 运行时缓存
    if let Err(e) = crate::js_runtime::clear_all_plugin_runtimes().await {
        println!("[plugin_manager] Warning: Failed to clear JS runtimes: {}", e);
        // 不要因为这个错误而失败，继续刷新插件
    } else {
        println!("[plugin_manager] Successfully cleared all plugin runtimes");
    }
    
    // 清除插件加载状态缓存
    let loaded_state = app.state::<crate::plugin_api::command::PluginLoadedState>();
    {
        let mut state = loaded_state.0.lock().unwrap();
        let count = state.len();
        state.clear();
        println!("[plugin_manager] Cleared {} plugin loaded states", count);
    }
    
    // 重新加载所有插件
    let result = load_plugins_internal(&app, &store, true);
    
    match &result {
        Ok(plugins) => {
            println!("[plugin_manager] Successfully refreshed {} plugins.", plugins.len());
        }
        Err(e) => {
            println!("[plugin_manager] Failed to refresh plugins: {}", e);
        }
    }
    
    result
}

#[tauri::command]
pub fn execute_plugin_entry(
    app: tauri::AppHandle,
    store: State<'_, PluginStore>,
    plugin_id: String,
) -> Result<(), String> {
    // Clone plugin data to release the lock ASAP
    let plugin = {
        let store_lock = store.0.lock().unwrap();
        store_lock.get(&plugin_id).cloned()
    }
    .ok_or_else(|| "Plugin not found".to_string())?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
    let entry_path = plugin_dir.join(&plugin.manifest.entry);

    if !entry_path.is_file() {
        return Err(format!("Plugin entry file not found: {:?}", entry_path));
    }

    if let Some(extension) = Path::new(&plugin.manifest.entry)
        .extension()
        .and_then(|s| s.to_str())
    {
        match extension {
            "js" => {
                // Headless plugin, execute in the background
                let js_code = std::fs::read_to_string(entry_path).map_err(|e| e.to_string())?;
                let app_clone = app.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        if let Err(e) = js_runtime::execute_js(&app_clone, &js_code).await {
                            eprintln!("Failed to execute headless plugin: {}", e);
                        }
                    });
                });
                Ok(())
            }
            "html" => {
                // UI plugin, create a new webview in a background task
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let window_label = format!("plugin_{}", plugin.manifest.id.replace('.', "_"));

                    if let Some(window) = app_clone.get_webview_window(&window_label) {
                        if let Err(e) = window.set_focus() {
                            eprintln!("Failed to focus plugin window: {}", e);
                        }
                        return;
                    }

                    // 使用自定义协议来加载插件文件
                    let plugin_url = format!(
                        "plugin://localhost/{}/{}",
                        plugin.dir_name, plugin.manifest.entry
                    );
                    println!("[plugin_manager] Loading plugin from: {}", plugin_url);

                    let builder = WebviewWindowBuilder::new(
                        &app_clone,
                        window_label.clone(),
                        tauri::WebviewUrl::External(plugin_url.parse().unwrap()),
                    )
                    .title(plugin.manifest.name)
                    .inner_size(800.0, 600.0)
                    .resizable(true);

                    if let Err(e) = builder.build() {
                        eprintln!("Failed to build plugin window: {}", e);
                    }
                });
                Ok(())
            }
            _ => Err(format!("Unsupported plugin entry type: {}", extension)),
        }
    } else {
        Err("Plugin entry file has no extension".to_string())
    }
}



pub fn handle_plugin_protocol<R: tauri::Runtime>(
    context: tauri::UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<std::borrow::Cow<'static, [u8]>> {
    let uri = request.uri();
    let path = uri.path();

    println!("[plugin_protocol] Request URI: {}", uri);
    println!("[plugin_protocol] Request path: {}", path);

    // 解析路径，格式为 /plugin_dir_name/file_path 或者 /plugin_dir_name/assets/file_path
    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    println!("[plugin_protocol] Path parts: {:?}", path_parts);

    if path_parts.is_empty() || path_parts[0].is_empty() {
        println!("[plugin_protocol] Empty path");
        return Response::builder()
            .status(404)
            .body("Not Found".as_bytes().to_vec().into())
            .unwrap();
    }

    let plugin_dir_name = path_parts[0];
    let file_path = if path_parts.len() > 1 {
        path_parts[1..].join("/")
    } else {
        // 如果只有插件目录名，默认加载 index.html
        "index.html".to_string()
    };

    // 获取插件目录
    let data_dir = match context.app_handle().path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("[plugin_protocol] Failed to get app data dir: {}", e);
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

    println!("[plugin_protocol] Requesting file: {:?}", plugin_file_path);

    // 检查文件是否存在
    if !plugin_file_path.exists() {
        println!("[plugin_protocol] File does not exist: {:?}", plugin_file_path);
        
        // 尝试列出插件目录的内容以便调试
        let plugin_dir = data_dir.join("plugins").join(plugin_dir_name);
        if plugin_dir.exists() {
            println!("[plugin_protocol] Plugin directory exists: {:?}", plugin_dir);
            if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
                println!("[plugin_protocol] Directory contents:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {:?}", entry.file_name());
                    }
                }
            }
        } else {
            println!("[plugin_protocol] Plugin directory does not exist: {:?}", plugin_dir);
        }
        
        return Response::builder()
            .status(404)
            .body(format!("File Not Found: {}", file_path).as_bytes().to_vec().into())
            .unwrap();
    }

    // 读取文件内容
    let mut content = match std::fs::read(&plugin_file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("[plugin_protocol] Failed to read file: {}", e);
            return Response::builder()
                .status(500)
                .body("Failed to read file".as_bytes().to_vec().into())
                .unwrap();
        }
    };

    // 如果是 HTML 文件，需要修改其中的资源路径
    if plugin_file_path.extension().and_then(|s| s.to_str()) == Some("html") {
        if let Ok(html_content) = String::from_utf8(content.clone()) {
            // 将绝对路径转换为相对路径，这样浏览器会通过同一个协议请求资源
            let modified_html = html_content
                .replace("src=\"/assets/", "src=\"./assets/")
                .replace("href=\"/assets/", "href=\"./assets/")
                .replace("src='/assets/", "src='./assets/")
                .replace("href='/assets/", "href='./assets/")
                .replace("href=\"/vite.svg\"", "href=\"./vite.svg\"");
            
            content = modified_html.into_bytes();
            println!("[plugin_protocol] Modified HTML content for plugin: {}", plugin_dir_name);
        }
    }

    // 根据文件扩展名设置Content-Type
    let content_type = match plugin_file_path.extension().and_then(|s| s.to_str()) {
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

    println!("[plugin_protocol] Serving file with content-type: {}", content_type);

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
