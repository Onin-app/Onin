import { beforeEach, describe, expect, it, vi } from 'vitest';

class MockWindow extends EventTarget {
  location = { search: '' };
}

describe('plugin helpers', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.unstubAllGlobals();
  });

  describe('setupPlugin', () => {
    it('registers setup through lifecycle.onLoad', async () => {
      const { setupPlugin } = await import('./plugin');

      const setup = vi.fn();
      setupPlugin({ setup }, { ready: true });

      await new Promise<void>((resolve) => queueMicrotask(resolve));

      expect(setup).toHaveBeenCalledTimes(1);
      expect(setup).toHaveBeenCalledWith({ ready: true });
    });
  });

  describe('mountPlugin', () => {
    it('throws when mount is missing', async () => {
      const { mountPlugin } = await import('./plugin');

      await expect(
        mountPlugin({}, {} as HTMLElement),
      ).rejects.toThrowError('Plugin mount is not defined.');
    });

    it('returns undefined when mount has no cleanup', async () => {
      const { mountPlugin } = await import('./plugin');

      const target = {} as HTMLElement;
      const mount = vi.fn(async () => undefined);

      const cleanup = await mountPlugin({ mount }, target);

      expect(cleanup).toBeUndefined();
      expect(mount).toHaveBeenCalledWith({ target });
    });

    it('runs cleanup on plugin cleanup runtime event', async () => {
      const mockWindow = new MockWindow();
      vi.stubGlobal('window', mockWindow);

      const { mountPlugin } = await import('./plugin');

      const cleanup = vi.fn();
      const target = {} as HTMLElement;
      const returnedCleanup = await mountPlugin(
        {
          mount: () => cleanup,
        },
        target,
      );

      mockWindow.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'cleanup' },
        }),
      );

      await Promise.resolve();

      expect(returnedCleanup).toBeTypeOf('function');
      expect(cleanup).toHaveBeenCalledTimes(1);
    });

    it('does not run cleanup on hide runtime event', async () => {
      const mockWindow = new MockWindow();
      vi.stubGlobal('window', mockWindow);

      const { mountPlugin } = await import('./plugin');

      const cleanup = vi.fn();
      await mountPlugin(
        {
          mount: () => cleanup,
        },
        {} as HTMLElement,
      );

      mockWindow.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'hide' },
        }),
      );

      await Promise.resolve();

      expect(cleanup).not.toHaveBeenCalled();
    });

    it('runs cleanup on pagehide and beforeunload only once', async () => {
      const mockWindow = new MockWindow();
      vi.stubGlobal('window', mockWindow);

      const { mountPlugin } = await import('./plugin');

      const cleanup = vi.fn();
      await mountPlugin(
        {
          mount: () => cleanup,
        },
        {} as HTMLElement,
      );

      mockWindow.dispatchEvent(new Event('pagehide'));
      mockWindow.dispatchEvent(new Event('beforeunload'));
      mockWindow.dispatchEvent(
        new MessageEvent('message', {
          data: { type: 'plugin-lifecycle-event', event: 'cleanup' },
        }),
      );

      await Promise.resolve();

      expect(cleanup).toHaveBeenCalledTimes(1);
    });

    it('returned cleanup is idempotent', async () => {
      const mockWindow = new MockWindow();
      vi.stubGlobal('window', mockWindow);

      const { mountPlugin } = await import('./plugin');

      const cleanup = vi.fn();
      const returnedCleanup = await mountPlugin(
        {
          mount: () => cleanup,
        },
        {} as HTMLElement,
      );

      await returnedCleanup?.();
      await returnedCleanup?.();

      expect(cleanup).toHaveBeenCalledTimes(1);
    });
  });
});
