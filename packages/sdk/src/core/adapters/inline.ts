/**
 * 内联模式适配器
 *
 * 处理 iframe 内嵌模式下的窗口事件。
 * 使用 postMessage 接收父窗口的事件通知，
 * 并通过原生浏览器事件作为 fallback。
 *
 * @module core/adapters/inline
 */

import { BaseAdapter } from './base';

/**
 * 内联模式（iframe）窗口适配器
 */
export class InlineAdapter extends BaseAdapter {
  private messageHandler: ((event: MessageEvent) => void) | null = null;
  private visibilityHandler: (() => void) | null = null;
  private focusHandler: (() => void) | null = null;
  private blurHandler: (() => void) | null = null;

  initialize(): void {
    if (typeof window === 'undefined') {
      return;
    }

    // 监听来自父窗口的 postMessage 事件
    this.messageHandler = (event: MessageEvent) => {
      if (event.data?.type !== 'plugin-lifecycle-event') {
        return;
      }

      const { event: eventName } = event.data;

      switch (eventName) {
        case 'show':
          this.executeShowCallbacks().catch(console.error);
          break;
        case 'hide':
          this.executeHideCallbacks().catch(console.error);
          break;
        case 'focus':
          this.executeFocusCallbacks().catch(console.error);
          break;
        case 'blur':
          this.executeBlurCallbacks().catch(console.error);
          break;
      }
    };

    window.addEventListener('message', this.messageHandler);

    // 设置 fallback：使用浏览器原生事件
    this.setupFallbackEvents();
  }

  /**
   * 设置 fallback 事件监听
   * 当 postMessage 不可用时，使用浏览器原生事件
   */
  private setupFallbackEvents(): void {
    // visibilitychange 用于 show/hide
    this.visibilityHandler = () => {
      if (document.hidden) {
        this.executeHideCallbacks().catch(console.error);
      } else {
        this.executeShowCallbacks().catch(console.error);
      }
    };
    document.addEventListener('visibilitychange', this.visibilityHandler);

    // window focus/blur 用于焦点事件
    this.focusHandler = () => {
      this.executeFocusCallbacks().catch(console.error);
    };
    this.blurHandler = () => {
      this.executeBlurCallbacks().catch(console.error);
    };

    window.addEventListener('focus', this.focusHandler);
    window.addEventListener('blur', this.blurHandler);
  }

  destroy(): void {
    if (this.messageHandler) {
      window.removeEventListener('message', this.messageHandler);
      this.messageHandler = null;
    }

    if (this.visibilityHandler) {
      document.removeEventListener('visibilitychange', this.visibilityHandler);
      this.visibilityHandler = null;
    }

    if (this.focusHandler) {
      window.removeEventListener('focus', this.focusHandler);
      this.focusHandler = null;
    }

    if (this.blurHandler) {
      window.removeEventListener('blur', this.blurHandler);
      this.blurHandler = null;
    }

    super.destroy();
  }
}
