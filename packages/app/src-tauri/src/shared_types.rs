use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemSource {
    Application, // System-installed applications
    Custom,      // User-defined items (files, folders, URLs, etc.)
    Command,     // System commands
    FileCommand, // User-defined file commands
    FileSearch,  // Indexed local file search results
    Plugin,      // Plugins
    Extension,   // Internal extensions (emoji, calculator, etc.)
}

impl Default for ItemSource {
    fn default() -> Self {
        // 对于用户手动添加的项目，默认为 Custom
        ItemSource::FileCommand
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemType {
    App,
    Folder,
    File,
    // Can add more types like Script, URL, etc. in the future
}

impl Default for ItemType {
    fn default() -> Self {
        ItemType::File // 将 File 作为默认类型
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IconType {
    Base64,
    Iconfont,
    Url, // HTTP/HTTPS URL for remote icons (marketplace plugins)
}

impl Default for IconType {
    fn default() -> Self {
        IconType::Base64
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AppOrigin {
    Hkey,
    Shortcut,
    Uwp,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LaunchableItem {
    pub name: String,
    /// 描述信息，用于前端显示
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub keywords: Vec<CommandKeyword>,
    pub path: String,
    pub icon: String,
    pub icon_type: IconType,
    pub item_type: ItemType,
    pub source: ItemSource,
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AppOrigin>,
    /// Display name for the source (e.g., plugin name instead of "Plugin")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_display: Option<String>,
    /// Match conditions for paste content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<CommandMatch>>,
    /// Last modified time in Unix milliseconds, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<u64>,
    /// 是否需要二次确认(用于敏感操作如关机、重启等)
    #[serde(default)]
    pub requires_confirmation: bool,
}

/// 命令匹配配置
///
/// 三层优雅降级模型：
/// 1. 开发者层：只需配置 extensions
/// 2. 系统层：自动映射为内部 MIME 类型
/// 3. 运行层：优先 MIME 判断，fallback 到 extensions
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandMatch {
    #[serde(rename = "type")]
    pub match_type: String, // "text" | "image" | "file" | "folder"
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CommandKeyword {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandAction {
    System(String),
    App(String), // The string will hold the executable path
    File(String),
    PluginEntry {
        plugin_id: String,
    }, // Open a plugin entry by plugin id
    PluginCommand {
        plugin_id: String,
        command_code: String,
    }, // Plugin command with plugin id and command code
    Extension {
        extension_id: String,
        command_code: String,
    }, // Internal extension command (e.g., emoji, calculator)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub name: String, // A unique identifier, e.g., "shutdown" or a hash for an app
    pub title: String,
    /// 描述信息，用于前端显示
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub english_name: String,
    pub keywords: Vec<CommandKeyword>,
    pub icon: String,
    pub source: ItemSource,
    pub action: CommandAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AppOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<CommandMatch>>,
    /// 是否需要二次确认(用于敏感操作如关机、重启等)
    #[serde(default)]
    pub requires_confirmation: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub shortcut: String,
    pub command_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_title: Option<String>,
}

/// Represents a command registered by a plugin.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommand {
    /// A unique identifier for the command.
    pub name: String,
    /// The name of the command displayed to the user.
    pub label: String,
    /// A brief description of what the command does.
    pub description: String,
    /// A list of keywords to make the command easier to find.
    pub keywords: Vec<String>,
    /// The ID of the plugin that registered this command.
    pub plugin_id: String,
}

/// Represents the result of a command execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandResult {
    /// Indicates that the command executed successfully, returning a string result.
    Success(String),
    /// Indicates that the command failed to execute, returning an error message.
    Error(String),
}

/// Dynamic command registered by a plugin at runtime.
///
/// Unlike static commands defined in manifest.json, dynamic commands
/// can be created and removed programmatically by plugins.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicCommand {
    /// Unique command code within the plugin
    pub code: String,
    /// Display name
    pub name: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Trigger keywords
    #[serde(default)]
    pub keywords: Vec<CommandKeyword>,
    /// Content match rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<CommandMatch>>,
    /// Plugin ID that registered this command
    pub plugin_id: String,
    /// Timestamp when the command was created (milliseconds since epoch)
    pub created_at: u64,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default 实现 ====================

    #[test]
    fn test_item_source_default() {
        assert_eq!(ItemSource::default(), ItemSource::FileCommand);
    }

    #[test]
    fn test_item_type_default() {
        assert_eq!(ItemType::default(), ItemType::File);
    }

    #[test]
    fn test_icon_type_default() {
        assert_eq!(IconType::default(), IconType::Base64);
    }

    // ==================== Serde 序列化/反序列化 ====================

    #[test]
    fn test_item_source_roundtrip() {
        let cases = vec![
            ItemSource::Application,
            ItemSource::Custom,
            ItemSource::Command,
            ItemSource::FileCommand,
            ItemSource::FileSearch,
            ItemSource::Plugin,
            ItemSource::Extension,
        ];
        for source in cases {
            let json = serde_json::to_string(&source).unwrap();
            let deserialized: ItemSource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, deserialized);
        }
    }

    #[test]
    fn test_item_type_roundtrip() {
        let cases = vec![ItemType::App, ItemType::Folder, ItemType::File];
        for t in cases {
            let json = serde_json::to_string(&t).unwrap();
            let deserialized: ItemType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, deserialized);
        }
    }

    #[test]
    fn test_icon_type_roundtrip() {
        let cases = vec![IconType::Base64, IconType::Iconfont, IconType::Url];
        for t in cases {
            let json = serde_json::to_string(&t).unwrap();
            let deserialized: IconType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, deserialized);
        }
    }

    #[test]
    fn test_shortcut_roundtrip() {
        let s = Shortcut {
            shortcut: "Ctrl+K".into(),
            command_name: "test".into(),
            command_title: Some("Test".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Shortcut = serde_json::from_str(&json).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn test_shortcut_without_title() {
        let s = Shortcut {
            shortcut: "Ctrl+K".into(),
            command_name: "test".into(),
            command_title: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: Shortcut = serde_json::from_str(&json).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn test_command_keyword_partial_eq() {
        let a = CommandKeyword {
            name: "hello".into(),
            disabled: None,
            is_default: None,
        };
        let b = CommandKeyword {
            name: "hello".into(),
            disabled: None,
            is_default: None,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_command_keyword_not_equal() {
        let a = CommandKeyword {
            name: "hello".into(),
            disabled: None,
            is_default: None,
        };
        let b = CommandKeyword {
            name: "world".into(),
            disabled: None,
            is_default: None,
        };
        assert_ne!(a, b);
    }

    // ==================== LaunchableItem 默认值 ====================

    #[test]
    fn test_launchable_item_requires_confirmation_default() {
        let item: LaunchableItem = serde_json::from_str(
            r#"{"name":"test","keywords":[],"path":"/","icon":"","icon_type":"Base64","item_type":"File","source":"FileCommand"}"#,
        )
        .unwrap();
        assert!(!item.requires_confirmation);
        assert_eq!(item.name, "test");
    }

    // ==================== CommandAction debug ====================

    #[test]
    fn test_command_action_system() {
        let a = CommandAction::System("shutdown".into());
        assert!(matches!(a, CommandAction::System(ref cmd) if cmd == "shutdown"));
    }

    #[test]
    fn test_command_action_extension() {
        let a = CommandAction::Extension {
            extension_id: "emoji".into(),
            command_code: "search".into(),
        };
        assert!(
            matches!(a, CommandAction::Extension { ref extension_id, ref command_code } if extension_id == "emoji" && command_code == "search")
        );
    }

    #[test]
    fn test_command_action_plugin_command() {
        let a = CommandAction::PluginCommand {
            plugin_id: "plugin-1".into(),
            command_code: "cmd-1".into(),
        };
        assert!(
            matches!(a, CommandAction::PluginCommand { ref plugin_id, ref command_code } if plugin_id == "plugin-1" && command_code == "cmd-1")
        );
    }

    // ==================== 非法 JSON 反序列化 ====================

    #[test]
    fn test_shortcut_deserialize_missing_required() {
        let result: Result<Shortcut, _> = serde_json::from_str(r#"{"shortcut": "Ctrl+K"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_shortcut_deserialize_wrong_type() {
        let result: Result<Shortcut, _> =
            serde_json::from_str(r#"{"shortcut": 42, "command_name": "test"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_match_deserialize_invalid_json() {
        let result: Result<CommandMatch, _> = serde_json::from_str(r#"not valid json"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_keyword_deserialize_unknown_field_ok() {
        let result: Result<CommandKeyword, _> =
            serde_json::from_str(r#"{"name": "test", "unknown_field": "value"}"#);
        assert!(result.is_ok());
    }
}
