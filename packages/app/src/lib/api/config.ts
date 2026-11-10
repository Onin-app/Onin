import { safeInvoke } from "./ipc";
import type { AppConfig } from "$lib/type";

/**
 * 获取全局应用配置
 */
export function getAppConfig(): Promise<AppConfig> {
  return safeInvoke("get_app_config");
}

/**
 * 更新全局应用配置
 */
export function updateAppConfig(config: AppConfig): Promise<void> {
  return safeInvoke("update_app_config", { config });
}
