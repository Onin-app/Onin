/**
 * 插件运行时环境模块
 *
 * 提供统一的运行时环境检测，通过主应用注入的 __ONIN_RUNTIME__ 对象
 * 来判断插件的运行模式，而不是使用不可靠的 iframe 检测方法。
 *
 * @module core/runtime
 */

/**
 * 插件运行模式
 */
export type PluginMode = 'inline' | 'window';

/**
 * 运行时环境信息接口
 */
export interface OninRuntime {
  /** 插件运行模式：inline（iframe 内联）或 window（独立窗口） */
  mode: PluginMode;
  /** 插件 ID */
  pluginId: string;
  /** 插件版本 */
  version: string;
  /** 主窗口标签 */
  mainWindowLabel: string;
}

// 声明全局变量类型
declare global {
  interface Window {
    __ONIN_RUNTIME__?: OninRuntime;
  }
}

// 缓存的运行时信息
let cachedRuntime: OninRuntime | null = null;

/**
 * 检测是否在 iframe 中运行
 */
function isInIframe(): boolean {
  try {
    return typeof window !== 'undefined' && window.self !== window.top;
  } catch (e) {
    // 跨域 iframe 会抛出异常，说明确实在 iframe 中
    return true;
  }
}

/**
 * 获取运行时环境信息
 *
 * 优先从 window.__ONIN_RUNTIME__ 读取主应用注入的信息，
 * 如果不存在则根据环境自动检测。
 */
export function getRuntime(): OninRuntime {
  // 返回缓存的运行时信息
  if (cachedRuntime) {
    return cachedRuntime;
  }

  // 检查是否有注入的运行时信息
  if (typeof window !== 'undefined' && window.__ONIN_RUNTIME__) {
    console.log(
      '[SDK Runtime] Using injected runtime:',
      window.__ONIN_RUNTIME__,
    );
    cachedRuntime = window.__ONIN_RUNTIME__;
    return cachedRuntime;
  }

  // 开发模式 fallback - 根据环境自动检测
  const inIframe = isInIframe();
  const mode: PluginMode = inIframe ? 'inline' : 'window';

  // 尝试从 URL 或其他来源获取 pluginId
  const pluginId =
    (typeof window !== 'undefined' &&
      ((window as any).__PLUGIN_ID__ ||
        new URLSearchParams(window.location?.search).get('plugin_id'))) ||
    'dev-plugin';

  console.log(
    `[SDK Runtime] No runtime injected, auto-detected mode: ${mode}, inIframe: ${inIframe}`,
  );

  cachedRuntime = {
    mode,
    pluginId,
    version: '0.0.0-dev',
    mainWindowLabel: 'main',
  };

  return cachedRuntime;
}

/**
 * 重置运行时缓存（用于测试）
 * @internal
 */
export function _resetRuntimeCache(): void {
  cachedRuntime = null;
}

/**
 * 当前运行时环境信息
 * 注意：这是延迟计算的 getter，不是立即执行的
 */
export const runtime = {
  get mode(): PluginMode {
    return getRuntime().mode;
  },
  get pluginId(): string {
    return getRuntime().pluginId;
  },
  get version(): string {
    return getRuntime().version;
  },
  get mainWindowLabel(): string {
    return getRuntime().mainWindowLabel;
  },
};

/**
 * 是否为内联模式（iframe）
 */
export function isInlineMode(): boolean {
  return getRuntime().mode === 'inline';
}

/**
 * 是否为窗口模式（独立窗口）
 */
export function isWindowMode(): boolean {
  return getRuntime().mode === 'window';
}

/**
 * 获取当前插件 ID
 */
export function getPluginId(): string {
  return getRuntime().pluginId;
}
