/**
 * 窗口管理 API
 *
 * 提供窗口事件监听 API，支持 inline（内嵌 native webview）和 window（独立窗口）两种模式。
 * 使用生命周期消息适配器，从宿主窗口接收事件。
 *
 * @module api/window
 * @example
 * ```typescript
 * import { pluginWindow } from '@anthropic/sdk';
 *
 * // 监听窗口显示
 * pluginWindow.onShow(() => {
 *   console.log('窗口显示了');
 *   refreshData();
 * });
 *
 * // 监听窗口隐藏
 * pluginWindow.onHide(() => {
 *   console.log('窗口隐藏了');
 *   pauseTimers();
 * });
 *
 * // 监听窗口获得焦点
 * pluginWindow.onFocus(() => {
 *   console.log('窗口获得焦点');
 * });
 *
 * // 监听窗口失去焦点
 * pluginWindow.onBlur(() => {
 *   console.log('窗口失去焦点');
 * });
 * ```
 */

import { LifecycleMessageAdapter } from '../core/adapters/lifecycle-message';
import type { EventCallback } from '../core/adapters/base';

// 使用统一的生命周期消息适配器
let adapter: LifecycleMessageAdapter | null = null;

function getAdapter(): LifecycleMessageAdapter {
  if (!adapter) {
    console.log('[pluginWindow] Creating LifecycleMessageAdapter');
    adapter = new LifecycleMessageAdapter();
  }
  return adapter;
}

/**
 * 注册窗口显示回调
 *
 * 当窗口从隐藏状态变为可见时触发。
 * - inline 模式：内嵌 webview 变为可见时触发
 * - window 模式：独立窗口显示或从最小化恢复时触发
 *
 * @param callback - 窗口显示时执行的回调函数
 */
function onShow(callback: EventCallback): void {
  getAdapter().onShow(callback);
}

/**
 * 注册窗口隐藏回调
 *
 * 当窗口从可见状态变为隐藏时触发。
 * - inline 模式：内嵌 webview 被隐藏时触发
 * - window 模式：独立窗口被最小化或隐藏时触发
 *
 * @param callback - 窗口隐藏时执行的回调函数
 */
function onHide(callback: EventCallback): void {
  getAdapter().onHide(callback);
}

/**
 * 注册窗口获得焦点回调
 *
 * 当窗口获得焦点时触发。
 *
 * @param callback - 窗口获得焦点时执行的回调函数
 */
function onFocus(callback: EventCallback): void {
  getAdapter().onFocus(callback);
}

/**
 * 注册窗口失去焦点回调
 *
 * 当窗口失去焦点时触发。
 *
 * @param callback - 窗口失去焦点时执行的回调函数
 */
function onBlur(callback: EventCallback): void {
  getAdapter().onBlur(callback);
}

/**
 * 获取当前运行模式
 *
 * @returns 'inline' 或 'window'，如果尚未收到运行时信息则返回 'unknown'
 */
function getMode(): 'inline' | 'window' | 'unknown' {
  const runtime = getAdapter().getRuntimeSync();
  return runtime?.mode ?? 'unknown';
}

/**
 * 异步获取当前运行模式
 *
 * 等待运行时信息初始化后返回
 *
 * @returns Promise<'inline' | 'window'>
 */
async function getModeAsync(): Promise<'inline' | 'window'> {
  const runtime = await getAdapter().getRuntime();
  return runtime.mode;
}

/**
 * 获取插件 ID
 *
 * @returns 插件 ID，如果尚未收到运行时信息则返回 'unknown'
 */
function getPluginId(): string {
  const runtime = getAdapter().getRuntimeSync();
  return runtime?.pluginId ?? 'unknown';
}

/**
 * 窗口 API 命名空间
 */
export const pluginWindow = {
  onShow,
  onHide,
  onFocus,
  onBlur,
  getMode,
  getModeAsync,
  getPluginId,
  // 内部函数，用于测试和高级用例
  _getAdapter: getAdapter,
  _resetAdapter: () => {
    if (adapter) {
      adapter.destroy();
      adapter = null;
    }
  },
};

