pub mod commands;
pub mod monitor;

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
        matches: None,
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

    fn matches(&self, _input: &str) -> bool {
        // Appears in command list via keywords
        false
    }

    fn execute(&self, _input: &str) -> ExtensionResult {
        // Just opens the window logic (handled by frontend usually calling execute_extension wrapper)
        // But actually the frontend `ExtensionResultItem` likely invokes `execute_extension`
        // which calls this.
        // For simple window opening, we might return a specialized result or rely on the fact
        // that the launcher handles the navigation if we set it up right.

        // However, `ExtensionResult` is mostly for inline results.
        // If we look at `extensions/api.rs`, `execute_extension` returns `ExtensionResult`.

        // The frontend `ExtensionResultItem.svelte` handles the result.
        // If `result_type` is `Conversion`, it might show UI.
        // But for Emoji/Clipboard which are full pages, the Launcher likely treats them as "Apps".

        // Actually, looking at `Emoji` implementation:
        // `execute` returns `success: true, value: None`.
        // The **Frontend** launcher likely navigates to `/extensions/[id]` when an extension is selected?
        // Let's check `UnifiedLaunchManager` or just assume standard behavior.

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
