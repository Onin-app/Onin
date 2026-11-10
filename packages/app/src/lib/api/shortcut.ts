import { safeInvoke } from "./ipc";
import type { Shortcut } from "$lib/type";

/**
 * 获取所有自定义快捷键列表
 */
export async function getShortcuts(): Promise<Shortcut[]> {
  return await safeInvoke("get_shortcuts");
}

/**
 * 添加或更新快捷键
 */
export async function addShortcut(
  shortcut: Shortcut,
  oldShortcutStr?: string | null,
): Promise<void> {
  await safeInvoke("add_shortcut", { shortcut, oldShortcutStr });
}

/**
 * 删除快捷键
 */
export async function removeShortcut(shortcutStr: string): Promise<void> {
  await safeInvoke("remove_shortcut", { shortcutStr });
}

/**
 * 获取主窗口唤起快捷键
 */
export async function getToggleShortcut(): Promise<string> {
  return await safeInvoke("get_toggle_shortcut");
}

/**
 * 设置主窗口唤起快捷键
 */
export async function setToggleShortcut(shortcutStr: string): Promise<void> {
  await safeInvoke("set_toggle_shortcut", { shortcutStr });
}

/**
 * 获取分离窗口快捷键
 */
export async function getDetachWindowShortcut(): Promise<string> {
  return await safeInvoke("get_detach_window_shortcut");
}

/**
 * 设置分离窗口快捷键
 */
export async function setDetachWindowShortcut(
  shortcutStr: string,
): Promise<void> {
  await safeInvoke("set_detach_window_shortcut", { shortcutStr });
}

/**
 * 设置是否正在录制快捷键
 */
export async function setShortcutRecording(recording: boolean): Promise<void> {
  await safeInvoke("set_shortcut_recording", { recording });
}
