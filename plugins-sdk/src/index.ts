/**
 * @module sdk
 * @description 插件 SDK 主入口，提供统一的 API。
 */

// Core utilities
export { invoke, listen } from './core/ipc';
export { getEnvironment, RuntimeEnvironment } from './core/environment';

// APIs
export * from './api/request';
export * from './api/notification';
export * from './api/command';
export * from './api/storage';

// Import APIs for default export
import * as request from './api/request';
import * as notification from './api/notification';
import * as command from './api/command';
import * as storage from './api/storage';
import { invoke, listen } from './core/ipc';
import { getEnvironment } from './core/environment';

// SDK 信息和调试工具
export const debug = {
  version: "0.0.1",
  getEnvironment,
  getRuntimeInfo: () => ({
    timestamp: Date.now(),
    userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Deno Runtime',
    platform: typeof navigator !== 'undefined' ? navigator.platform : 'Unknown'
  }),
  async testConnection() {
    try {
      // 测试基础的 invoke 连接
      const result = await invoke('plugin_test_connection', {});
      return { success: true, result };
    } catch (error) {
      return { success: false, error: error instanceof Error ? error.message : String(error) };
    }
  }
};

// 创建默认导出对象
const baize = {
  ...request,
  ...notification,
  ...command,
  ...storage,
  invoke,
  listen,
  debug,
};

// 默认导出
export default baize;