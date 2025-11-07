import type { LaunchableItem, CommandUsageStats, SortMode } from "$lib/type";

/**
 * 根据使用频率对项目进行排序
 */
export function sortByUsage(
  items: LaunchableItem[],
  usageStats: CommandUsageStats[],
  sortMode: SortMode,
  enableTracking: boolean
): LaunchableItem[] {
  // 如果未启用追踪或使用默认模式，返回原数组
  if (!enableTracking || sortMode === "default") {
    return items;
  }

  // 创建使用统计的映射
  const statsMap = new Map<string, CommandUsageStats>();
  usageStats.forEach(stat => {
    statsMap.set(stat.command_name, stat);
  });

  // 计算分数
  const calculateScore = (item: LaunchableItem): number => {
    if (!item.action) return 0;
    
    const stat = statsMap.get(item.action);
    if (!stat) return 0;

    const usageCount = stat.usage_count;
    const lastUsed = stat.last_used;

    switch (sortMode) {
      case "smart": {
        // 智能排序：综合频率和最近使用
        const recencyScore = calculateRecencyScore(lastUsed);
        return usageCount * 0.7 + recencyScore * 0.3;
      }
      case "frequency":
        // 纯频率排序
        return usageCount;
      case "recent":
        // 最近使用排序
        return calculateRecencyScore(lastUsed);
      default:
        return 0;
    }
  };

  const calculateRecencyScore = (lastUsed: number): number => {
    const now = Math.floor(Date.now() / 1000);
    const daysAgo = (now - lastUsed) / 86400; // 转换为天数
    // 使用指数衰减：最近使用的分数更高
    return 100 * Math.exp(-daysAgo / 10);
  };

  // 复制数组并排序
  const sortedItems = [...items];
  sortedItems.sort((a, b) => {
    const scoreA = calculateScore(a);
    const scoreB = calculateScore(b);
    // 降序排序（分数高的在前）
    return scoreB - scoreA;
  });

  return sortedItems;
}
