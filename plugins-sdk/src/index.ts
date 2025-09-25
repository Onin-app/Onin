/**
 * @module sdk
 * @description 插件 SDK 主入口，提供统一的 API。
 */

// Core utilities
export { invoke, listen } from './core/ipc';
export { getEnvironment, RuntimeEnvironment } from './core/environment';

// 直接导入各个模块的命名空间对象
export { http } from './api/request';
export { storage } from './api/storage';
export { notification } from './api/notification';
export { command } from './api/command';
export { fs } from './api/fs';
export { dialog } from './api/dialog';
export { clipboard } from './api/clipboard';
import { invoke, listen } from './core/ipc';
import { getEnvironment } from './core/environment';
import { http } from './api/request';
import { storage } from './api/storage';
import { notification } from './api/notification';
import { command } from './api/command';
import { fs } from './api/fs';
import { dialog } from './api/dialog';
import { clipboard } from './api/clipboard';

// 为了向后兼容，仍然导出原始 API（但不推荐使用）
// export * from './api/request';
// export * from './api/notification';
// export * from './api/command';
// export * from './api/storage';

// 类型定义
export type {
  HttpPermission,
  StoragePermission,
  NotificationPermission,
  CommandPermission,
  FileSystemPermission,
  DialogPermission,
  ClipboardPermission,
  PluginPermissions,
  PluginManifest
} from './types/permissions';

// 文件系统相关类型
export type {
  FileSystemError,
  FileInfo
} from './api/fs';

// 对话框相关类型
export type {
  DialogError,
  MessageDialogOptions,
  ConfirmDialogOptions,
  DialogFilter,
  OpenDialogOptions,
  SaveDialogOptions
} from './api/dialog';

// 剪贴板相关类型
export type {
  ClipboardError
} from './api/clipboard';

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

// 创建默认导出对象 - 使用命名空间结构
const baize = {
  http,
  storage,
  notification,
  command,
  fs,
  dialog,
  clipboard,
  invoke,
  listen,
  debug,
};

// 默认导出
export default baize;