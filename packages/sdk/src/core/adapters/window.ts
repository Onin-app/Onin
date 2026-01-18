/**
 * 窗口模式适配器
 *
 * 处理独立窗口模式下的窗口事件。
 * 使用 Tauri 事件系统接收后端发送的窗口事件。
 *
 * @module core/adapters/window
 */

import { BaseAdapter } from './base';

/**
 * 窗口模式适配器
 * 使用 Tauri 事件系统监听窗口事件
 */
export class WindowModeAdapter extends BaseAdapter {
  private unlistenVisibility: (() => void) | null = null;
  private unlistenFocus: (() => void) | null = null;
  private unlistenBlur: (() => void) | null = null;
  private tauriEventsActive = false;

  initialize(): void {
    if (typeof window === 'undefined') {
      return;
    }

    // 初始化状态：假设窗口刚创建时是聚焦的
    // 但我们不知道实际状态，所以设置为 null 表示未知
    // 这样第一个事件无论是 focus 还是 blur 都会被正确处理
    this.resetStateToUnknown();

    this.tryListenTauriEvents();
  }

  /**
   * 重置状态为未知，这样第一个事件会被正确处理
   */
  private resetStateToUnknown(): void {
    // 使用特殊值表示"未知状态"
    // 这样无论第一个事件是什么，都会被触发
    (this as any)._stateUnknown = true;
  }

  /**
   * 尝试监听 Tauri 事件（带重试机制）
   */
  private tryListenTauriEvents(attempt = 1, maxAttempts = 10): void {
    const tauri = (window as any).__TAURI__;

    if (!tauri?.event?.listen) {
      if (attempt < maxAttempts) {
        setTimeout(
          () => this.tryListenTauriEvents(attempt + 1, maxAttempts),
          attempt * 100,
        );
      } else {
        console.warn(
          '[WindowAdapter] Tauri API not available after max attempts',
        );
      }
      return;
    }

    this.log('Setting up Tauri event listeners');
    this.tauriEventsActive = true;

    // 监听窗口焦点事件 - 这是主要的事件源
    tauri.event
      .listen('window_focus', () => {
        this.log('Received window_focus event');
        this.handleFocusEvent();
      })
      .then((unlisten: () => void) => {
        this.unlistenFocus = unlisten;
      })
      .catch((error: Error) => {
        console.error('[WindowAdapter] Failed to listen window_focus:', error);
      });

    // 监听窗口失焦事件
    tauri.event
      .listen('window_blur', () => {
        this.log('Received window_blur event');
        this.handleBlurEvent();
      })
      .then((unlisten: () => void) => {
        this.unlistenBlur = unlisten;
      })
      .catch((error: Error) => {
        console.error('[WindowAdapter] Failed to listen window_blur:', error);
      });

    // 监听窗口可见性事件 - 用于 show/hide
    tauri.event
      .listen('window_visibility', (event: any) => {
        const isVisible = event.payload;
        this.log(`Received window_visibility event: ${isVisible}`);
        if (isVisible) {
          this.handleShowEvent();
        } else {
          this.handleHideEvent();
        }
      })
      .then((unlisten: () => void) => {
        this.unlistenVisibility = unlisten;
      })
      .catch((error: Error) => {
        console.error(
          '[WindowAdapter] Failed to listen window_visibility:',
          error,
        );
      });
  }

  /**
   * 处理 focus 事件
   */
  private handleFocusEvent(): void {
    // 如果状态未知，先设置为 not focused，这样 focus 会被触发
    if ((this as any)._stateUnknown) {
      this.isFocused = false;
      (this as any)._stateUnknown = false;
    }
    this.executeFocusCallbacks().catch(console.error);
  }

  /**
   * 处理 blur 事件
   */
  private handleBlurEvent(): void {
    // 如果状态未知，先设置为 focused，这样 blur 会被触发
    if ((this as any)._stateUnknown) {
      this.isFocused = true;
      (this as any)._stateUnknown = false;
    }
    this.executeBlurCallbacks().catch(console.error);
  }

  /**
   * 处理 show 事件
   */
  private handleShowEvent(): void {
    if ((this as any)._stateUnknown) {
      this.isVisible = false;
      (this as any)._stateUnknown = false;
    }
    this.executeShowCallbacks().catch(console.error);
  }

  /**
   * 处理 hide 事件
   */
  private handleHideEvent(): void {
    if ((this as any)._stateUnknown) {
      this.isVisible = true;
      (this as any)._stateUnknown = false;
    }
    this.executeHideCallbacks().catch(console.error);
  }

  destroy(): void {
    if (this.unlistenVisibility) {
      this.unlistenVisibility();
      this.unlistenVisibility = null;
    }

    if (this.unlistenFocus) {
      this.unlistenFocus();
      this.unlistenFocus = null;
    }

    if (this.unlistenBlur) {
      this.unlistenBlur();
      this.unlistenBlur = null;
    }

    this.tauriEventsActive = false;
    super.destroy();
  }
}
