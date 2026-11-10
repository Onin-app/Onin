import { safeInvoke } from "./ipc";
import type { CommandUsageStats } from "$lib/type";

/**
 * 获取所有命令使用统计数据
 */
export function getUsageStats(): Promise<CommandUsageStats[]> {
  return safeInvoke("get_usage_stats");
}

/**
 * 记录命令使用次数
 */
export function recordCommandUsage(commandName: string): Promise<void> {
  return safeInvoke("record_command_usage", { commandName });
}

/**
 * 清除所有命令使用统计数据
 */
export function clearUsageStats(): Promise<void> {
  return safeInvoke("clear_usage_stats");
}
