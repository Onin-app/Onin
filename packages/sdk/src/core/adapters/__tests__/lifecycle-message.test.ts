import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { RuntimeInfo, EventCallback } from '../base';

function stubWindow(search: string, extra: Record<string, any> = {}) {
  vi.stubGlobal('window', {
    location: { search },
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    ...extra,
  });
}

describe('LifecycleMessageAdapter', () => {
  let LifecycleMessageAdapter: typeof import('../lifecycle-message').LifecycleMessageAdapter;

  beforeEach(async () => {
    vi.resetModules();
    vi.restoreAllMocks();
    vi.clearAllMocks();
    const mod = await import('../lifecycle-message');
    LifecycleMessageAdapter = mod.LifecycleMessageAdapter;
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('constructor', () => {
    it('registers message listener on window', () => {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      expect(window.addEventListener).toHaveBeenCalledWith(
        'message',
        expect.any(Function),
      );
      adapter.destroy();
    });

    it('sets initialized to true', () => {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      expect((adapter as any).initialized).toBe(true);
      adapter.destroy();
    });
  });

  describe('getRuntime', () => {
    it('returns runtime from URL params', async () => {
      stubWindow('?mode=inline&plugin_id=test-plugin');
      const adapter = new LifecycleMessageAdapter();
      const runtime = adapter.getRuntimeSync();
      expect(runtime?.mode).toBe('inline');
      expect(runtime?.pluginId).toBe('test-plugin');
      expect(runtime?.version).toBeDefined();
      expect(runtime?.mainWindowLabel).toBe('main');
      adapter.destroy();
    });

    it('returns runtime via async getRuntime', async () => {
      stubWindow('?mode=window&plugin_id=async-plugin');
      const adapter = new LifecycleMessageAdapter();
      const runtime = await adapter.getRuntime();
      expect(runtime.mode).toBe('window');
      expect(runtime.pluginId).toBe('async-plugin');
      adapter.destroy();
    });

    it('returns null sync when no runtime available', () => {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      expect(adapter.getRuntimeSync()).toBeNull();
      adapter.destroy();
    });
  });

  describe('lifecycle message handling', () => {
    function createAdapter(onShow?: EventCallback, onHide?: EventCallback) {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      if (onShow) adapter.onShow(onShow);
      if (onHide) adapter.onHide(onHide);
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;
      return { adapter, handler };
    }

    it('receives runtime init message', async () => {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      const handler = (window.addEventListener as ReturnType<typeof vi.fn>).mock
        .calls[0][1] as (event: MessageEvent) => void;

      const runtime: RuntimeInfo = {
        mode: 'window',
        pluginId: 'msg-plugin',
        version: '1.0.0',
        mainWindowLabel: 'main',
      };

      handler({
        data: { type: 'plugin-runtime-init', runtime },
      } as MessageEvent);

      const result = await adapter.getRuntime();
      expect(result).toEqual(runtime);
      adapter.destroy();
    });

    it('handles lifecycle show event', () => {
      const onShow = vi.fn();
      const { adapter, handler } = createAdapter(onShow);
      (adapter as any).isVisible = false;

      handler({
        data: { type: 'plugin-lifecycle-event', event: 'show' },
      } as MessageEvent);

      expect(onShow).toHaveBeenCalled();
      adapter.destroy();
    });

    it('handles lifecycle hide event', () => {
      const onHide = vi.fn();
      const { adapter, handler } = createAdapter(undefined, onHide);
      (adapter as any).isVisible = true;

      handler({
        data: { type: 'plugin-lifecycle-event', event: 'hide' },
      } as MessageEvent);

      expect(onHide).toHaveBeenCalled();
      adapter.destroy();
    });

    it('ignores unknown message types', () => {
      const onShow = vi.fn();
      const { adapter, handler } = createAdapter(onShow);
      handler({ data: { type: 'unknown' } } as MessageEvent);
      expect(onShow).not.toHaveBeenCalled();
      adapter.destroy();
    });

    it('ignores non-object messages', () => {
      const onShow = vi.fn();
      const { adapter, handler } = createAdapter(onShow);
      handler({ data: 'string' } as any);
      expect(onShow).not.toHaveBeenCalled();
      adapter.destroy();
    });
  });

  describe('destroy', () => {
    it('removes message listener and clears callbacks', () => {
      stubWindow('');
      const adapter = new LifecycleMessageAdapter();
      const onShow = vi.fn();
      adapter.onShow(onShow);
      adapter.destroy();
      expect(window.removeEventListener).toHaveBeenCalledWith(
        'message',
        expect.any(Function),
      );
      expect((adapter as any).showCallbacks).toHaveLength(0);
    });
  });
});
