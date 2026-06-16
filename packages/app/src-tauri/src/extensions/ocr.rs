use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
    StaticCommandMatch,
};

pub fn init(_app: &tauri::AppHandle) {}

pub static OCR_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "ocr",
    name: "文字识别",
    description: "Onin 官方 OCR 扩展，支持识别剪贴板或拖拽的图片中的文字",
    icon: "scan",
    commands: &[ExtensionCommand {
        code: "recognize",
        name: "文字识别",
        description: Some("从剪贴板或拖拽的图片中识别文字"),
        icon: Some("scan"),
        keywords: &[],
        matches: Some(&[StaticCommandMatch {
            match_type: "image",
            name: "文字识别",
            description: "识别图片中的文字",
            regexp: None,
            min: Some(1),
            max: None,
        }]),
    }],
};

pub struct OCRExtension;
pub static OCR_EXTENSION: OCRExtension = OCRExtension;

impl Extension for OCRExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &OCR_MANIFEST
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        // OCR 的执行和界面逻辑流转至前端 Svelte 页面处理
        ExtensionResult {
            success: true,
            value: Some(input.to_string()),
            result_type: ExtensionResultType::Conversion,
            copyable: None,
            subtitle: None,
            error: None,
        }
    }

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}
