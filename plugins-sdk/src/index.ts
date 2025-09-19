/**
 * @module sdk
 * @description 插件 SDK 主入口，提供统一的 API。
 */

import { getEnvironment, RuntimeEnvironment } from './core/environment';
import * as HeadlessAdapter from './adapters/headless';
import * as WebviewAdapter from './adapters/webview';

// 定义通知选项的类型
export interface NotificationOptions {
  title: string;
  body: string;
}

// 定义指令处理器类型
export type CommandHandler = (command: string, args: any) => any | Promise<any>;

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

/**
 * 注册指令处理器。
 *
 * 当宿主应用执行插件指令时，会调用注册的处理器函数。
 *
 * @param handler 指令处理器函数
 * @returns {Promise<void>} 注册完成时解析的 Promise
 */
export function registerCommandHandler(handler: CommandHandler): Promise<void> {
  const environment = getEnvironment();

  if (environment === RuntimeEnvironment.Headless) {
    HeadlessAdapter.registerCommandHandler(handler);
    return Promise.resolve();
  }

  if (environment === RuntimeEnvironment.Webview) {
    return WebviewAdapter.registerCommandHandler(handler);
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
  registerCommandHandler,
  test,
};

// 默认导出
export default baize;
