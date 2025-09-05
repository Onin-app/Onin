import type { PluginSDKFunctions } from './types';
import { detectEnvironment } from './core/environment';
import { createHeadlessAdapter } from './adapters/headless';
import { createUIAdapter } from './adapters/ui';

/**
 * 创建插件 SDK 实例
 * @returns PluginSDKFunctions 实现
 * @throws {PluginSDKError} 当环境不支持时
 */
export function createPluginSDK(): PluginSDKFunctions {
  const environment = detectEnvironment();
  
  switch (environment) {
    case 'headless':
      return createHeadlessAdapter();
    case 'ui':
      return createUIAdapter();
    default:
      // TypeScript 应该确保这里不会到达，但为了安全起见
      throw new Error(`不支持的环境: ${environment}`);
  }
}

/**
 * 便捷的全局 SDK 实例
 * 在模块加载时创建，可直接使用
 */
export const sdk = createPluginSDK();