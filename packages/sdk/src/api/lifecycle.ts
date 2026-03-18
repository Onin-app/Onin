/**
 * 插件生命周期 API
 *
 * 提供两个简单的生命周期钩子用于插件初始化和清理：
 * - onLoad: 插件加载时调用
 * - onUnload: 插件卸载时调用
 *
 * 注意：窗口事件（显示/隐藏/焦点）已移至 window 模块。
 *
 * @module api/lifecycle
 * @example
 * ```typescript
 * import { lifecycle } from 'sdk';
 *
 * lifecycle.onLoad(async () => {
 *   await settings.useSettingsSchema([...]);
 *   command.register(async (cmd, args) => {...});
 *   console.log('插件已加载！');
 * });
 *
 * lifecycle.onUnload(async () => {
 *   console.log('正在清理...');
 * });
 * ```
 */

import { createError } from '../types/errors';

type LifecycleCallback = () => void | Promise<void>;

let loadCallbacks: LifecycleCallback[] = [];
let unloadCallbacks: LifecycleCallback[] = [];
let loadExecutionScheduled = false;

/**
 * 注册插件加载时的回调函数
 *
 * 此钩子在插件被系统加载时自动运行，在任何用户交互之前。
 * 回调函数在当前事件循环结束时执行，因此你可以注册多个回调，
 * 它们会一起执行。
 *
 * 用途：
 * - 注册设置模式
 * - 注册命令处理器
 * - 初始化默认数据
 * - 设置插件状态
 *
 * @param callback - 插件加载时执行的函数
 *
 * @example
 * ```typescript
 * import { lifecycle, settings, command, storage } from 'sdk';
 *
 * lifecycle.onLoad(async () => {
 *   // 1. 注册设置
 *   await settings.useSettingsSchema([
 *     {
 *       key: 'apiKey',
 *       label: 'API 密钥',
 *       type: 'password',
 *       required: true
 *     }
 *   ]);
 *
 *   // 2. 注册命令处理器
 *   command.register(async (cmd, args) => {
 *     if (cmd === 'get-status') {
 *       return { status: 'ready' };
 *     }
 *   });
 *
 *   // 3. 初始化首次运行数据
 *   const firstRun = await storage.getItem('first-run');
 *   if (firstRun === null) {
 *     await storage.setItem('first-run', false);
 *     await storage.setItem('install-time', new Date().toISOString());
 *   }
 *
 *   console.log('插件初始化成功');
 * });
 * ```
 */
function onLoad(callback: LifecycleCallback): void {
  loadCallbacks.push(callback);

  // Schedule execution if not already scheduled
  if (!loadExecutionScheduled) {
    loadExecutionScheduled = true;
    // Use queueMicrotask to execute at the end of current event loop tick
    queueMicrotask(() => {
      executeLoadCallbacks().catch((error) => {
        console.error('[Lifecycle] Failed to execute onLoad callbacks:', error);
      });
    });
  }
}

/**
 * 注册插件卸载时的回调函数
 *
 * 此钩子在插件被禁用、卸载或应用程序关闭时运行。
 * 用途：
 * - 清理资源
 * - 保存状态
 * - 取消待处理的操作
 * - 关闭连接
 *
 * @param callback - 插件卸载时执行的函数
 *
 * @example
 * ```typescript
 * import { lifecycle, storage } from 'sdk';
 *
 * let intervalId: number;
 *
 * lifecycle.onLoad(() => {
 *   intervalId = setInterval(() => {
 *     console.log('后台任务运行中...');
 *   }, 5000);
 * });
 *
 * lifecycle.onUnload(async () => {
 *   clearInterval(intervalId);
 *   await storage.setItem('last-unload', new Date().toISOString());
 *   console.log('插件已清理');
 * });
 * ```
 */
function onUnload(callback: LifecycleCallback): void {
  unloadCallbacks.push(callback);
}

/**
 * 内部函数：执行所有已注册的加载回调
 * 由插件系统自动调用
 * @internal
 */
async function executeLoadCallbacks(): Promise<void> {
  for (const callback of loadCallbacks) {
    try {
      await callback();
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      console.error('[Lifecycle] Error in onLoad callback:', errorMessage);
      throw createError.common.unknown(
        `onLoad callback failed: ${errorMessage}`,
      );
    }
  }
}

/**
 * 内部函数：执行所有已注册的卸载回调
 * 由插件系统自动调用
 * @internal
 */
async function executeUnloadCallbacks(): Promise<void> {
  for (const callback of unloadCallbacks) {
    try {
      await callback();
    } catch (error) {
      console.error('[Lifecycle] Error in onUnload callback:', error);
      // Don't throw on unload errors, just log them
    }
  }
}

type LifecycleRuntimeGlobals = typeof globalThis & {
  __ONIN_EXECUTE_UNLOAD_CALLBACKS__?: () => Promise<void>;
  __ONIN_RESET_LIFECYCLE_CALLBACKS__?: () => void;
};

/**
 * 内部函数：重置所有生命周期回调
 * 用于测试和插件重新加载
 * @internal
 */
function resetCallbacks(): void {
  loadCallbacks = [];
  unloadCallbacks = [];
  loadExecutionScheduled = false;
}

const lifecycleGlobals = globalThis as LifecycleRuntimeGlobals;
lifecycleGlobals.__ONIN_EXECUTE_UNLOAD_CALLBACKS__ = executeUnloadCallbacks;
lifecycleGlobals.__ONIN_RESET_LIFECYCLE_CALLBACKS__ = resetCallbacks;

/**
 * 生命周期 API 命名空间
 *
 * 提供生命周期钩子：
 * - onLoad: 插件加载时调用（自动执行）
 * - onUnload: 插件卸载时调用
 *
 * 注意：窗口事件请使用 window 模块：
 * - window.onShow: 窗口显示时调用
 * - window.onHide: 窗口隐藏时调用
 * - window.onFocus: 窗口获得焦点时调用
 * - window.onBlur: 窗口失去焦点时调用
 */
export const lifecycle = {
  onLoad,
  onUnload,
  // Internal functions exposed for plugin system
  _executeUnloadCallbacks: executeUnloadCallbacks,
  _resetCallbacks: resetCallbacks,
};
