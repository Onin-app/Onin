//! 剪贴板类型定义

use serde::{Deserialize, Serialize};

/// 剪贴板错误
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardError {
    pub name: String,
    pub message: String,
    pub code: Option<String>,
}

impl From<String> for ClipboardError {
    fn from(message: String) -> Self {
        ClipboardError {
            name: "ClipboardError".to_string(),
            message,
            code: None,
        }
    }
}

impl From<&str> for ClipboardError {
    fn from(message: &str) -> Self {
        ClipboardError {
            name: "ClipboardError".to_string(),
            message: message.to_string(),
            code: None,
        }
    }
}

/// 写入文本选项
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteTextOptions {
    pub text: String,
}

/// 写入图片选项
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteImageOptions {
    #[serde(rename = "imageData")]
    pub image_data: Vec<u8>,
}

/// 剪贴板文件信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardFile {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

/// 剪贴板内容
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub files: Option<Vec<ClipboardFile>>,
    /// 剪贴板内容的时间戳（Unix 时间戳，秒）
    pub timestamp: Option<u64>,
}

/// 剪贴板内容类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardContentType {
    Text,
    Image,
    Files,
    Empty,
}

/// 剪贴板元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardMetadata {
    /// 剪贴板文本内容
    pub text: Option<String>,
    /// 文件路径列表
    pub files: Option<Vec<ClipboardFile>>,
    /// 内容类型
    #[serde(rename = "contentType")]
    pub content_type: ClipboardContentType,
    /// Unix 时间戳（秒）
    pub timestamp: Option<u64>,
    /// 距离当前时间的秒数
    pub age: Option<u64>,
}
