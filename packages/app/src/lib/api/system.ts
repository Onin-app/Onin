import { safeInvoke } from "./ipc";

/**
 * 打开应用数据存储目录
 */
export function openAppDataDir(): Promise<void> {
  return safeInvoke("open_app_data_dir");
}

/**
 * 设置托盘图标显示状态
 */
export function setTrayVisibility(visible: boolean): Promise<void> {
  return safeInvoke("set_tray_visibility", { visible });
}

/**
 * 查询托盘图标是否可见
 */
export function isTrayVisible(): Promise<boolean> {
  return safeInvoke("is_tray_visible");
}

/**
 * 发送系统通知
 */
export function showNotification(options: {
  title: string;
  body?: string;
  [key: string]: unknown;
}): Promise<void> {
  return safeInvoke("show_notification", { options });
}
