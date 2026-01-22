import pinyin from "pinyin";
import type { LaunchableItem } from "$lib/type";

/**
 * 计算匹配分数
 * 分数越高，匹配越精确
 */
const calculateMatchScore = (query: string, item: LaunchableItem): number => {
  const lowerQuery = query.toLowerCase();
  let maxScore = 0;

  // 检查名称
  const lowerName = item.name.toLowerCase();
  if (lowerName === lowerQuery) {
    maxScore = Math.max(maxScore, 100); // 名称精确匹配
  } else if (lowerName.startsWith(lowerQuery)) {
    maxScore = Math.max(maxScore, 80); // 名称前缀匹配
  } else if (lowerName.includes(lowerQuery)) {
    maxScore = Math.max(maxScore, 60); // 名称包含匹配
  }

  // 检查关键词
  for (const keyword of item.keywords || []) {
    const lowerKeyword = keyword.name.toLowerCase();
    if (lowerKeyword === lowerQuery) {
      maxScore = Math.max(maxScore, 100); // 关键词精确匹配
    } else if (lowerKeyword.startsWith(lowerQuery)) {
      maxScore = Math.max(maxScore, 70); // 关键词前缀匹配
    } else if (lowerKeyword.includes(lowerQuery)) {
      maxScore = Math.max(maxScore, 50); // 关键词包含匹配
    }
  }

  // 首字母匹配
  if (maxScore === 0) {
    const initials = lowerName
      .split(/\s+/)
      .map((word) => word.charAt(0))
      .join("");
    if (initials.includes(lowerQuery)) {
      maxScore = 30;
    }
  }

  // 拼音匹配
  if (maxScore === 0) {
    const pinyinResult = pinyin(item.name, {
      style: pinyin.STYLE_NORMAL,
      heteronym: false,
    })
      .flat()
      .join("")
      .toLowerCase();

    const pinyinInitials = pinyin(item.name, {
      style: pinyin.STYLE_FIRST_LETTER,
    })
      .flat()
      .join("")
      .toLowerCase();

    if (
      pinyinResult.includes(lowerQuery) ||
      pinyinInitials.includes(lowerQuery)
    ) {
      maxScore = 20;
    }
  }

  return maxScore;
};

/**
 * 模糊匹配工具函数
 * @param value 搜索值
 * @param array 要搜索的数组
 * @returns 匹配的结果数组（按匹配分数排序）
 */
export const fuzzyMatch = (
  value: string,
  array: LaunchableItem[],
): LaunchableItem[] => {
  if (!value || !array?.length) return array;

  // 计算每个项目的匹配分数
  const scoredItems = array
    .map((item) => ({ item, score: calculateMatchScore(value, item) }))
    .filter(({ score }) => score > 0);

  // 按分数降序排序
  scoredItems.sort((a, b) => b.score - a.score);

  return scoredItems.map(({ item }) => item);
};
