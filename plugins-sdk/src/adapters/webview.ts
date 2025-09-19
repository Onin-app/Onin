/**
 * @module adapters/ui
 * @description UI 环境（Tauri）的 API 适配器。
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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

/**
 * 在webview插件中注册指令处理器。
 * 通过监听IPC事件来接收指令执行请求。
 * @param handler 指令处理器函数
 * @returns Promise<void>
 */
export async function registerCommandHandler(
  handler: (command: string, args: any) => any | Promise<any>
): Promise<void> {
  // 监听来自宿主应用的指令执行事件
  await listen("plugin_command_execute", async (event) => {
    const { command, args, requestId } = event.payload as {
      command: string;
      args: any;
      requestId: string;
    };

    try {
      const result = await handler(command, args);
      // 发送执行结果回宿主应用
      await invoke("plugin_command_result", {
        requestId,
        success: true,
        result,
      });
    } catch (error) {
      // 发送错误结果回宿主应用
      await invoke("plugin_command_result", {
        requestId,
        success: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  });
}

