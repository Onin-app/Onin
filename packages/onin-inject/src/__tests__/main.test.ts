import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const mockShow = vi.fn();

// 模拟 Svelte 和 Toast 组件
vi.mock('svelte', () => ({
  mount: vi.fn((component, options) => {
    return {
      show: mockShow,
    };
  }),
}));

vi.mock('../components/Toast.svelte', () => ({
  default: {},
}));

describe('Onin Plugin Runtime Injection Layer (main.ts)', () => {
  let consoleErrorSpy: any;
  let consoleWarnSpy: any;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.resetModules();

    // 模拟控制台，避免污染测试输出
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // 清理全局变量和 DOM
    delete (window as any).__ONIN_TOAST_INJECTED__;
    delete (window as any).__ONIN_SHOW_TOAST__;
    delete (window as any).__ONIN_BRIDGE__;
    delete (window as any).__PLUGIN_ID__;
    delete (window as any).__ONIN_RUNTIME__;

    // 重设 window.location
    Object.defineProperty(window, 'location', {
      value: {
        search: '',
      },
      writable: true,
      configurable: true,
    });

    // 默认文档就绪状态为 complete
    Object.defineProperty(document, 'readyState', {
      value: 'complete',
      writable: true,
      configurable: true,
    });

    document.body.innerHTML = '';
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
    consoleWarnSpy.mockRestore();
  });

  it('should not inject twice if __ONIN_TOAST_INJECTED__ is already true', async () => {
    (window as any).__ONIN_TOAST_INJECTED__ = true;

    await import('../main');

    expect((window as any).__ONIN_SHOW_TOAST__).toBeUndefined();
    expect((window as any).__ONIN_BRIDGE__).toBeUndefined();
  });

  it('should inject APIs and set __ONIN_TOAST_INJECTED__ to true on load', async () => {
    await import('../main');

    expect((window as any).__ONIN_TOAST_INJECTED__).toBe(true);
    expect(typeof (window as any).__ONIN_SHOW_TOAST__).toBe('function');
    expect((window as any).__ONIN_BRIDGE__).toBeDefined();
    expect((window as any).__ONIN_BRIDGE__.version).toBe('0.1.0');
  });

  describe('resolvePluginId', () => {
    it('should parse plugin_id from URL query params', async () => {
      Object.defineProperty(window, 'location', {
        value: { search: '?plugin_id=url-plugin-123' },
        writable: true,
        configurable: true,
      });

      await import('../main');

      expect((window as any).__PLUGIN_ID__).toBe('url-plugin-123');
      expect((globalThis as any).__PLUGIN_ID__).toBe('url-plugin-123');
    });

    it('should fallback to existing __PLUGIN_ID__ if not in URL', async () => {
      (window as any).__PLUGIN_ID__ = 'existing-plugin';

      await import('../main');

      expect((window as any).__PLUGIN_ID__).toBe('existing-plugin');
    });

    it('should fallback to unknown if no plugin id is provided anywhere', async () => {
      await import('../main');

      expect((window as any).__PLUGIN_ID__).toBe('unknown');
    });
  });

  describe('initRuntime', () => {
    it('should set runtime mode from URL params', async () => {
      Object.defineProperty(window, 'location', {
        value: { search: '?mode=window&plugin_id=test' },
        writable: true,
        configurable: true,
      });

      await import('../main');

      expect((window as any).__ONIN_RUNTIME__).toEqual({
        mode: 'window',
        pluginId: 'test',
        version: 'dev-fallback',
        mainWindowLabel: 'main',
      });
    });

    it('should fallback to default inline mode if mode parameter is missing', async () => {
      await import('../main');

      expect((window as any).__ONIN_RUNTIME__).toEqual({
        mode: 'inline',
        pluginId: 'unknown',
        version: 'dev-fallback',
        mainWindowLabel: 'main',
      });
    });

    it('should reuse existing __ONIN_RUNTIME__ metadata if present', async () => {
      const existingRuntime = {
        mode: 'window',
        pluginId: 'custom',
        version: '1.0',
        mainWindowLabel: 'custom-main',
      };
      (window as any).__ONIN_RUNTIME__ = existingRuntime;

      await import('../main');

      expect((window as any).__ONIN_RUNTIME__).toBe(existingRuntime);
    });
  });

  describe('Bridge API and Toast Operations', () => {
    it('should call toast show method when showToast/__ONIN_SHOW_TOAST__ is executed', async () => {
      await import('../main');

      const payload = {
        message: 'Hello Toast',
        kind: 'success',
        duration: 3000,
      };

      // 调用全局 Toast 接口
      (window as any).__ONIN_SHOW_TOAST__(payload);

      expect(mockShow).toHaveBeenCalledWith(payload);

      // 验证 DOM 容器被正确创建挂载
      const container = document.getElementById('onin-inject-root');
      expect(container).toBeDefined();
    });

    it('should bridge postMessage calls to window.postMessage', async () => {
      await import('../main');

      const postMessageSpy = vi.spyOn(window, 'postMessage');

      const msg = { type: 'custom-event', payload: 'data' };
      (window as any).__ONIN_BRIDGE__.postMessage(msg);

      expect(postMessageSpy).toHaveBeenCalledWith(msg, '*');
      postMessageSpy.mockRestore();
    });
  });

  describe('Lifecycle Events', () => {
    it('should dispatch lifecycle events to registered listeners', async () => {
      await import('../main');

      const showSpy1 = vi.fn();
      const showSpy2 = vi.fn();
      const hideSpy = vi.fn();
      const focusSpy = vi.fn();
      const blurSpy = vi.fn();

      const bridge = (window as any).__ONIN_BRIDGE__;
      bridge.onShow(showSpy1);
      bridge.onShow(showSpy2);
      bridge.onHide(hideSpy);
      bridge.onFocus(focusSpy);
      bridge.onBlur(blurSpy);

      // 触发 show 生命周期事件
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'show' },
        }),
      );

      expect(showSpy1).toHaveBeenCalledTimes(1);
      expect(showSpy2).toHaveBeenCalledTimes(1);
      expect(hideSpy).not.toHaveBeenCalled();

      // 触发 hide
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'hide' },
        }),
      );
      expect(hideSpy).toHaveBeenCalledTimes(1);

      // 触发 focus & blur
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'focus' },
        }),
      );
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'blur' },
        }),
      );
      expect(focusSpy).toHaveBeenCalledTimes(1);
      expect(blurSpy).toHaveBeenCalledTimes(1);
    });

    it('should handle listener callback errors gracefully without crashing', async () => {
      await import('../main');

      const badSpy = vi.fn().mockImplementation(() => {
        throw new Error('Callback crash');
      });
      const goodSpy = vi.fn();

      const bridge = (window as any).__ONIN_BRIDGE__;
      bridge.onShow(badSpy);
      bridge.onShow(goodSpy);

      // 触发生命周期事件，虽然 badSpy 崩溃，但 goodSpy 应依然执行，且程序整体不崩溃
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'show' },
        }),
      );

      expect(badSpy).toHaveBeenCalledTimes(1);
      expect(goodSpy).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining('[Onin SDK] Error in show listener:'),
        expect.any(Error),
      );
    });

    it('should ignore message events that are not plugin-lifecycle-events', async () => {
      await import('../main');

      const showSpy = vi.fn();
      (window as any).__ONIN_BRIDGE__.onShow(showSpy);

      // 触发其它类型消息，不应触发回调
      window.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'other-event', event: 'show' },
        }),
      );

      expect(showSpy).not.toHaveBeenCalled();
    });
  });

  describe('DOM Mounting states', () => {
    it('should wait for DOMContentLoaded if state is loading', async () => {
      // 模拟 loading 状态
      Object.defineProperty(document, 'readyState', {
        value: 'loading',
        writable: true,
        configurable: true,
      });

      await import('../main');

      // 此时因为 DOM loading，且还没触发 DOMContentLoaded，不应有容器挂载
      let container = document.getElementById('onin-inject-root');
      expect(container).toBeNull();

      // 触发 DOMContentLoaded
      document.dispatchEvent(new Event('DOMContentLoaded'));

      // 此时应该成功挂载了
      container = document.getElementById('onin-inject-root');
      expect(container).not.toBeNull();
    });
  });
});
