pub mod commands;
pub mod monitor;
pub mod storage;
pub mod types;

pub use monitor::init;

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
};

// ============================================================================
// Clipboard Manifest
// ============================================================================

pub static CLIPBOARD_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "clipboard",
    name: "Clipboard History",
    description: "View and paste from clipboard history",
    icon: "clipboard",
    commands: &[ExtensionCommand {
        code: "history",
        name: "Clipboard History",
        description: "Search and paste from clipboard history",
        keywords: &["clipboard", "history", "paste", "cp", "jiantieban"],
        matches: None, // 不参与匹配指令，仅通过关键词触发
    }],
};

// ============================================================================
// Clipboard Extension Implementation
// ============================================================================

pub struct ClipboardExtension;

pub static CLIPBOARD_EXTENSION: ClipboardExtension = ClipboardExtension;

impl Extension for ClipboardExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &CLIPBOARD_MANIFEST
    }

    // 不覆盖 custom_matches()，使用默认行为
    // Clipboard 作为命令出现在列表中，不需要匹配指令

    fn execute(&self, _input: &str) -> ExtensionResult {
        ExtensionResult {
            success: true,
            value: None,
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
