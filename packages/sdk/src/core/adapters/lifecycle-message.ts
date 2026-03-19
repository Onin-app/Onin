/**
 * 生命周期消息适配器
 *
 * 插件窗口生命周期由宿主通过消息转发到页面。
 * - Inline 模式：消息由主应用内联容器转发
 * - Window 模式：消息由独立插件窗口转发
 *
 * 消息格式：
 * - plugin-runtime-init: { type, runtime: { mode, pluginId, ... } }
 * - plugin-lifecycle-event: { type, event: 'show'|'hide'|'focus'|'blur'|'cleanup' }
 */

import { BaseAdapter } from './base';

/**
 * 运行时信息
 */
interface RuntimeInfo {
  mode: 'inline' | 'window';
  pluginId: string;
  version: string;
  mainWindowLabel: string;
}

/**
 * 生命周期消息适配器
 */
export class LifecycleMessageAdapter extends BaseAdapter {
  private messageHandler: ((event: MessageEvent) => void) | null = null;
  private runtimeResolve: ((runtime: RuntimeInfo) => void) | null = null;
  private runtimePromise: Promise<RuntimeInfo>;
  private _runtime: RuntimeInfo | null = null;

  constructor() {
    super();

    // 创建运行时 Promise
    this.runtimePromise = new Promise((resolve) => {
      this.runtimeResolve = resolve;
    });

    // 立即初始化
    this.initializeNow();
  }

  /**
   * 立即初始化
   */
  private initializeNow(): void {
    console.log('[LifecycleMessageAdapter] Initializing...');

    // 尝试从 URL 参数获取模式信息作为 fallback
    this.trySetRuntimeFromUrl();

    // 监听来自宿主窗口的消息
    this.messageHandler = (event: MessageEvent) => {
      if (event.data && typeof event.data === 'object' && event.data.type) {
        console.log(
          '[LifecycleMessageAdapter] Received message:',
          event.data.type,
          event.data,
        );
      }

      const data = event.data;
      if (!data || typeof data !== 'object') return;

      if (data.type === 'plugin-runtime-init') {
        this.handleRuntimeInit(data.runtime);
      } else if (data.type === 'plugin-lifecycle-event') {
        this.handleLifecycleEvent(data.event);
      }
    };

    window.addEventListener('message', this.messageHandler);
    this.initialized = true;
    console.log(
      '[LifecycleMessageAdapter] Initialized, listening for lifecycle messages',
    );
  }

  /**
   * 尝试从 URL 参数设置运行时信息（作为 fallback）
   */
  private trySetRuntimeFromUrl(): void {
    try {
      const params = new URLSearchParams(window.location.search);
      const modeParam = params.get('mode');
      const pluginId = params.get('plugin_id') || 'unknown';

      if (modeParam === 'window' || modeParam === 'inline') {
        console.log(
          '[LifecycleMessageAdapter] Setting runtime from URL params:',
          {
            mode: modeParam,
            pluginId,
          },
        );
        this._runtime = {
          mode: modeParam,
          pluginId,
          version: '0.1.0',
          mainWindowLabel: 'main',
        };

        if (this.runtimeResolve) {
          this.runtimeResolve(this._runtime);
          this.runtimeResolve = null;
        }
      }
    } catch (error) {
      console.warn(
        '[LifecycleMessageAdapter] Failed to read URL params:',
        error,
      );
    }
  }

  /**
   * 获取运行时信息（异步）
   */
  async getRuntime(): Promise<RuntimeInfo> {
    if (this._runtime) {
      return this._runtime;
    }
    return this.runtimePromise;
  }

  /**
   * 获取运行时信息（同步，可能返回 null）
   */
  getRuntimeSync(): RuntimeInfo | null {
    return this._runtime;
  }

  protected initialize(): void {
    // 初始化已在构造函数中完成
  }

  /**
   * 处理运行时初始化
   */
  private handleRuntimeInit(runtime: RuntimeInfo): void {
    console.log('[LifecycleMessageAdapter] Runtime init received:', runtime);
    this._runtime = runtime;

    // 设置初始状态
    (this as any)._stateUnknown = true;

    // Resolve 等待的 Promise
    if (this.runtimeResolve) {
      this.runtimeResolve(runtime);
      this.runtimeResolve = null;
    }
  }

  /**
   * 处理生命周期事件
   */
  private handleLifecycleEvent(
    event: 'show' | 'hide' | 'focus' | 'blur' | 'cleanup',
  ): void {
    console.log('[LifecycleMessageAdapter] Lifecycle event:', event);

    if (event === 'cleanup') {
      return;
    }

    // 如果状态未知，设置正确的初始状态
    if ((this as any)._stateUnknown) {
      (this as any)._stateUnknown = false;
      if (event === 'show') {
        this.isVisible = false;
      } else if (event === 'hide') {
        this.isVisible = true;
      } else if (event === 'focus') {
        this.isFocused = false;
      } else if (event === 'blur') {
        this.isFocused = true;
      }
    }

    switch (event) {
      case 'show':
        this.executeShowCallbacks();
        break;
      case 'hide':
        this.executeHideCallbacks();
        break;
      case 'focus':
        this.executeFocusCallbacks();
        break;
      case 'blur':
        this.executeBlurCallbacks();
        break;
    }
  }

  destroy(): void {
    if (this.messageHandler) {
      window.removeEventListener('message', this.messageHandler);
      this.messageHandler = null;
    }

    this.initialized = false;
    this.showCallbacks = [];
    this.hideCallbacks = [];
    this.focusCallbacks = [];
    this.blurCallbacks = [];
    console.log('[LifecycleMessageAdapter] Destroyed');
  }
}
