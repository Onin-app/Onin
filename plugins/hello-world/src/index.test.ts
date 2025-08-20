import { describe, it, expect, vi, beforeEach } from 'vitest';
import helloWorldPlugin, { getStoredData } from './index';
import type { PluginContext, Plugin } from '@baize/plugin-sdk';
import { contextManager, injectPluginContext } from '@baize/plugin-sdk';

// Mock the plugin context
const createMockContext = (): PluginContext => ({
  app: {
    showNotification: vi.fn().mockResolvedValue(undefined),
    getAppVersion: vi.fn().mockResolvedValue('0.1.0'),
    openDialog: vi.fn().mockResolvedValue(null)
  },
  events: {
    on: vi.fn(),
    off: vi.fn(),
    emit: vi.fn()
  },
  storage: {
    get: vi.fn().mockResolvedValue(undefined),
    set: vi.fn().mockResolvedValue(undefined),
    remove: vi.fn().mockResolvedValue(undefined),
    clear: vi.fn().mockResolvedValue(undefined),
    keys: vi.fn().mockResolvedValue([])
  }
});

describe('HelloWorldPlugin (Functional)', () => {
  let plugin: Plugin;
  let mockContext: PluginContext;

  beforeEach(() => {
    mockContext = createMockContext();
    // Inject context into the plugin for testing
    plugin = injectPluginContext(helloWorldPlugin, mockContext);
    vi.clearAllMocks();
  });

  describe('activation', () => {
    it('should activate successfully', async () => {
      // Set context for hooks to work
      contextManager.setContext(mockContext);
      
      await plugin.onActivate!();
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello World Plugin activated! 🎉'
      );
      expect(mockContext.storage.set).toHaveBeenCalledWith(
        'lastActivated',
        expect.any(String)
      );
    });

    it('should set up event listeners on activation', async () => {
      contextManager.setContext(mockContext);
      
      await plugin.onActivate!();
      
      expect(mockContext.events.on).toHaveBeenCalledWith(
        'app:ready',
        expect.any(Function)
      );
      expect(mockContext.events.on).toHaveBeenCalledWith(
        'app:shutdown',
        expect.any(Function)
      );
      expect(mockContext.events.on).toHaveBeenCalledWith(
        'hello:greet',
        expect.any(Function)
      );
    });

    it('should handle activation errors', async () => {
      const error = new Error('Notification failed');
      mockContext.app.showNotification = vi.fn().mockRejectedValue(error);
      contextManager.setContext(mockContext);
      
      await expect(plugin.onActivate!()).rejects.toThrow('Notification failed');
    });
  });

  describe('deactivation', () => {
    beforeEach(async () => {
      contextManager.setContext(mockContext);
      await plugin.onActivate!();
      vi.clearAllMocks();
    });

    it('should deactivate successfully', async () => {
      contextManager.setContext(mockContext);
      
      await plugin.onDeactivate!();
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello World Plugin deactivated. Goodbye! 👋'
      );
      expect(mockContext.storage.set).toHaveBeenCalledWith(
        'lastDeactivated',
        expect.any(String)
      );
    });

    it('should clean up event listeners on deactivation', async () => {
      contextManager.setContext(mockContext);
      
      await plugin.onDeactivate!();
      
      expect(mockContext.events.off).toHaveBeenCalledWith(
        'app:ready',
        expect.any(Function)
      );
      expect(mockContext.events.off).toHaveBeenCalledWith(
        'app:shutdown',
        expect.any(Function)
      );
      expect(mockContext.events.off).toHaveBeenCalledWith(
        'hello:greet',
        expect.any(Function)
      );
    });
  });

  describe('event handling', () => {
    beforeEach(async () => {
      contextManager.setContext(mockContext);
      await plugin.onActivate!();
    });

    it('should handle app ready event', async () => {
      const onAppReady = (mockContext.events.on as any).mock.calls
        .find((call: any) => call[0] === 'app:ready')[1];
      
      contextManager.setContext(mockContext);
      await onAppReady();
      
      expect(mockContext.app.getAppVersion).toHaveBeenCalled();
      expect(mockContext.events.emit).toHaveBeenCalledWith(
        'hello:plugin-ready',
        expect.objectContaining({
          pluginName: 'hello-world',
          timestamp: expect.any(String)
        })
      );
    });

    it('should handle greet event', async () => {
      const onGreetEvent = (mockContext.events.on as any).mock.calls
        .find((call: any) => call[0] === 'hello:greet')[1];
      
      contextManager.setContext(mockContext);
      const testData = { message: 'Test greeting!' };
      await onGreetEvent(testData);
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith('Test greeting!');
    });

    it('should handle greet event with default message', async () => {
      const onGreetEvent = (mockContext.events.on as any).mock.calls
        .find((call: any) => call[0] === 'hello:greet')[1];
      
      contextManager.setContext(mockContext);
      await onGreetEvent();
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello from the Hello World Plugin!'
      );
    });
  });

  describe('storage operations', () => {
    beforeEach(async () => {
      contextManager.setContext(mockContext);
      await plugin.onActivate!();
    });

    it('should retrieve stored data', async () => {
      const mockData = {
        lastActivated: '2023-01-01T00:00:00.000Z',
        lastDeactivated: '2023-01-02T00:00:00.000Z',
        lastShutdown: '2023-01-03T00:00:00.000Z'
      };

      mockContext.storage.get = vi.fn()
        .mockResolvedValueOnce(mockData.lastActivated)
        .mockResolvedValueOnce(mockData.lastDeactivated)
        .mockResolvedValueOnce(mockData.lastShutdown);

      contextManager.setContext(mockContext);
      const result = await getStoredData();
      
      expect(result).toEqual(mockData);
      expect(mockContext.storage.get).toHaveBeenCalledTimes(3);
    });

    it('should handle storage errors gracefully', async () => {
      mockContext.storage.get = vi.fn().mockRejectedValue(new Error('Storage error'));
      
      contextManager.setContext(mockContext);
      const result = await getStoredData();
      
      expect(result).toBeNull();
    });
  });

  describe('plugin metadata', () => {
    it('should have correct metadata', () => {
      expect(plugin.meta).toEqual({
        name: 'Hello World Plugin',
        version: '1.0.0',
        description: 'A simple example plugin demonstrating functional plugin development',
        author: 'Baize Team'
      });
    });

    it('should have activation and deactivation functions', () => {
      expect(typeof plugin.onActivate).toBe('function');
      expect(typeof plugin.onDeactivate).toBe('function');
    });
  });
});