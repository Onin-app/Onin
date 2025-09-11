use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ItemSource {
    Application, // System-installed applications
    Custom,      // User-defined items (files, folders, URLs, etc.)
    Command,     // System commands
}

impl Default for ItemSource {
    fn default() -> Self {
        // 对于用户手动添加的项目，默认为 Custom
        ItemSource::Custom
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LaunchableItem {
    pub name: String,
    pub aliases: Vec<String>,
    pub path: String,
    pub icon: String,
    pub icon_type: IconType,
    pub item_type: ItemType,
    pub source: ItemSource,
    pub action: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CommandKeyword {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub name: String,
    pub title: String,
    pub english_name: String,
    pub keywords: Vec<CommandKeyword>,
    pub icon: String,
    pub source: ItemSource,
}
