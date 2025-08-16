import { describe, it, expect, vi, beforeEach } from 'vitest';
import { HelloWorldPlugin } from './index';
import type { PluginContext } from '@baize/plugin-sdk';

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

describe('HelloWorldPlugin', () => {
  let plugin: HelloWorldPlugin;
  let mockContext: PluginContext;

  beforeEach(() => {
    plugin = new HelloWorldPlugin();
    mockContext = createMockContext();
    vi.clearAllMocks();
  });

  describe('activation', () => {
    it('should activate successfully', async () => {
      await plugin.activate(mockContext);
      
      expect(plugin.isPluginActive()).toBe(true);
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello World Plugin activated! 🎉'
      );
      expect(mockContext.storage.set).toHaveBeenCalledWith(
        'lastActivated',
        expect.any(String)
      );
    });

    it('should set up event listeners on activation', async () => {
      await plugin.activate(mockContext);
      
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
      
      await expect(plugin.activate(mockContext)).rejects.toThrow('Notification failed');
    });
  });

  describe('deactivation', () => {
    beforeEach(async () => {
      await plugin.activate(mockContext);
      vi.clearAllMocks();
    });

    it('should deactivate successfully', async () => {
      await plugin.deactivate();
      
      expect(plugin.isPluginActive()).toBe(false);
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello World Plugin deactivated. Goodbye! 👋'
      );
      expect(mockContext.storage.set).toHaveBeenCalledWith(
        'lastDeactivated',
        expect.any(String)
      );
    });

    it('should clean up event listeners on deactivation', async () => {
      await plugin.deactivate();
      
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
      await plugin.activate(mockContext);
    });

    it('should handle app ready event', async () => {
      const onAppReady = (mockContext.events.on as any).mock.calls
        .find(call => call[0] === 'app:ready')[1];
      
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
        .find(call => call[0] === 'hello:greet')[1];
      
      const testData = { message: 'Test greeting!' };
      await onGreetEvent(testData);
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith('Test greeting!');
    });

    it('should handle greet event with default message', async () => {
      const onGreetEvent = (mockContext.events.on as any).mock.calls
        .find(call => call[0] === 'hello:greet')[1];
      
      await onGreetEvent();
      
      expect(mockContext.app.showNotification).toHaveBeenCalledWith(
        'Hello from the Hello World Plugin!'
      );
    });
  });

  describe('storage operations', () => {
    beforeEach(async () => {
      await plugin.activate(mockContext);
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

      const result = await plugin.getStoredData();
      
      expect(result).toEqual(mockData);
      expect(mockContext.storage.get).toHaveBeenCalledTimes(3);
    });

    it('should handle storage errors gracefully', async () => {
      mockContext.storage.get = vi.fn().mockRejectedValue(new Error('Storage error'));
      
      const result = await plugin.getStoredData();
      
      expect(result).toBeNull();
    });
  });

  describe('inactive plugin', () => {
    it('should return null for stored data when inactive', async () => {
      const result = await plugin.getStoredData();
      expect(result).toBeNull();
    });

    it('should start as inactive', () => {
      expect(plugin.isPluginActive()).toBe(false);
    });
  });
});