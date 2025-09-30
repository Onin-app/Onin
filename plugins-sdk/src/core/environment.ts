/**
 * @module core/environment
 * @description 运行环境检测模块，用于识别当前 SDK 是在 Headless (Deno) 环境还是 Vwebview (html) 环境中运行。
 */

/**
 * 定义支持的运行环境枚举。
 */
export enum RuntimeEnvironment {
  Headless = 'headless',  // 无界面的 Deno 环境
  Webview = 'webview',    // 有界面的 Webview 环境
  Unknown = 'unknown',
}

/**
 * 获取当前的运行环境。
 * 
 * 通过检查全局对象和运行时特征来确定当前代码的运行环境。
 * - 如果 `window.__TAURI_INTERNALS__` 对象存在，则认为是 Webview 环境
 * - 如果存在 `Deno.core`，则认为是 Headless 环境（包括插件运行时）
 * - 否则，认为是未知环境。
 * 
 * @returns {RuntimeEnvironment} 当前的运行环境。
 */
export function getEnvironment(): RuntimeEnvironment {
  // @ts-ignore
  if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
    return RuntimeEnvironment.Webview;
  }

  // @ts-ignore
  if (typeof Deno !== 'undefined' && Deno.core) {
    return RuntimeEnvironment.Headless;
  }

  return RuntimeEnvironment.Unknown;
}