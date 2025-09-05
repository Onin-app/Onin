import type { PluginEnvironment } from '../types';
import { PluginSDKError, ERROR_CODES } from '../types';
import { MESSAGES } from '../utils/constants';

/**
 * 检测当前插件运行环境
 * @returns 插件环境类型
 * @throws {PluginSDKError} 当无法检测到支持的环境时
 */
export function detectEnvironment(): PluginEnvironment {
  // 检测 Deno headless 环境
  if (typeof globalThis !== 'undefined' && 
      globalThis.Deno && 
      globalThis.Deno.core && 
      globalThis.Deno.core.ops) {
    return 'headless';
  }

  // 检测 Tauri UI 环境
  if (typeof window !== 'undefined' && 
      window.__TAURI__) {
    return 'ui';
  }

  // 未知环境
  throw new PluginSDKError(
    MESSAGES.UNSUPPORTED_ENVIRONMENT,
    ERROR_CODES.UNSUPPORTED_ENVIRONMENT
  );
}

/**
 * 检查是否为 headless 环境
 * @returns 是否为 headless 环境
 */
export function isHeadlessEnvironment(): boolean {
  try {
    return detectEnvironment() === 'headless';
  } catch {
    return false;
  }
}

/**
 * 检查是否为 UI 环境
 * @returns 是否为 UI 环境
 */
export function isUIEnvironment(): boolean {
  try {
    return detectEnvironment() === 'ui';
  } catch {
    return false;
  }
}