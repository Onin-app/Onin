import { safeInvoke } from "./ipc";

/**
 * 获取窗口关闭锁（防止意外失焦或关闭）
 */
export function acquireWindowCloseLock(): Promise<void> {
  return safeInvoke("acquire_window_close_lock");
}

/**
 * 释放窗口关闭锁
 */
export function releaseWindowCloseLock(): Promise<void> {
  return safeInvoke("release_window_close_lock");
}

/**
 * 关闭主窗口
 */
export function closeMainWindow(): Promise<void> {
  return safeInvoke("close_main_window");
}
