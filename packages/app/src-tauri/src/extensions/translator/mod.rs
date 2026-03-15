pub mod commands;

use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
    StaticCommandMatch,
};

/// 全局 AppHandle 引用
static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

/// 初始化扩展
pub fn init(app: &tauri::AppHandle) {
    let _ = APP_HANDLE.set(app.clone());
}

// ============================================================================
// Translator 清单定义
// ============================================================================

pub static TRANSLATOR_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "translator",
    name: "翻译",
    description: "多引擎聚合翻译",
    icon: "translate",
    commands: &[ExtensionCommand {
        code: "open",
        keywords: &["translate", "translator", "fy", "fanyi", "翻译"],
        // 使用统一的 CommandMatch 声明式配置
        // 匹配任意文本（≥1 字符），由前端 matchCommand.ts 处理
        matches: Some(&[StaticCommandMatch {
            match_type: "text",
            name: "翻译文本",
            description: "输入要翻译的文本",
            regexp: None,
            min: Some(1),
            max: None,
        }]),
    }],
};

// ============================================================================
// Translator Extension 实现
// ============================================================================

pub struct TranslatorExtension;

pub static TRANSLATOR_EXTENSION: TranslatorExtension = TranslatorExtension;

impl crate::extension::registry::Extension for TranslatorExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &TRANSLATOR_MANIFEST
    }

    /// 实时输入匹配：任何非空文本都可以翻译
    fn custom_matches(&self, input: &str) -> Option<bool> {
        Some(!input.trim().is_empty())
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        let text_to_translate = input.trim().to_string();

        let text_arg = if text_to_translate.is_empty() {
            None
        } else {
            Some(text_to_translate)
        };

        if let Some(app) = APP_HANDLE.get() {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::open_window(&app_clone, text_arg).await {
                    eprintln!("Failed to open translator window: {}", e);
                }
            });

            ExtensionResult {
                success: true,
                value: Some("翻译器已打开".to_string()),
                result_type: ExtensionResultType::Conversion,
                copyable: None,
                subtitle: None,
                error: None,
            }
        } else {
            ExtensionResult::error("AppHandle not initialized".to_string())
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(ExtensionPreview {
            extension_id: "translator".to_string(),
            command_code: "open".to_string(),
            title: format!("翻译: {}", trimmed),
            description: "打开翻译窗口".to_string(),
            icon: "translate".to_string(),
            copyable: trimmed.to_string(),
            view_type: crate::extension::types::PreviewViewType::Single,
            grid_data: None,
        })
    }
}
