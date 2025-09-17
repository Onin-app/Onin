/**
 * @module sdk
 * @description 插件 SDK 主入口，提供统一的 API。
 */

import { getEnvironment, RuntimeEnvironment } from './core/environment';
import * as HeadlessAdapter from './adapters/headless';
import * as WebviewAdapter from './adapters/webview';
import { invoke } from '@tauri-apps/api/core';

// 定义插件命令的类型
export interface PluginCommand {
  name: string;
  label: string;
  description: string;
  keywords: string[];
  pluginId: string;
}

/**
 * 注册一个插件命令。
 *
 * @param command 要注册的命令，除了 `pluginId`。
 * @returns {Promise<any>} 一个在操作完成时解析的 Promise。
 */
export async function registerCommand(command: Omit<PluginCommand, 'pluginId'>): Promise<any> {
  console.log("Registering command:", command);
  // TODO: 将来从插件上下文中动态获取 pluginId
  const pluginId = "temp-plugin-id";
  const fullCommand: PluginCommand = {
    ...command,
    pluginId,
  };
  return invoke('register_plugin_command', { command: fullCommand });
}

// 定义通知选项的类型
export interface NotificationOptions {
  title: string;
  body: string;
}

/**
 * 显示一个系统通知。
 *
 * 这个函数会动态检测运行环境，并调用相应的适配器来发送通知。
 *
 * @param options 通知的选项，包括标题和内容。
 * @returns {Promise<any>} 一个在操作完成时解析的 Promise。
 */
export function showNotification(options: NotificationOptions): Promise<any> {
  const environment = getEnvironment();

  if (environment === RuntimeEnvironment.Headless) {
    return HeadlessAdapter.showNotification(options);
  }

  if (environment === RuntimeEnvironment.Webview) {
    // 浏览器 Tauri 环境，使用 Webview 适配器
    return WebviewAdapter.showNotification(options);
  }

  return Promise.reject(new Error(`Unsupported environment: ${environment}`));
}

// 测试对象
export const test = {
  message: "SDK is working!",
  version: "0.0.1",
  timestamp: Date.now()
};

// 创建默认导出对象
const baize = {
  showNotification,
  registerCommand,
  test,
};

// 默认导出
export default baize;
