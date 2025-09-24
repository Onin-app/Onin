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

// 测试对象
export const test = {
  message: "SDK is working!",
  version: "0.0.1",
  timestamp: Date.now()
};

// 创建默认导出对象
const baize = {
  ...request,
  ...notification,
  ...command,
  ...storage,
  invoke,
  listen,
  test,
};

// 默认导出
export default baize;