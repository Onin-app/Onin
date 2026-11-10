import { safeInvoke } from "./ipc";
import type { Shortcut } from "$lib/type";

/**
 * 获取所有自定义快捷键列表
 */
export function getShortcuts(): Promise<Shortcut[]> {
  return safeInvoke("get_shortcuts");
}

/**
 * 添加或更新快捷键
 */
export function addShortcut(
  shortcut: Shortcut,
  oldShortcutStr?: string | null,
): Promise<void> {
  return safeInvoke("add_shortcut", { shortcut, oldShortcutStr });
}

/**
 * 删除快捷键
 */
export function removeShortcut(shortcutStr: string): Promise<void> {
  return safeInvoke("remove_shortcut", { shortcutStr });
}

/**
 * 获取主窗口唤起快捷键
 */
export function getToggleShortcut(): Promise<string> {
  return safeInvoke("get_toggle_shortcut");
}

/**
 * 设置主窗口唤起快捷键
 */
export function setToggleShortcut(shortcutStr: string): Promise<void> {
  return safeInvoke("set_toggle_shortcut", { shortcutStr });
}

/**
 * 获取分离窗口快捷键
 */
export function getDetachWindowShortcut(): Promise<string> {
  return safeInvoke("get_detach_window_shortcut");
}

/**
 * 设置分离窗口快捷键
 */
export function setDetachWindowShortcut(shortcutStr: string): Promise<void> {
  return safeInvoke("set_detach_window_shortcut", { shortcutStr });
}

/**
 * 设置是否正在录制快捷键
 */
export function setShortcutRecording(recording: boolean): Promise<void> {
  return safeInvoke("set_shortcut_recording", { recording });
}
