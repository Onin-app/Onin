use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LaunchableItem {
    pub name: String,
    pub path: String,
    pub icon: String,
    pub item_type: ItemType,
}
