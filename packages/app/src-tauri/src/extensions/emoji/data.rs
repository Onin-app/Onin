//! Emoji 数据模块
//!
//! 使用编译时嵌入的 JSON 数据，延迟解析

use crate::extension::types::{EmojiGroup, EmojiItem};
use serde::Deserialize;
use std::sync::LazyLock;

/// 原始 JSON 数据（编译时嵌入）
const EMOJI_JSON: &str = include_str!("emoji_data.json");

/// 原始 JSON emoji 项结构
#[derive(Debug, Deserialize)]
struct RawEmoji {
    emoji: String,
    name: String,
    #[serde(default)]
    skin_tone_support: bool,
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
                // 过滤掉支持肤色变体的 emoji（避免重复，只保留基础版本）
                .filter(|e| !e.skin_tone_support)
                .map(|e| EmojiItem {
                    emoji: e.emoji,
                    name: e.name,
                })
                .collect(),
        })
        .collect()
});

/// 获取所有 emoji 分组
pub fn get_all_groups() -> Vec<EmojiGroup> {
    EMOJI_DATA.clone()
}

/// 搜索 emoji（模糊匹配名称）
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
                    // 所有搜索词都必须匹配
                    query_parts.iter().all(|part| name_lower.contains(part))
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
