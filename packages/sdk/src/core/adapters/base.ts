/**
 * 窗口适配器基础接口
 *
 * 定义了处理窗口事件的统一接口，具体实现由 InlineAdapter 和 WindowAdapter 提供。
 *
 * @module core/adapters/base
 */

export type EventCallback = () => void | Promise<void>;

/**
 * 窗口适配器接口
 *
 * 定义了窗口事件监听的统一 API，不同运行模式有不同的实现：
 * - InlineAdapter: 使用 postMessage 和 visibilitychange
 * - WindowAdapter: 使用 Tauri 事件系统
 */
export interface WindowAdapter {
  /**
   * 监听窗口显示事件
   */
  onShow(callback: EventCallback): void;

  /**
   * 监听窗口隐藏事件
   */
  onHide(callback: EventCallback): void;

  /**
   * 监听窗口获得焦点事件
   */
  onFocus(callback: EventCallback): void;

  /**
   * 监听窗口失去焦点事件
   */
  onBlur(callback: EventCallback): void;

  /**
   * 销毁适配器（清理事件监听器）
   */
  destroy(): void;
}

/**
 * 获取精确时间戳（带毫秒）
 */
function getTimestamp(): string {
  const now = new Date();
  const h = now.getHours().toString().padStart(2, '0');
  const m = now.getMinutes().toString().padStart(2, '0');
  const s = now.getSeconds().toString().padStart(2, '0');
  const ms = now.getMilliseconds().toString().padStart(3, '0');
  return `${h}:${m}:${s}.${ms}`;
}

/**
 * 适配器基类 - 提供回调管理的通用逻辑
 */
export abstract class BaseAdapter implements WindowAdapter {
  protected showCallbacks: EventCallback[] = [];
  protected hideCallbacks: EventCallback[] = [];
  protected focusCallbacks: EventCallback[] = [];
  protected blurCallbacks: EventCallback[] = [];
  protected initialized = false;

  // 状态跟踪：防止连续触发相同类型的事件
  protected isVisible = true; // 假设初始状态为可见
  protected isFocused = true; // 假设初始状态为聚焦

  // 防抖时间（毫秒）
  protected static readonly DEBOUNCE_MS = 100;
  protected lastShowTime = 0;
  protected lastHideTime = 0;
  protected lastFocusTime = 0;
  protected lastBlurTime = 0;

  /**
   * 获取带时间戳的日志前缀
   */
  protected log(msg: string): void {
    console.log(`[${getTimestamp()}] [Adapter] ${msg}`);
  }

  onShow(callback: EventCallback): void {
    if (!this.showCallbacks.includes(callback)) {
      this.showCallbacks.push(callback);
    }
    this.ensureInitialized();
  }

  onHide(callback: EventCallback): void {
    if (!this.hideCallbacks.includes(callback)) {
      this.hideCallbacks.push(callback);
    }
    this.ensureInitialized();
  }

  onFocus(callback: EventCallback): void {
    if (!this.focusCallbacks.includes(callback)) {
      this.focusCallbacks.push(callback);
    }
    this.ensureInitialized();
  }

  onBlur(callback: EventCallback): void {
    if (!this.blurCallbacks.includes(callback)) {
      this.blurCallbacks.push(callback);
    }
    this.ensureInitialized();
  }

  protected ensureInitialized(): void {
    if (!this.initialized) {
      this.initialize();
      this.initialized = true;
    }
  }

  protected abstract initialize(): void;

  destroy(): void {
    this.showCallbacks = [];
    this.hideCallbacks = [];
    this.focusCallbacks = [];
    this.blurCallbacks = [];
    this.initialized = false;
    this.isVisible = true;
    this.isFocused = true;
  }

  /**
   * 执行显示回调（带防抖和状态检查）
   */
  protected async executeShowCallbacks(): Promise<void> {
    // 状态检查：如果已经是可见状态，不重复触发
    if (this.isVisible) {
      return;
    }

    const now = Date.now();
    if (now - this.lastShowTime < BaseAdapter.DEBOUNCE_MS) {
      return;
    }
    this.lastShowTime = now;
    this.isVisible = true;

    for (const callback of this.showCallbacks) {
      try {
        await callback();
      } catch (error) {
        console.error('[WindowAdapter] Error in onShow callback:', error);
      }
    }
  }

  /**
   * 执行隐藏回调（带防抖和状态检查）
   */
  protected async executeHideCallbacks(): Promise<void> {
    // 状态检查：如果已经是隐藏状态，不重复触发
    if (!this.isVisible) {
      return;
    }

    const now = Date.now();
    if (now - this.lastHideTime < BaseAdapter.DEBOUNCE_MS) {
      return;
    }
    this.lastHideTime = now;
    this.isVisible = false;

    for (const callback of this.hideCallbacks) {
      try {
        await callback();
      } catch (error) {
        console.error('[WindowAdapter] Error in onHide callback:', error);
      }
    }
  }

  protected async executeFocusCallbacks(): Promise<void> {
    this.log(`executeFocusCallbacks - isFocused: ${this.isFocused}`);

    // 状态检查：如果已经是聚焦状态，不重复触发
    if (this.isFocused) {
      this.log('Skipping focus - already focused');
      return;
    }

    const now = Date.now();
    if (now - this.lastFocusTime < BaseAdapter.DEBOUNCE_MS) {
      return;
    }
    this.lastFocusTime = now;
    this.isFocused = true;

    for (const callback of this.focusCallbacks) {
      try {
        await callback();
      } catch (error) {
        console.error('[WindowAdapter] Error in onFocus callback:', error);
      }
    }
  }

  protected async executeBlurCallbacks(): Promise<void> {
    this.log(`executeBlurCallbacks - isFocused: ${this.isFocused}`);

    // 状态检查：如果已经是失焦状态，不重复触发
    if (!this.isFocused) {
      this.log('Skipping blur - already blurred');
      return;
    }

    const now = Date.now();
    if (now - this.lastBlurTime < BaseAdapter.DEBOUNCE_MS) {
      return;
    }
    this.lastBlurTime = now;
    this.isFocused = false;

    for (const callback of this.blurCallbacks) {
      try {
        await callback();
      } catch (error) {
        console.error('[WindowAdapter] Error in onBlur callback:', error);
      }
    }
  }
}
