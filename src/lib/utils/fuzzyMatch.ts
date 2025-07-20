import pinyin from 'pinyin';
import type { LaunchableItem } from '$lib/type';

/**
 * 模糊匹配工具函数
 * @param value 搜索值
 * @param array 要搜索的数组
 * @returns 匹配的结果数组
 */
export const fuzzyMatch = (value: string, array: LaunchableItem[]): LaunchableItem[] => {
  if (!value || !array?.length) return array;

  const lowerValue = value.toLowerCase();

  const checkMatch = (text: string): boolean => {
    const lowerText = text.toLowerCase();

    // 规则1: 简单模糊匹配(忽略大小写)
    if (lowerText.includes(lowerValue)) {
      return true;
    }

    // 规则2: 首字母匹配
    const initials = lowerText.split(/\s+/)
      .map(word => word.charAt(0))
      .join('');
    if (initials.includes(lowerValue)) {
      return true;
    }

    // 规则3: 中文拼音匹配
    const pinyinResult = pinyin(text, {
      style: pinyin.STYLE_NORMAL, // 全拼
      heteronym: false
    }).flat().join('').toLowerCase();

    const pinyinInitials = pinyin(text, {
      style: pinyin.STYLE_FIRST_LETTER // 首字母
    }).flat().join('').toLowerCase();

    return pinyinResult.includes(lowerValue) ||
      pinyinInitials.includes(lowerValue);
  };

  return array.reduce<LaunchableItem[]>((results, item) => {
    // 优先匹配名称
    if (checkMatch(item.name)) {
      // 如果名称匹配，直接将原项目（的副本）加入结果列表
      results.push({ ...item });
      return results;
    }

    // 如果名称不匹配，则查找匹配的别名
    if (item.aliases) {
      const matchedAlias = item.aliases.find(alias => checkMatch(alias));
      if (matchedAlias) {
        // 如果找到匹配的别名，创建一个新对象，将 name 设置为该别名
        results.push({ ...item, name: matchedAlias });
      }
    }

    return results;
  }, []);
}
