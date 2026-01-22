use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemSource {
    Application, // System-installed applications
    Custom,      // User-defined items (files, folders, URLs, etc.)
    Command,     // System commands
    FileCommand, // User-defined file commands
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
    #[deprecated(note = "Use PluginCommand instead. Will be removed in v2.0")]
    Plugin(String), // The string will hold the plugin id (deprecated, kept for compatibility)
    // TODO: Remove Plugin(String) variant in v2.0 and migrate existing data
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
