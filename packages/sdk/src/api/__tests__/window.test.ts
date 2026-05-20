import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

function stubWindow(search: string, extra: Record<string, any> = {}) {
  vi.stubGlobal('window', {
    location: { search },
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    ...extra,
  });
}

describe('pluginWindow', () => {
  let pluginWindow: import('../window').pluginWindow;

  beforeEach(async () => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.clearAllMocks();
  });

  afterEach(() => {
    if (pluginWindow) {
      pluginWindow._resetAdapter();
    }
    vi.unstubAllGlobals();
  });

  describe('LifecycleMessageAdapter mode (inline)', () => {
    beforeEach(async () => {
      stubWindow('?mode=inline&plugin_id=test-plugin');
      const mod = await import('../window');
      pluginWindow = mod.pluginWindow;
    });

    it('getMode returns inline', () => {
      expect(pluginWindow.getMode()).toBe('inline');
    });

    it('getModeAsync returns inline', async () => {
      const mode = await pluginWindow.getModeAsync();
      expect(mode).toBe('inline');
    });

    it('getPluginId returns correct id', () => {
      expect(pluginWindow.getPluginId()).toBe('test-plugin');
    });

    it('onShow registers callback and fires on message', () => {
      const cb = vi.fn();
      pluginWindow.onShow(cb);
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;
      (pluginWindow as any)._getAdapter().isVisible = false;

      // 触发非生命周期事件，应该被忽略
      handler({
        data: { type: 'unknown-event', event: 'show' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发错误的生命周期事件，应该被忽略
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'hide' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发正确的生命周期事件，应该被调用
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'show' },
      } as MessageEvent);
      expect(cb).toHaveBeenCalled();
    });

    it('onHide registers callback and fires on message', () => {
      const cb = vi.fn();
      pluginWindow.onHide(cb);
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;
      (pluginWindow as any)._getAdapter().isVisible = true;

      // 触发非生命周期事件，应该被忽略
      handler({
        data: { type: 'unknown-event', event: 'hide' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发错误的生命周期事件，应该被忽略
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'show' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发正确的生命周期事件，应该被调用
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'hide' },
      } as MessageEvent);
      expect(cb).toHaveBeenCalled();
    });

    it('onFocus registers callback and fires on message', () => {
      const cb = vi.fn();
      pluginWindow.onFocus(cb);
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;
      (pluginWindow as any)._getAdapter().isFocused = false;

      // 触发非生命周期事件，应该被忽略
      handler({
        data: { type: 'unknown-event', event: 'focus' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发错误的生命周期事件，应该被忽略
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'blur' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发正确的生命周期事件，应该被调用
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'focus' },
      } as MessageEvent);
      expect(cb).toHaveBeenCalled();
    });

    it('onBlur registers callback and fires on message', () => {
      const cb = vi.fn();
      pluginWindow.onBlur(cb);
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;
      (pluginWindow as any)._getAdapter().isFocused = true;

      // 触发非生命周期事件，应该被忽略
      handler({
        data: { type: 'unknown-event', event: 'blur' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发错误的生命周期事件，应该被忽略
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'focus' },
      } as MessageEvent);
      expect(cb).not.toHaveBeenCalled();

      // 触发正确的生命周期事件，应该被调用
      handler({
        data: { type: 'plugin-lifecycle-event', event: 'blur' },
      } as MessageEvent);
      expect(cb).toHaveBeenCalled();
    });
  });

  describe('WindowModeAdapter mode (window)', () => {
    beforeEach(async () => {
      stubWindow('?mode=window&plugin_id=window-plugin', {
        __TAURI__: { event: { listen: vi.fn().mockResolvedValue(vi.fn()) } },
        __ONIN_RUNTIME__: {
          mode: 'window',
          pluginId: 'window-plugin',
          version: '1.0.0',
          mainWindowLabel: 'main',
        },
      });
      const mod = await import('../window');
      pluginWindow = mod.pluginWindow;
    });

    it('getMode returns window', () => {
      expect(pluginWindow.getMode()).toBe('window');
    });

    it('getModeAsync returns window', async () => {
      const mode = await pluginWindow.getModeAsync();
      expect(mode).toBe('window');
    });

    it('getPluginId returns correct id', () => {
      expect(pluginWindow.getPluginId()).toBe('window-plugin');
    });
  });

  describe('no mode specified (fallback)', () => {
    beforeEach(async () => {
      stubWindow('');
      const mod = await import('../window');
      pluginWindow = mod.pluginWindow;
    });

    it('getMode returns unknown', () => {
      expect(pluginWindow.getMode()).toBe('unknown');
    });

    it('getPluginId returns unknown', () => {
      expect(pluginWindow.getPluginId()).toBe('unknown');
    });
  });

  describe('_resetAdapter', () => {
    it('resets the adapter and allows re-creation', async () => {
      stubWindow('?mode=inline&plugin_id=test-plugin');
      const mod = await import('../window');
      const pw = mod.pluginWindow;
      const first = pw._getAdapter();
      pw._resetAdapter();
      const second = pw._getAdapter();
      expect(second).not.toBe(first);
    });
  });
});
