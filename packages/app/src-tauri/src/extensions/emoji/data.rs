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

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_groups_returns_all() {
        let groups = get_all_groups();
        assert!(!groups.is_empty());
        assert!(groups.iter().any(|g| g.slug == "smileys_emotion"));
        assert!(groups.iter().any(|g| g.slug == "people_body"));
    }

    #[test]
    fn test_search_by_name_partial() {
        let groups = search_emojis("heart");
        assert!(!groups.is_empty());
        let all_hearts: Vec<&EmojiItem> = groups.iter().flat_map(|g| g.emojis.iter()).collect();
        assert!(all_hearts.iter().any(|e| e.name.contains("heart")));
    }

    #[test]
    fn test_search_by_tag() {
        let groups = search_emojis("love");
        assert!(!groups.is_empty());
        let all: Vec<&EmojiItem> = groups.iter().flat_map(|g| g.emojis.iter()).collect();
        assert!(all
            .iter()
            .any(|e| e.name.contains("heart") || e.tags.iter().any(|t| t == "love")));
    }

    #[test]
    fn test_search_multi_word_all_match() {
        let groups = search_emojis("crying face");
        assert!(!groups.is_empty());
        let all: Vec<&EmojiItem> = groups.iter().flat_map(|g| g.emojis.iter()).collect();
        assert!(all
            .iter()
            .any(|e| e.name.contains("crying") && e.name.contains("face")));
    }

    #[test]
    fn test_search_case_insensitive() {
        let groups = search_emojis("HEART");
        assert!(!groups.is_empty());
        let groups_lower = search_emojis("heart");
        assert_eq!(
            groups.iter().flat_map(|g| g.emojis.iter()).count(),
            groups_lower.iter().flat_map(|g| g.emojis.iter()).count()
        );
    }

    #[test]
    fn test_search_no_match_returns_empty() {
        let groups = search_emojis("xyznonexistent12345");
        assert!(groups.is_empty() || groups.iter().all(|g| g.emojis.is_empty()));
    }

    #[test]
    fn test_search_empty_query_returns_all() {
        let all = get_all_groups();
        let searched = search_emojis("");
        assert_eq!(all.len(), searched.len());
    }

    #[test]
    fn test_search_whitespace_query() {
        let all = get_all_groups();
        let searched = search_emojis("  ");
        assert_eq!(all.len(), searched.len());
    }
}
