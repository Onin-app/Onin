use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, ExtensionResultType,
    StaticCommandMatch,
};

pub fn init(_app: &tauri::AppHandle) {}

pub static AI_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "ai",
    name: "AI 助手",
    description: "Onin 官方 AI 扩展，支持自由聊天与上下文自动匹配问答",
    icon: "sparkles",
    commands: &[
        ExtensionCommand {
            code: "chat",
            name: "AI 问答",
            description: Some("进入 AI 自由对话模式"),
            icon: Some("sparkles"),
            keywords: &["ai", "chat", "ask", "问答", "对话", "liaotian", "wd"],
            matches: None,
        },
        ExtensionCommand {
            code: "action",
            name: "AI 上下文分析",
            description: Some("对输入的文本或剪贴板内容进行 AI 提问与分析"),
            icon: Some("sparkles"),
            keywords: &["ai", "action", "分析", "explain", "summarize", "fenxi"],
            matches: Some(&[StaticCommandMatch {
                match_type: "text",
                name: "分析文本",
                description: "输入需要分析的内容",
                regexp: None,
                min: Some(1),
                max: None,
            }]),
        },
    ],
};

pub struct AIExtension;
pub static AI_EXTENSION: AIExtension = AIExtension;

impl Extension for AIExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &AI_MANIFEST
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        // AI 的执行全部流转至前端 Svelte 页面处理
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
