pub mod commands;

use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult,
    ExtensionResultType, ExtensionMatch,
};

/// 全局 AppHandle 引用
static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

/// 初始化扩展
pub fn init(app: &tauri::AppHandle) {
    let _ = APP_HANDLE.set(app.clone());
}

pub fn get_invoke_handler(
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![commands::open_translator_window]
}

// ============================================================================
// Translator 清单定义
// ============================================================================

pub static TRANSLATOR_MANIFEST: ExtensionManifest = crate::extension::types::ExtensionManifest {
    id: "translator",
    name: "翻译",
    description: "多引擎聚合翻译",
    icon: "translate", 
    commands: &[ExtensionCommand {
        code: "open",
        name: "打开翻译器",
        description: "打开翻译窗口",
        keywords: &[
            "translate",
            "translator",
            "fy",
            "fanyi",
            "翻译",
        ],
        matches: Some(ExtensionMatch {
            pattern: ".*",
            min_length: Some(1),
            max_length: None,
        }),
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

    fn matches(&self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        // 1. 优先检查关键词
        let keyword_match = TRANSLATOR_MANIFEST.commands[0]
            .keywords
            .iter()
            .any(|kw| trimmed.to_lowercase().contains(kw));
        
        if keyword_match {
            return true;
        }

        // 2. 检查正则匹配
        if let Some(match_rule) = &TRANSLATOR_MANIFEST.commands[0].matches {
            // 检查最小长度
            if let Some(min) = match_rule.min_length {
                if trimmed.len() < min {
                    return false;
                }
            }
             // 检查最大长度
             if let Some(max) = match_rule.max_length {
                if trimmed.len() > max {
                    return false;
                }
            }
            
            // 简单正则匹配 (对于 .* 可以直接返回 true)
            if match_rule.pattern == ".*" {
                return true;
            }
            
            // 如果有更复杂的正则，这里需要引入 regex 库编译执行
            // 为保持简单和性能，暂时只处理 .* 或在 manifest 中定义
            // 实际生产环境应该缓存编译好的 Regex
            // 这里为了演示 "暂时支持"，直接通过
            return true; 
        }

        false
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        let mut text_to_translate = input.trim().to_string();

        // Check if input starts with any keyword, if so, strip it
        for kw in TRANSLATOR_MANIFEST.commands[0].keywords {
             if text_to_translate.to_lowercase().starts_with(kw) {
                // simple strip, might be risky if keyword is part of a word, but keywords are spaces usually? 
                // "translator" vs "translatorbot"
                // Let's assume space separator if stripped.
                // or just remove the keyword string.
                
                // Better: if input is exactly keyword, text is empty.
                // if input starts with "keyword ", strip it.
                
                if text_to_translate.to_lowercase() == *kw {
                    text_to_translate = "".to_string();
                    break;
                } else if text_to_translate.to_lowercase().starts_with(&format!("{} ", kw)) {
                     // wait, slicing by byte index of kw length
                     text_to_translate = input.trim()[kw.len()..].trim().to_string();
                     break;
                }
             }
        }

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
            
            // Return a result indicating we launched it
            ExtensionResult {
                success: true,
                value: Some("翻译器已打开".to_string()),
                result_type: ExtensionResultType::Conversion, // Reuse Conversion type
                copyable: None,
                subtitle: None,
                error: None,
            }
        } else {
            ExtensionResult::error("AppHandle not initialized".to_string())
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        if self.matches(input) {
             Some(ExtensionPreview {
                extension_id: "translator".to_string(),
                command_code: "open".to_string(),
                title: format!("翻译: {}", input),
                description: "打开翻译窗口".to_string(),
                icon: "translate".to_string(),
                copyable: input.to_string(),
                view_type: crate::extension::types::PreviewViewType::Single,
                grid_data: None,
            })
        } else {
            None
        }
    }
}


// ExtensionResult helper for convenience (if not already public, I'll assume standard usage)
// Wait, ExtensionResult in types.rs has constructors.
// ExtensionResult::success is not defined in types.rs, checking types.rs again.
// It has `calculation`, `conversion`, `datetime`, `currency`, `error`.
// It does NOT have a generic `success`.
// I should use `ExtensionResult::conversion` or create a new type if needed.
// Or just use `ExtensionResult { success: true, ... }`
