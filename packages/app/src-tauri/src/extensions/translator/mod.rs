pub mod commands;

use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult,
    ExtensionResultType,
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
    description: "多引擎聚合翻译 (Google, DeepL)",
    icon: "translate", 
    commands: &[ExtensionCommand {
        code: "open",
        name: "打开翻译器",
        description: "打开包含 Google 和 DeepL 的翻译窗口",
        keywords: &[
            "translate",
            "translator",
            "fy",
            "翻译",
            "google",
            "deepl",
        ],
        matches: None,
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
        let trimmed = input.trim().to_lowercase();
        TRANSLATOR_MANIFEST.commands[0]
            .keywords
            .iter()
            .any(|kw| trimmed.contains(kw))
    }

    fn execute(&self, _input: &str) -> ExtensionResult {
        if let Some(app) = APP_HANDLE.get() {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::open_window(&app_clone).await {
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

    fn preview(&self, _input: &str) -> Option<ExtensionPreview> {
        None
    }
}


// ExtensionResult helper for convenience (if not already public, I'll assume standard usage)
// Wait, ExtensionResult in types.rs has constructors.
// ExtensionResult::success is not defined in types.rs, checking types.rs again.
// It has `calculation`, `conversion`, `datetime`, `currency`, `error`.
// It does NOT have a generic `success`.
// I should use `ExtensionResult::conversion` or create a new type if needed.
// Or just use `ExtensionResult { success: true, ... }`
