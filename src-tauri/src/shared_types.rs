use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemSource {
    Application, // System-installed applications
    Custom,      // User-defined items (files, folders, URLs, etc.)
    Command,     // System commands
    FileCommand, // User-defined file commands
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
    pub keywords: Vec<CommandKeyword>,
    pub path: String,
    pub icon: String,
    pub icon_type: IconType,
    pub item_type: ItemType,
    pub source: ItemSource,
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AppOrigin>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub name: String, // A unique identifier, e.g., "shutdown" or a hash for an app
    pub title: String,
    pub english_name: String,
    pub keywords: Vec<CommandKeyword>,
    pub icon: String,
    pub source: ItemSource,
    pub action: CommandAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AppOrigin>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub shortcut: String,
    pub command_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_title: Option<String>,
}
