use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub text: String,
    pub timestamp: u64,
    pub item_type: String,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub image_path: Option<String>,
}
