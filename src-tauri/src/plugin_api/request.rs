use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::command;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    data: Value,
}

#[command]
pub async fn plugin_request(options: RequestOptions) -> Result<Response, String> {
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
                    Value::Array(bytes.into_iter().map(|b| Value::Number(b.into())).collect())
                }
            };

            Ok(Response {
                status,
                status_text,
                headers,
                data,
            })
        }
        Err(e) => {
            if e.is_timeout() {
                Err("Request timed out".to_string())
            } else {
                Err(e.to_string())
            }
        }
    }
}