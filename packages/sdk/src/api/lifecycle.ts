/**
 * 插件生命周期 API
 * 
 * 提供两个简单的生命周期钩子用于插件初始化和清理：
 * - onLoad: 插件加载时调用
 * - onUnload: 插件卸载时调用
 * 
 * headless 插件在 index.js 中实现生命周期
 * view 插件在 lifecycle.js 中实现生命周期（不依赖 DOM）
 * 
 * 回调函数会在当前事件循环结束时自动执行。
 * 
 * @module api/lifecycle
 * @example
 * ```typescript
 * // Headless 插件 (index.js)
 * import { lifecycle, settings, command } from 'baize-plugin-sdk';
 * 
 * lifecycle.onLoad(async () => {
 *   await settings.useSettingsSchema([...]);
 *   command.register(async (cmd, args) => {...});
 *   console.log('Headless 插件已加载！');
 * });
 * 
 * lifecycle.onUnload(async () => {
 *   console.log('正在清理...');
 * });
 * ```
 * 
 * @example
 * ```typescript
 * // View 插件 (lifecycle.js)
 * import { lifecycle, settings, command } from 'baize-plugin-sdk';
 * 
 * lifecycle.onLoad(async () => {
 *   await settings.useSettingsSchema([...]);
 *   command.register(async (cmd, args) => {...});
 *   console.log('View 插件生命周期已加载！');
 *   // 注意：这里不要操作 DOM，UI 代码在单独的 HTML/JS 文件中
 * });
 * 
 * lifecycle.onUnload(async () => {
 *   console.log('View 插件正在卸载...');
 * });
 * ```
 */

import { createError } from '../types/errors';

type LifecycleCallback = () => void | Promise<void>;

