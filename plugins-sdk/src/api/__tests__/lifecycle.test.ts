/**
 * Lifecycle API Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

describe('Lifecycle API', () => {
  beforeEach(() => {
    // Reset modules before each test
    vi.resetModules();
  });

  describe('onLoad', () => {
    it('should execute callback automatically', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const mockCallback = vi.fn();
      lifecycle.onLoad(mockCallback);

      // Wait for microtask to execute
      await new Promise<void>(resolve => queueMicrotask(resolve));

      expect(mockCallback).toHaveBeenCalledTimes(1);
    });

    it('should execute multiple callbacks in order', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const executionOrder: number[] = [];
      
      lifecycle.onLoad(async () => {
        executionOrder.push(1);
      });
      
      lifecycle.onLoad(async () => {
        executionOrder.push(2);
      });
      
      lifecycle.onLoad(async () => {
        executionOrder.push(3);
      });

      // Wait for microtasks to execute
      await new Promise(resolve => setTimeout(resolve, 10));

      expect(executionOrder).toEqual([1, 2, 3]);
    });

    it('should handle async callbacks', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      let value = 0;
      
      lifecycle.onLoad(async () => {
        await new Promise(resolve => setTimeout(resolve, 10));
        value = 42;
      });

      // Wait for callback to complete
      await new Promise(resolve => setTimeout(resolve, 50));

      expect(value).toBe(42);
    });

    it('should handle errors in callbacks', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      lifecycle.onLoad(async () => {
        throw new Error('Test error');
      });

      // Wait for callback to execute
      await new Promise(resolve => setTimeout(resolve, 10));

      expect(consoleErrorSpy).toHaveBeenCalled();
      
      consoleErrorSpy.mockRestore();
    });

    it('should not execute callbacks multiple times', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const mockCallback = vi.fn();
      
      lifecycle.onLoad(mockCallback);
      lifecycle.onLoad(mockCallback);

      // Wait for microtasks
      await new Promise(resolve => setTimeout(resolve, 10));

      // Each registration should execute once
      expect(mockCallback).toHaveBeenCalledTimes(2);
    });
  });

  describe('onUnload', () => {
    it('should register unload callback', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const mockCallback = vi.fn();
      
      expect(() => {
        lifecycle.onUnload(mockCallback);
      }).not.toThrow();
    });

    it('should execute unload callbacks', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const mockCallback = vi.fn();
      lifecycle.onUnload(mockCallback);

      // Execute unload callbacks
      await lifecycle._executeUnloadCallbacks();

      expect(mockCallback).toHaveBeenCalledTimes(1);
    });

    it('should handle errors in unload callbacks gracefully', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      lifecycle.onUnload(async () => {
        throw new Error('Unload error');
      });

      // Should not throw
      await expect(lifecycle._executeUnloadCallbacks()).resolves.not.toThrow();
      
      expect(consoleErrorSpy).toHaveBeenCalled();
      consoleErrorSpy.mockRestore();
    });
  });

  describe('Integration', () => {
    it('should work with settings registration', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      let settingsRegistered = false;
      
      lifecycle.onLoad(async () => {
        // Simulate settings registration
        await new Promise(resolve => setTimeout(resolve, 5));
        settingsRegistered = true;
      });

      // Wait for callback
      await new Promise(resolve => setTimeout(resolve, 20));

      expect(settingsRegistered).toBe(true);
    });

    it('should work with command registration', async () => {
      const { lifecycle } = await import('../lifecycle');
      
      const commands: string[] = [];
      
      lifecycle.onLoad(async () => {
        // Simulate command registration
        commands.push('command1');
        commands.push('command2');
      });

      // Wait for callback
      await new Promise(resolve => setTimeout(resolve, 10));

      expect(commands).toEqual(['command1', 'command2']);
    });
  });
});
