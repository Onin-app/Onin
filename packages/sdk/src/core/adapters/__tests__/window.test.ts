import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { EventCallback } from '../base';

function stubWindow(overrides: Record<string, any> = {}) {
  vi.stubGlobal('window', {
    location: { search: '' },
    ...overrides,
  });
}

describe('WindowModeAdapter', () => {
  let WindowModeAdapter: typeof import('../window').WindowModeAdapter;

  beforeEach(async () => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.clearAllMocks();
    const mod = await import('../window');
    WindowModeAdapter = mod.WindowModeAdapter;
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('constructor', () => {
    it('creates runtime promise', () => {
      stubWindow();
      const adapter = new WindowModeAdapter();
      expect(typeof (adapter as any).runtimePromise?.then).toBe('function');
      adapter.destroy();
    });
  });

  describe('getRuntime', () => {
    it('returns runtime from __ONIN_RUNTIME__ global', () => {
      stubWindow({
        __ONIN_RUNTIME__: {
          mode: 'window',
          pluginId: 't-plugin',
          version: '2.0.0',
          mainWindowLabel: 'main',
        },
      });
      const adapter = new WindowModeAdapter();
      const runtime = adapter.getRuntimeSync();
      expect(runtime).toEqual({
        mode: 'window',
        pluginId: 't-plugin',
        version: '2.0.0',
        mainWindowLabel: 'main',
      });
      adapter.destroy();
    });

    it('returns runtime from URL params', () => {
      stubWindow({ location: { search: '?mode=window&plugin_id=url-plugin' } });
      const adapter = new WindowModeAdapter();
      const runtime = adapter.getRuntimeSync();
      expect(runtime?.pluginId).toBe('url-plugin');
      expect(runtime?.mode).toBe('window');
      adapter.destroy();
    });

    it('returns null when no runtime available', () => {
      stubWindow();
      const adapter = new WindowModeAdapter();
      expect(adapter.getRuntimeSync()).toBeNull();
      adapter.destroy();
    });
  });

  describe('Tauri event handling', () => {
    function createAdapter(
      onShow?: EventCallback,
      onFocus?: EventCallback,
      onBlur?: EventCallback,
    ) {
      const listen = vi.fn().mockResolvedValue(vi.fn());
      stubWindow({
        __TAURI__: { event: { listen } },
        __ONIN_RUNTIME__: {
          mode: 'window',
          pluginId: 't-plugin',
          version: '1.0.0',
          mainWindowLabel: 'main',
        },
      });
      const adapter = new WindowModeAdapter();
      if (onShow) adapter.onShow(onShow);
      if (onFocus) adapter.onFocus(onFocus);
      if (onBlur) adapter.onBlur(onBlur);
      return { adapter, listen };
    }

    it('subscribes to Tauri events', () => {
      const { listen, adapter } = createAdapter();
      expect(listen).toHaveBeenCalledWith('window_focus', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('window_blur', expect.any(Function));
      expect(listen).toHaveBeenCalledWith(
        'window_visibility',
        expect.any(Function),
      );
      adapter.destroy();
    });

    it('handles visibility event (show)', () => {
      const onShow = vi.fn();
      const { listen, adapter } = createAdapter(onShow);
      (adapter as any).isVisible = false;

      const visibilityCb = listen.mock.calls.find(
        (c: [string]) => c[0] === 'window_visibility',
      )![1] as (event: { payload: boolean }) => void;
      visibilityCb({ payload: true });

      expect(onShow).toHaveBeenCalled();
      adapter.destroy();
    });

    it('handles visibility event (hide)', () => {
      const onShow = vi.fn();
      const { listen, adapter } = createAdapter(onShow);
      (adapter as any).isVisible = true;

      const visibilityCb = listen.mock.calls.find(
        (c: [string]) => c[0] === 'window_visibility',
      )![1] as (event: { payload: boolean }) => void;
      visibilityCb({ payload: false });

      expect(onShow).not.toHaveBeenCalled();
      adapter.destroy();
    });

    it('handles focus event', () => {
      const onFocus = vi.fn();
      const { listen, adapter } = createAdapter(undefined, onFocus);
      (adapter as any).isFocused = false;

      const focusCb = listen.mock.calls.find(
        (c: [string]) => c[0] === 'window_focus',
      )![1] as () => void;
      focusCb();

      expect(onFocus).toHaveBeenCalled();
      adapter.destroy();
    });

    it('handles blur event', () => {
      const onBlur = vi.fn();
      const { listen, adapter } = createAdapter(undefined, undefined, onBlur);
      (adapter as any).isFocused = true;

      const blurCb = listen.mock.calls.find(
        (c: [string]) => c[0] === 'window_blur',
      )![1] as () => void;
      blurCb();

      expect(onBlur).toHaveBeenCalled();
      adapter.destroy();
    });

    it('retries listening when Tauri API not immediately available', async () => {
      vi.useFakeTimers();
      stubWindow({ __TAURI__: undefined });
      const adapter = new WindowModeAdapter();
      expect((adapter as any).tauriEventsActive).toBe(false);
      vi.advanceTimersByTime(200);
      vi.useRealTimers();
      adapter.destroy();
    });
  });

  describe('destroy', () => {
    it('cleans up Tauri event listeners', async () => {
      const unlistenFocus = vi.fn();
      const unlistenBlur = vi.fn();
      const unlistenVisibility = vi.fn();
      const listen = vi.fn().mockImplementation((event: string) => {
        if (event === 'window_focus') return Promise.resolve(unlistenFocus);
        if (event === 'window_blur') return Promise.resolve(unlistenBlur);
        if (event === 'window_visibility')
          return Promise.resolve(unlistenVisibility);
        return Promise.resolve(vi.fn());
      });
      stubWindow({
        __TAURI__: { event: { listen } },
        __ONIN_RUNTIME__: {
          mode: 'window',
          pluginId: 't-plugin',
          version: '1.0.0',
          mainWindowLabel: 'main',
        },
      });
      const adapter = new WindowModeAdapter();

      // 注册事件以触发初始化监听
      adapter.onShow(vi.fn());
      // 等待 Promise 宏/微任务队列清空，使 listen 返回的 unlisten 能够赋值给 adapter 成员变量
      await new Promise((resolve) => setTimeout(resolve, 0));

      adapter.destroy();
      expect((adapter as any).tauriEventsActive).toBe(false);

      // 验证 unlisten 回调函数被正确执行
      expect(unlistenFocus).toHaveBeenCalled();
      expect(unlistenBlur).toHaveBeenCalled();
      expect(unlistenVisibility).toHaveBeenCalled();
    });
  });
});
