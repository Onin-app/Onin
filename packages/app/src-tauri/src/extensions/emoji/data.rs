//! Emoji 数据模块
//!
//! 使用编译时嵌入的 JSON 数据，延迟解析

use crate::extension::types::{EmojiGroup, EmojiItem};
use serde::Deserialize;
use std::sync::LazyLock;

/// 原始 JSON 数据（编译时嵌入）
const EMOJI_JSON: &str = include_str!("emoji_data.json");

/// 原始 JSON emoji 项结构（emojibase 格式）
#[derive(Debug, Deserialize)]
struct RawEmoji {
    emoji: String,
    name: String,
    #[serde(default)]
    tags: Vec<String>,
}

/// 原始 JSON 分组结构
#[derive(Debug, Deserialize)]
struct RawGroup {
    name: String,
    slug: String,
    emojis: Vec<RawEmoji>,
}

/// 解析后的 emoji 数据（延迟初始化）
static EMOJI_DATA: LazyLock<Vec<EmojiGroup>> = LazyLock::new(|| {
    let raw_groups: Vec<RawGroup> =
        serde_json::from_str(EMOJI_JSON).expect("Failed to parse emoji_data.json");

    raw_groups
        .into_iter()
        .map(|group| EmojiGroup {
            name: group.name,
            slug: group.slug,
            emojis: group
                .emojis
                .into_iter()
                .map(|e| EmojiItem {
                    emoji: e.emoji,
                    name: e.name,
                    tags: e.tags,
                })
                .collect(),
        })
        .collect()
});

/// 获取所有 emoji 分组
pub fn get_all_groups() -> Vec<EmojiGroup> {
    EMOJI_DATA.clone()
}

/// 搜索 emoji（匹配名称和 tags）
pub fn search_emojis(query: &str) -> Vec<EmojiGroup> {
    let query_lower = query.to_lowercase();
    let query_parts: Vec<&str> = query_lower.split_whitespace().collect();

    EMOJI_DATA
        .iter()
        .filter_map(|group| {
            let filtered_emojis: Vec<EmojiItem> = group
                .emojis
                .iter()
                .filter(|emoji| {
                    let name_lower = emoji.name.to_lowercase();

                    // 检查所有搜索词是否匹配 name 或 tags
                    query_parts.iter().all(|part| {
                        // 匹配名称
                        if name_lower.contains(part) {
                            return true;
                        }
                        // 匹配任一 tag
                        emoji
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(part))
                    })
                })
                .cloned()
                .collect();

            if filtered_emojis.is_empty() {
                None
            } else {
                Some(EmojiGroup {
                    name: group.name.clone(),
                    slug: group.slug.clone(),
                    emojis: filtered_emojis,
                })
            }
        })
        .collect()
}
