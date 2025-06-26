import pinyin from 'pinyin';
import type { AppInfo } from '$lib/type';

/**
 * 模糊匹配工具函数
 * @param value 搜索值
 * @param array 要搜索的数组
 * @returns 匹配的结果数组
 */
export const fuzzyMatch = (value: string, array: AppInfo[]): AppInfo[] => {
  if (!value || !array?.length) return array;

  const lowerValue = value.toLowerCase();

  return array.filter(item => {
    const name = item.name.toLowerCase();

    // 规则1: 简单模糊匹配(忽略大小写)
    if (name.toLowerCase().includes(lowerValue)) {
      return true;
    }

    // 规则2: 首字母匹配
    const initials = name.split(/\s+/)
      .map(word => word.charAt(0).toLowerCase())
      .join('');
    if (initials.includes(lowerValue)) {
      return true;
    }

    // 规则3: 中文拼音匹配
    const pinyinResult = pinyin(name, {
      style: pinyin.STYLE_NORMAL, // 全拼
      heteronym: false
    }).flat().join('');

    const pinyinInitials = pinyin(name, {
      style: pinyin.STYLE_FIRST_LETTER // 首字母
    }).flat().join('');

    return pinyinResult.includes(lowerValue) ||
      pinyinInitials.includes(lowerValue);
  });
}
