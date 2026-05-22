import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { BaseAdapter } from '../base';
import type { RuntimeInfo, EventCallback } from '../base';

class TestAdapter extends BaseAdapter {
  runtime: RuntimeInfo | null = null;
  initializeCalled = false;

  constructor(runtime?: RuntimeInfo) {
    super();
    this.runtime = runtime ?? null;
  }

  getRuntimeSync(): RuntimeInfo | null {
    return this.runtime;
  }

  async getRuntime(): Promise<RuntimeInfo> {
    if (this.runtime) return this.runtime;
    throw new Error('no runtime');
  }

  protected initialize(): void {
    this.initializeCalled = true;
  }

  triggerShow() {
    return this.executeShowCallbacks();
  }
  triggerHide() {
    return this.executeHideCallbacks();
  }
  triggerFocus() {
    return this.executeFocusCallbacks();
  }
  triggerBlur() {
    return this.executeBlurCallbacks();
  }
}

describe('BaseAdapter', () => {
  let adapter: TestAdapter;

  beforeEach(() => {
    vi.useFakeTimers();
    adapter = new TestAdapter();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('onShow / onHide', () => {
    it('registers and executes show callbacks', async () => {
      const cb = vi.fn();
      adapter.onShow(cb);
      adapter.isVisible = false;
      await adapter.triggerShow();
      expect(cb).toHaveBeenCalled();
    });

    it('skips show when already visible', async () => {
      const cb = vi.fn();
      adapter.onShow(cb);
      adapter.isVisible = true;
      await adapter.triggerShow();
      expect(cb).not.toHaveBeenCalled();
    });

    it('registers and executes hide callbacks', async () => {
      const cb = vi.fn();
      adapter.onHide(cb);
      adapter.isVisible = true;
      await adapter.triggerHide();
      expect(cb).toHaveBeenCalled();
    });

    it('skips hide when already hidden', async () => {
      const cb = vi.fn();
      adapter.onHide(cb);
      adapter.isVisible = false;
      await adapter.triggerHide();
      expect(cb).not.toHaveBeenCalled();
    });

    it('deduplicates same callback registration', () => {
      const cb = vi.fn();
      adapter.onShow(cb);
      adapter.onShow(cb);
      expect(adapter['showCallbacks']).toHaveLength(1);
    });

    it('debounces rapid show calls', async () => {
      const cb = vi.fn();
      adapter.onShow(cb);
      adapter.isVisible = false;
      await adapter.triggerShow();
      await adapter.triggerShow();
      expect(cb).toHaveBeenCalledTimes(1);
    });

    it('allows show after debounce window', async () => {
      const cb = vi.fn();
      adapter.onShow(cb);
      adapter.isVisible = false;
      await adapter.triggerShow();
      adapter.isVisible = false;
      vi.advanceTimersByTime(150);
      await adapter.triggerShow();
      expect(cb).toHaveBeenCalledTimes(2);
    });
  });

  describe('onFocus / onBlur', () => {
    it('registers and executes focus callbacks', async () => {
      const cb = vi.fn();
      adapter.onFocus(cb);
      adapter.isFocused = false;
      await adapter.triggerFocus();
      expect(cb).toHaveBeenCalled();
    });

    it('skips focus when already focused', async () => {
      const cb = vi.fn();
      adapter.onFocus(cb);
      adapter.isFocused = true;
      await adapter.triggerFocus();
      expect(cb).not.toHaveBeenCalled();
    });

    it('registers and executes blur callbacks', async () => {
      const cb = vi.fn();
      adapter.onBlur(cb);
      adapter.isFocused = true;
      await adapter.triggerBlur();
      expect(cb).toHaveBeenCalled();
    });

    it('skips blur when already blurred', async () => {
      const cb = vi.fn();
      adapter.onBlur(cb);
      adapter.isFocused = false;
      await adapter.triggerBlur();
      expect(cb).not.toHaveBeenCalled();
    });

    it('handles callback errors gracefully', async () => {
      const cb = vi.fn().mockRejectedValue(new Error('fail'));
      adapter.onFocus(cb);
      adapter.isFocused = false;
      await expect(adapter.triggerFocus()).resolves.not.toThrow();
      expect(cb).toHaveBeenCalled();
    });
  });

  describe('destroy', () => {
    it('clears all callbacks and resets state', () => {
      adapter.onShow(vi.fn());
      adapter.onHide(vi.fn());
      adapter.destroy();
      expect(adapter['showCallbacks']).toHaveLength(0);
      expect(adapter['hideCallbacks']).toHaveLength(0);
      expect(adapter['focusCallbacks']).toHaveLength(0);
      expect(adapter['blurCallbacks']).toHaveLength(0);
      expect(adapter['initialized']).toBe(false);
      expect(adapter['isVisible']).toBe(true);
      expect(adapter['isFocused']).toBe(true);
    });
  });

  describe('ensureInitialized', () => {
    it('calls initialize on first callback registration', () => {
      expect(adapter.initializeCalled).toBe(false);
      adapter.onShow(vi.fn());
      expect(adapter.initializeCalled).toBe(true);
    });

    it('does not call initialize twice', () => {
      adapter.onShow(vi.fn());
      adapter.onHide(vi.fn());
      expect(adapter.initializeCalled).toBe(true);
    });
  });

  describe('getRuntime', () => {
    it('returns runtime info sync', () => {
      const info: RuntimeInfo = {
        mode: 'inline',
        pluginId: 'test',
        version: '1.0.0',
        mainWindowLabel: 'main',
      };
      adapter.runtime = info;
      expect(adapter.getRuntimeSync()).toEqual(info);
    });

    it('returns null when no runtime available', () => {
      expect(adapter.getRuntimeSync()).toBeNull();
    });
  });
});
