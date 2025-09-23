import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

export interface NotificationOptions {
  title: string;
  body: string;
}

/**
 * 显示一个系统通知。
 *
 * @param options 通知的选项，包括标题和内容。
 * @returns {Promise<void>} 一个在操作完成时解析的 Promise。
 */
export function showNotification(options: NotificationOptions): Promise<void> {
  return dispatch({
    // Webview 环境：Tauri 期望的参数结构是 { options: { ... } }
    webview: () => invoke("show_notification", { options }),
    // Headless 环境：直接传递 options
    headless: () => invoke("show_notification", options),
  });
}