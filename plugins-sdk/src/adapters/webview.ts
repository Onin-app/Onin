/**
 * @module adapters/ui
 * @description UI 环境（Tauri）的 API 适配器。
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * 在浏览器 Tauri 环境中显示通知。
 * 
 * @param options 通知选项。
 * @param options.title 通知的标题。
 * @param options.body 通知的正文。
 * @returns {Promise<void>} 一个在通知发送后解析的 Promise。
 */
export function showNotification(options: { title: string; body: string }): Promise<void> {
  return invoke("show_notification", { options });
}