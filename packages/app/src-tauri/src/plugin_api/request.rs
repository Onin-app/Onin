use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tauri::{command, AppHandle, State};
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Json,
    Text,
    Arraybuffer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestOptions {
    url: String,
    method: Option<HttpMethod>,
    headers: Option<HashMap<String, String>>,
    body: Option<Value>,
    timeout: Option<u64>,
    response_type: Option<ResponseType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestError {
    name: String,
    message: String,
    url: Option<String>,
    timeout: Option<u64>,
    response: Option<Response>,
}

impl From<String> for RequestError {
    fn from(message: String) -> Self {
        RequestError {
            name: "RequestError".to_string(),
            message,
            url: None,
            timeout: None,
            response: None,
        }
    }
}

// 简单的请求函数，类似于 show_notification
pub async fn make_request(
    _app: AppHandle,
    options: RequestOptions,
) -> Result<serde_json::Value, String> {
    // TODO: 实现插件 ID 获取机制后启用权限检查
    // 暂时跳过权限检查，允许所有请求
    let client = reqwest::Client::new();
    let method = match options.method.unwrap_or(HttpMethod::Get) {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    let mut request_builder = client.request(method, &options.url);

    if let Some(headers) = options.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }
    }

    if let Some(body) = options.body {
        if body.is_object() || body.is_array() {
            request_builder = request_builder.json(&body);
        } else if body.is_string() {
            request_builder = request_builder.body(body.as_str().unwrap().to_string());
        }
    }

    if let Some(timeout) = options.timeout {
        request_builder = request_builder.timeout(Duration::from_millis(timeout));
    }

    let response = request_builder.send().await;

    match response {
        Ok(res) => {
            let status = res.status().as_u16();
            let status_text = res.status().canonical_reason().unwrap_or("").to_string();
            let mut headers = HashMap::new();
            for (key, value) in res.headers().iter() {
                headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
            }

            let response_type = options.response_type.unwrap_or(ResponseType::Json);
            let data = match response_type {
                ResponseType::Json => res.json::<Value>().await.map_err(|e| e.to_string())?,
                ResponseType::Text => Value::String(res.text().await.map_err(|e| e.to_string())?),
                ResponseType::Arraybuffer => {
                    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
                    // 使用 base64 编码传输二进制数据
                    let base64_data = general_purpose::STANDARD.encode(&bytes);
                    Value::String(base64_data)
                }
            };

            let response = Response {
                status,
                status_text,
                headers,
                body: data,
            };

            // 直接返回序列化的 JSON 值
            serde_json::to_value(response).map_err(|e| e.to_string())
        }
        Err(e) => {
            if e.is_timeout() {
                Err(format!(
                    "Request to {} timed out after {}ms",
                    options.url,
                    options.timeout.unwrap_or(30000)
                ))
            } else if e.is_connect() {
                Err(format!("Network error: {}", e))
            } else {
                Err(e.to_string())
            }
        }
    }
}

#[command]
pub async fn plugin_request(
    _app: AppHandle,
    _plugin_store: State<'_, crate::plugin::PluginStore>,
    options: RequestOptions,
) -> Result<Response, RequestError> {
    // 这个函数保留给 Webview 插件使用
    match make_request(_app, options).await {
        Ok(response_value) => {
            serde_json::from_value(response_value).map_err(|e| RequestError::from(e.to_string()))
        }
        Err(e) => Err(RequestError::from(e)),
    }
}

#[allow(dead_code)]
async fn check_http_permission(
    _app: &AppHandle,
    plugin_store: &State<'_, crate::plugin::PluginStore>,
    plugin_id: &str,
    url: &str,
) -> Result<(), RequestError> {
    // 解析请求的 URL
    let request_url = Url::parse(url).map_err(|_| RequestError {
        name: "RequestError".to_string(),
        message: format!("Invalid URL: {}", url),
        url: Some(url.to_string()),
        timeout: None,
        response: None,
    })?;

    // 获取插件信息
    let plugin = {
        let plugins = plugin_store.0.lock().unwrap();
        plugins.get(plugin_id).cloned()
    }
    .ok_or_else(|| RequestError {
        name: "RequestError".to_string(),
        message: format!("Plugin '{}' not found", plugin_id),
        url: Some(url.to_string()),
        timeout: None,
        response: None,
    })?;

    // 检查插件 manifest 中的 HTTP 权限
    if let Some(permissions) = &plugin.manifest.permissions {
        if let Some(http_permission) = &permissions.http {
            // 首先检查 HTTP 权限是否启用
            if !http_permission.enable {
                return Err(RequestError {
                    name: "PermissionDeniedError".to_string(),
                    message: format!(
                        "HTTP permission is disabled for plugin. Please set 'permissions.http.enable' to true in your manifest.json."
                    ),
                    url: Some(url.to_string()),
                    timeout: None,
                    response: None,
                });
            }

            // 检查 URL 是否在允许列表中
            for allowed_url in &http_permission.allow_urls {
                if is_url_allowed(&request_url, allowed_url) {
                    return Ok(());
                }
            }
        }
    }

    // 权限检查失败
    Err(RequestError {
        name: "PermissionDeniedError".to_string(),
        message: format!(
            "Permission denied for URL: {}. Please add it to the 'permissions.http.allowUrls' array in your manifest.json.",
            url
        ),
        url: Some(url.to_string()),
        timeout: None,
        response: None,
    })
}

#[allow(dead_code)]
fn is_url_allowed(request_url: &Url, permission: &str) -> bool {
    // 解析权限 URL
    let permission_url = match Url::parse(permission) {
        Ok(url) => url,
        Err(_) => return false,
    };

    // 检查协议
    if request_url.scheme() != permission_url.scheme() {
        return false;
    }

    // 检查端口
    if request_url.port() != permission_url.port() {
        return false;
    }

    // 检查主机名（支持通配符）
    let request_host = request_url.host_str().unwrap_or("");
    let permission_host = permission_url.host_str().unwrap_or("");

    if permission_host.starts_with("*.") {
        // 通配符匹配
        let domain_suffix = &permission_host[2..]; // 去掉 "*."
        request_host.ends_with(domain_suffix) && request_host != domain_suffix
    } else {
        // 精确匹配
        request_host == permission_host
    }
}
