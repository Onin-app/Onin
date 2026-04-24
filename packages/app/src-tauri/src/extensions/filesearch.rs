use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
};

pub fn init(_app: &tauri::AppHandle) {}

pub static FILE_SEARCH_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "file_search",
    name: "文件搜索",
    description: "搜索本机文件和文件夹",
    icon: "folder",
    commands: &[ExtensionCommand {
        code: "search",
        name: "文件搜索",
        description: Some("进入本地文件搜索模式"),
        icon: Some("folder"),
        keywords: &["文件搜索", "本地搜索", "find", "files", "file", "search"],
        matches: None,
    }],
};

pub struct FileSearchExtension;

pub static FILE_SEARCH_EXTENSION: FileSearchExtension = FileSearchExtension;

impl Extension for FileSearchExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &FILE_SEARCH_MANIFEST
    }

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