let loadCallbacks: LifecycleCallback[] = [];
let unloadCallbacks: LifecycleCallback[] = [];
let windowShowCallbacks: LifecycleCallback[] = [];
let windowHideCallbacks: LifecycleCallback[] = [];
let loadExecutionScheduled = false;
let windowEventsInitialized = false;

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
 * import { lifecycle, settings, command, storage } from 'baize-plugin-sdk';
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
      executeLoadCallbacks().catch(error => {
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
 * import { lifecycle, storage } from 'baize-plugin-sdk';
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
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('[Lifecycle] Error in onLoad callback:', errorMessage);
      throw createError.common.unknown(`onLoad callback failed: ${errorMessage}`);
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

/**
 * 注册插件窗口显示时的回调函数
 * 
 * 支持两种运行模式：
 * - display_mode="window": 独立窗口获得焦点时触发
 * - display_mode="inline": iframe 可见时触发
 * 
 * 用途：
 * - 刷新窗口数据
 * - 恢复定时任务
 * - 更新 UI 状态
 * 
 * @param callback - 窗口显示时执行的函数
 * 
 * @example
 * ```typescript
 * import { lifecycle } from 'baize-plugin-sdk';
 * 
 * lifecycle.onWindowShow(() => {
 *   console.log('窗口已显示');
 *   // 刷新数据
 *   refreshData();
 * });
 * ```
 */
function onWindowShow(callback: LifecycleCallback): void {
  // 检查是否已经注册过相同的回调
  if (windowShowCallbacks.includes(callback)) {
    return;
  }

  windowShowCallbacks.push(callback);
  initializeWindowEvents();
}

/**
 * 注册插件窗口隐藏时的回调函数
 * 
 * 支持两种运行模式：
 * - display_mode="window": 独立窗口失去焦点时触发
 * - display_mode="inline": iframe 隐藏时触发
 * 
 * 用途：
 * - 暂停后台任务
 * - 保存临时状态
 * - 释放资源
 * 
 * @param callback - 窗口隐藏时执行的函数
 * 
 * @example
 * ```typescript
 * import { lifecycle } from 'baize-plugin-sdk';
 * 
 * lifecycle.onWindowHide(() => {
 *   console.log('窗口已隐藏');
 *   // 暂停定时器
 *   pauseTimers();
 * });
 * ```
 */
function onWindowHide(callback: LifecycleCallback): void {
  // 检查是否已经注册过相同的回调
  if (windowHideCallbacks.includes(callback)) {
    return;
  }

  windowHideCallbacks.push(callback);
  initializeWindowEvents();
}

// 防抖：防止短时间内多次触发
// 100ms 的防抖时间足以过滤掉窗口激活过程中的短暂焦点切换
const DEBOUNCE_MS = 100;
let lastShowTime = 0;
let lastHideTime = 0;

/**
 * 执行窗口显示回调（带防抖）
 * @internal
 */
async function executeWindowShowCallbacks(): Promise<void> {
  const now = Date.now();
  if (now - lastShowTime < DEBOUNCE_MS) {
    return;
  }
  lastShowTime = now;

  for (const callback of windowShowCallbacks) {
    try {
      await callback();
    } catch (error) {
      console.error('[Lifecycle] Error in onWindowShow callback:', error);
    }
  }
}

/**
 * 执行窗口隐藏回调（带防抖）
 * @internal
 */
async function executeWindowHideCallbacks(): Promise<void> {
  const now = Date.now();
  if (now - lastHideTime < DEBOUNCE_MS) {
    return;
  }
  lastHideTime = now;

  for (const callback of windowHideCallbacks) {
    try {
      await callback();
    } catch (error) {
      console.error('[Lifecycle] Error in onWindowHide callback:', error);
    }
  }
}

/**
 * 初始化窗口事件监听
 * 支持三种模式（按优先级）：
 * 1. 浏览器原生 API (visibilitychange) - 最简单可靠
 * 2. Tauri 事件系统 - 作为备选方案
 * 3. iframe postMessage - 用于 inline 模式
 * @internal
 */
function initializeWindowEvents(): void {
  if (windowEventsInitialized) {
    return;
  }

  windowEventsInitialized = true;

  if (typeof window === 'undefined') {
    return;
  }

  // 检测运行环境
  const isInIframe = window.self !== window.top;

  if (isInIframe) {
    // iframe 模式：监听来自父窗口的消息
    window.addEventListener('message', (event) => {
      if (event.data && event.data.type === 'plugin-lifecycle-event') {
        const { event: eventName } = event.data;

        if (eventName === 'show') {
          executeWindowShowCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window show callbacks:', error);
          });
        } else if (eventName === 'hide') {
          executeWindowHideCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window hide callbacks:', error);
          });
        }
      }
    });
  } else {
    // 独立窗口模式：监听后端发送的 window_visibility 事件
    let isUsingTauriEvents = false;

    const tryListenWindowVisibility = (attempt = 1, maxAttempts = 10) => {
      const tauri = (window as any).__TAURI__;

      if (!tauri?.event?.listen) {
        if (attempt < maxAttempts) {
          setTimeout(() => tryListenWindowVisibility(attempt + 1), attempt * 100);
        } else {
          // 降级方案：使用 visibilitychange
          setupFallbackVisibilityChange();
        }
        return;
      }

      // 监听后端发送的 window_visibility 事件
      tauri.event.listen('window_visibility', (event: any) => {
        const isVisible = event.payload;

        if (isVisible) {
          executeWindowShowCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window show callbacks:', error);
          });
        } else {
          executeWindowHideCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window hide callbacks:', error);
          });
        }
      }).then(() => {
        // 成功注册 Tauri 事件监听器，标记为已使用
        isUsingTauriEvents = true;
      }).catch((error: Error) => {
        console.error('[Lifecycle] Failed to listen to window_visibility:', error);
        // 如果注册失败，使用降级方案
        setupFallbackVisibilityChange();
      });
    };

    // 降级方案：使用 visibilitychange
    const setupFallbackVisibilityChange = () => {
      // 只有在没有使用 Tauri 事件时才设置降级方案
      if (isUsingTauriEvents) {
        return;
      }

      document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
          executeWindowHideCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window hide callbacks:', error);
          });
        } else {
          executeWindowShowCallbacks().catch((error) => {
            console.error('[Lifecycle] Failed to execute window show callbacks:', error);
          });
        }
      });
    };

    tryListenWindowVisibility();
  }
}

/**
 * 内部函数：重置所有生命周期回调
 * 用于测试和插件重新加载
 * @internal
 */
function resetCallbacks(): void {
  loadCallbacks = [];
  unloadCallbacks = [];
  windowShowCallbacks = [];
  windowHideCallbacks = [];
  loadExecutionScheduled = false;
  windowEventsInitialized = false;
}

/**
 * 生命周期 API 命名空间
 * 
 * 提供生命周期钩子：
 * - onLoad: 插件加载时调用（自动执行）
 * - onUnload: 插件卸载时调用
 * - onWindowShow: 插件窗口显示时调用（仅 window 模式）
 * - onWindowHide: 插件窗口隐藏时调用（仅 window 模式）
 */
export const lifecycle = {
  onLoad,
  onUnload,
  onWindowShow,
  onWindowHide,
  // Internal functions exposed for plugin system
  _executeUnloadCallbacks: executeUnloadCallbacks,
  _resetCallbacks: resetCallbacks,
};
