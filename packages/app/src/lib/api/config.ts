import { safeInvoke } from "./ipc";
import type { AppConfig } from "$lib/type";

/**
 * 获取全局应用配置
 */
export async function getAppConfig(): Promise<AppConfig> {
  return await safeInvoke("get_app_config");
}

/**
 * 更新全局应用配置
 */
export async function updateAppConfig(config: AppConfig): Promise<void> {
  await safeInvoke("update_app_config", { config });
}
