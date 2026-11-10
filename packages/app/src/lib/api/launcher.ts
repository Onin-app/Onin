import { safeInvoke } from "./ipc";
import type { LaunchableItem } from "$lib/type";

/**
 * 获取所有可启动项目（应用、指令等）
 */
export function getAllLaunchableItems(): Promise<LaunchableItem[]> {
  return safeInvoke("get_all_launchable_items");
}

/**
 * 通过系统启动本地应用
 */
export function openApp(path: string): Promise<void> {
  return safeInvoke("open_app", { path });
}
