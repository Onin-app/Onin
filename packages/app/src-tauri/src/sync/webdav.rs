use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastSyncInfo {
    pub last_sync_time: String,
    pub device_id: String,
}

pub struct WebDavClient {
    base_url: String,
    username: String,
    password: String,
    client: reqwest::Client,
}

impl WebDavClient {
    pub fn new(mut base_url: String, username: String, password: String) -> Self {
        // 保证 base_url 以 / 结尾
        if !base_url.ends_with('/') {
            base_url.push('/');
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            username,
            password,
            client,
        }
    }

    /// 获取 Basic Auth 请求头值
    fn get_auth_header(&self) -> HeaderValue {
        let auth = format!("{}:{}", self.username, self.password);
        let encoded = STANDARD.encode(auth);
        HeaderValue::from_str(&format!("Basic {}", encoded)).unwrap()
    }

    /// 拼接完整 URL，保证处理好斜杠
    fn get_full_url(&self, relative_path: &str) -> String {
        let rel = relative_path.trim_start_matches('/');
        format!("{}{}", self.base_url, rel)
    }

    /// 测试连接
    ///
    /// 发送一个 PROPFIND 请求到 base_url，若返回 2xx 或 404（鉴权通过）说明连接成功；若返回 401 说明鉴权失败。
    pub async fn test_connection(&self) -> Result<(), String> {
        let url = self.get_full_url("");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.get_auth_header());
        headers.insert("Depth", HeaderValue::from_static("0"));

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("无法连接至 WebDAV 服务器: {}", e))?;

        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err("用户名或密码有误（部分网盘需使用独立的应用授权密码）".to_string())
        } else {
            Err(format!("连接测试失败，HTTP 状态码: {}", status))
        }
    }

    /// 创建目录（如果不存在）
    ///
    /// WebDAV 使用 MKCOL 命令创建目录。
    pub async fn create_directory(&self, dir_name: &str) -> Result<(), String> {
        let url = self.get_full_url(dir_name);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.get_auth_header());

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("创建目录网络错误: {}", e))?;

        let status = response.status();
        // 201 Created 代表创建成功；405 Method Not Allowed 通常代表目录已经存在。
        if status == reqwest::StatusCode::CREATED
            || status == reqwest::StatusCode::METHOD_NOT_ALLOWED
        {
            Ok(())
        } else {
            Err(format!("创建目录失败, HTTP 状态码: {}", status))
        }
    }

    /// 上传文件 (PUT)
    pub async fn upload_file(&self, remote_path: &str, file_bytes: Vec<u8>) -> Result<(), String> {
        let url = self.get_full_url(remote_path);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.get_auth_header());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| format!("上传文件网络错误: {}", e))?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(format!("上传文件失败, HTTP 状态码: {}", status))
        }
    }

    /// 下载文件 (GET)
    ///
    /// 成功返回文件字节流；若文件不存在，则返回 None；其他网络或鉴权错误返回 Err。
    pub async fn download_file(&self, remote_path: &str) -> Result<Option<Vec<u8>>, String> {
        let url = self.get_full_url(remote_path);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.get_auth_header());

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("下载文件网络错误: {}", e))?;

        let status = response.status();
        if status.is_success() {
            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("读取文件流失败: {}", e))?;
            Ok(Some(bytes.to_vec()))
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Err(format!("下载文件失败, HTTP 状态码: {}", status))
        }
    }
}
