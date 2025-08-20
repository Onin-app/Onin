/**
 * Unit tests for Hook-style API functions
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  useApp,
  useEvents,
  useStorage,
  useContext,
  onActivate,
  onDeactivate,
  canUseHooks,
  safeHookCall,
  useOptionalContext,
  OptionalHooks,
  LifecycleUtils,
  HookComposers,
  lifecycleRegistry
} from './hooks';
import { contextManager } from './context';
import { PluginError, PluginErrorCode, PluginContext, AppAPI, EventAPI, StorageAPI } from './types';

// Mock implementations
const mockApp: AppAPI = {
  showNotification: vi.fn().mockResolvedValue(undefined),
  getAppVersion: vi.fn().mockResolvedValue('1.0.0'),
  openDialog: vi.fn().mockResolvedValue('ok')
};

const mockEvents: EventAPI = {
  on: vi.fn(),
  emit: vi.fn(),
  off: vi.fn()
};

const mockStorage: StorageAPI = {
  get: vi.fn().mockResolvedValue(null),
  set: vi.fn().mockResolvedValue(undefined),
  remove: vi.fn().mockResolvedValue(undefined),
  clear: vi.fn().mockResolvedValue(undefined),
  keys: vi.fn().mockResolvedValue([])
};

const mockContext: PluginContext = {
  app: mockApp,
  events: mockEvents,
  storage: mockStorage
};

describe('Hook API Functions', () => {
  beforeEach(() => {
    // Clear any existing context and lifecycle handlers
    contextManager.clearContext();
    LifecycleUtils.clearLifecycleHandlers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    contextManager.clearContext();
    LifecycleUtils.clearLifecycleHandlers();
  });

  describe('useApp', () => {
    it('should return app API when context is available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const app = useApp();
      
      expect(app).toBe(mockApp);
    });

    it('should throw PluginError when context is not available', () => {
      expect(() => useApp()).toThrow(PluginError);
      expect(() => useApp()).toThrow('useApp() can only be called within a plugin lifecycle function');
    });

    it('should throw error with correct error code', () => {
      try {
        useApp();
      } catch (error) {
        expect(error).toBeInstanceOf(PluginError);
        expect((error as PluginError).code).toBe(PluginErrorCode.HOOK_ERROR);
      }
    });
  });

  describe('useEvents', () => {
    it('should return events API when context is available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const events = useEvents();
      
      expect(events).toBe(mockEvents);
    });

    it('should throw PluginError when context is not available', () => {
      expect(() => useEvents()).toThrow(PluginError);
      expect(() => useEvents()).toThrow('useEvents() can only be called within a plugin lifecycle function');
    });

    it('should throw error with correct error code', () => {
      try {
        useEvents();
      } catch (error) {
        expect(error).toBeInstanceOf(PluginError);
        expect((error as PluginError).code).toBe(PluginErrorCode.HOOK_ERROR);
      }
    });
  });

  describe('useStorage', () => {
    it('should return storage API when context is available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const storage = useStorage();
      
      expect(storage).toBe(mockStorage);
    });

    it('should throw PluginError when context is not available', () => {
      expect(() => useStorage()).toThrow(PluginError);
      expect(() => useStorage()).toThrow('useStorage() can only be called within a plugin lifecycle function');
    });

    it('should throw error with correct error code', () => {
      try {
        useStorage();
      } catch (error) {
        expect(error).toBeInstanceOf(PluginError);
        expect((error as PluginError).code).toBe(PluginErrorCode.HOOK_ERROR);
      }
    });
  });

  describe('useContext', () => {
    it('should return complete context when available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const context = useContext();
      
      expect(context).toBe(mockContext);
      expect(context.app).toBe(mockApp);
      expect(context.events).toBe(mockEvents);
      expect(context.storage).toBe(mockStorage);
    });

    it('should throw PluginError when context is not available', () => {
      expect(() => useContext()).toThrow(PluginError);
      expect(() => useContext()).toThrow('useContext() can only be called within a plugin lifecycle function');
    });
  });

  describe('canUseHooks', () => {
    it('should return true when context is available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      expect(canUseHooks()).toBe(true);
    });

    it('should return false when context is not available', () => {
      expect(canUseHooks()).toBe(false);
    });
  });

  describe('useOptionalContext', () => {
    it('should return context when available', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const context = useOptionalContext();
      
      expect(context).toBe(mockContext);
    });

    it('should return null when context is not available', () => {
      const context = useOptionalContext();
      
      expect(context).toBeNull();
    });
  });

  describe('OptionalHooks', () => {
    describe('useOptionalApp', () => {
      it('should return app API when context is available', () => {
        contextManager.setContext(mockContext, 'test-plugin');
        
        const app = OptionalHooks.useOptionalApp();
        
        expect(app).toBe(mockApp);
      });

      it('should return null when context is not available', () => {
        const app = OptionalHooks.useOptionalApp();
        
        expect(app).toBeNull();
      });
    });

    describe('useOptionalEvents', () => {
      it('should return events API when context is available', () => {
        contextManager.setContext(mockContext, 'test-plugin');
        
        const events = OptionalHooks.useOptionalEvents();
        
        expect(events).toBe(mockEvents);
      });

      it('should return null when context is not available', () => {
        const events = OptionalHooks.useOptionalEvents();
        
        expect(events).toBeNull();
      });
    });

    describe('useOptionalStorage', () => {
      it('should return storage API when context is available', () => {
        contextManager.setContext(mockContext, 'test-plugin');
        
        const storage = OptionalHooks.useOptionalStorage();
        
        expect(storage).toBe(mockStorage);
      });

      it('should return null when context is not available', () => {
        const storage = OptionalHooks.useOptionalStorage();
        
        expect(storage).toBeNull();
      });
    });
  });

  describe('safeHookCall', () => {
    it('should return result when hook succeeds', () => {
      contextManager.setContext(mockContext, 'test-plugin');
      
      const result = safeHookCall(() => useApp(), 'useApp');
      
      expect(result).toBe(mockApp);
    });

    it('should re-throw PluginError as-is', () => {
      const originalError = new PluginError('Original error', PluginErrorCode.HOOK_ERROR);
      
      expect(() => safeHookCall(() => { throw originalError; }, 'testHook')).toThrow(originalError);
    });

    it('should wrap non-PluginError in PluginError', () => {
      const originalError = new Error('Regular error');
      
      try {
        safeHookCall(() => { throw originalError; }, 'testHook');
      } catch (error) {
        expect(error).toBeInstanceOf(PluginError);
        expect((error as PluginError).code).toBe(PluginErrorCode.HOOK_ERROR);
        expect((error as PluginError).message).toContain('Hook testHook() failed');
        expect((error as PluginError).message).toContain('Regular error');
      }
    });
  });
});

describe('Lifecycle Hooks', () => {
  beforeEach(() => {
    LifecycleUtils.clearLifecycleHandlers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    LifecycleUtils.clearLifecycleHandlers();
  });

  describe('onActivate', () => {
    it('should register activation handler', () => {
      const handler = vi.fn();
      
      onActivate(handler);
      
      const info = LifecycleUtils.getLifecycleInfo();
      expect(info.activate).toBe(1);
      expect(info.deactivate).toBe(0);
    });

    it('should register multiple activation handlers', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();
      
      onActivate(handler1);
      onActivate(handler2);
      
      const info = LifecycleUtils.getLifecycleInfo();
      expect(info.activate).toBe(2);
    });

    it('should throw error for non-function argument', () => {
      expect(() => onActivate('not a function' as any)).toThrow(PluginError);
      expect(() => onActivate('not a function' as any)).toThrow('onActivate() requires a function as argument');
    });

    it('should execute registered handlers', async () => {
      const handler1 = vi.fn().mockResolvedValue(undefined);
      const handler2 = vi.fn().mockResolvedValue(undefined);
      
      onActivate(handler1);
      onActivate(handler2);
      
      await LifecycleUtils.executeActivation();
      
      expect(handler1).toHaveBeenCalledOnce();
      expect(handler2).toHaveBeenCalledOnce();
    });

    it('should handle async handlers', async () => {
      const asyncHandler = vi.fn().mockImplementation(async () => {
        await new Promise(resolve => setTimeout(resolve, 10));
      });
      
      onActivate(asyncHandler);
      
      await LifecycleUtils.executeActivation();
      
      expect(asyncHandler).toHaveBeenCalledOnce();
    });

    it('should throw PluginError when handler fails', async () => {
      const failingHandler = vi.fn().mockRejectedValue(new Error('Handler failed'));
      
      onActivate(failingHandler);
      
      await expect(LifecycleUtils.executeActivation()).rejects.toThrow(PluginError);
      await expect(LifecycleUtils.executeActivation()).rejects.toThrow('Plugin activation handler failed');
    });
  });

  describe('onDeactivate', () => {
    it('should register deactivation handler', () => {
      const handler = vi.fn();
      
      onDeactivate(handler);
      
      const info = LifecycleUtils.getLifecycleInfo();
      expect(info.activate).toBe(0);
      expect(info.deactivate).toBe(1);
    });

    it('should register multiple deactivation handlers', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();
      
      onDeactivate(handler1);
      onDeactivate(handler2);
      
      const info = LifecycleUtils.getLifecycleInfo();
      expect(info.deactivate).toBe(2);
    });

    it('should throw error for non-function argument', () => {
      expect(() => onDeactivate(123 as any)).toThrow(PluginError);
      expect(() => onDeactivate(123 as any)).toThrow('onDeactivate() requires a function as argument');
    });

    it('should execute registered handlers', async () => {
      const handler1 = vi.fn().mockResolvedValue(undefined);
      const handler2 = vi.fn().mockResolvedValue(undefined);
      
      onDeactivate(handler1);
      onDeactivate(handler2);
      
      await LifecycleUtils.executeDeactivation();
      
      expect(handler1).toHaveBeenCalledOnce();
      expect(handler2).toHaveBeenCalledOnce();
    });

    it('should not throw when deactivation handler fails', async () => {
      const failingHandler = vi.fn().mockRejectedValue(new Error('Handler failed'));
      const workingHandler = vi.fn().mockResolvedValue(undefined);
      
      onDeactivate(failingHandler);
      onDeactivate(workingHandler);
      
      // Should not throw, but should continue with other handlers
      await expect(LifecycleUtils.executeDeactivation()).resolves.toBeUndefined();
      
      expect(failingHandler).toHaveBeenCalledOnce();
      expect(workingHandler).toHaveBeenCalledOnce();
    });
  });
});

describe('HookComposers', () => {
  beforeEach(() => {
    contextManager.setContext(mockContext, 'test-plugin');
    vi.clearAllMocks();
  });

  afterEach(() => {
    contextManager.clearContext();
  });

  describe('useAppAndStorage', () => {
    it('should return app and storage APIs', () => {
      const { app, storage } = HookComposers.useAppAndStorage();
      
      expect(app).toBe(mockApp);
      expect(storage).toBe(mockStorage);
    });
  });

  describe('useAllAPIs', () => {
    it('should return all APIs', () => {
      const { app, events, storage } = HookComposers.useAllAPIs();
      
      expect(app).toBe(mockApp);
      expect(events).toBe(mockEvents);
      expect(storage).toBe(mockStorage);
    });
  });

  describe('useSafeAPIs', () => {
    it('should return all APIs when context is available', () => {
      const { app, events, storage } = HookComposers.useSafeAPIs();
      
      expect(app).toBe(mockApp);
      expect(events).toBe(mockEvents);
      expect(storage).toBe(mockStorage);
    });

    it('should return null values when context is not available', () => {
      contextManager.clearContext();
      
      const { app, events, storage } = HookComposers.useSafeAPIs();
      
      expect(app).toBeNull();
      expect(events).toBeNull();
      expect(storage).toBeNull();
    });
  });
});

describe('LifecycleUtils', () => {
  beforeEach(() => {
    LifecycleUtils.clearLifecycleHandlers();
  });

  afterEach(() => {
    LifecycleUtils.clearLifecycleHandlers();
  });

  describe('getLifecycleInfo', () => {
    it('should return correct handler counts', () => {
      expect(LifecycleUtils.getLifecycleInfo()).toEqual({
        activate: 0,
        deactivate: 0
      });
      
      onActivate(() => {});
      onActivate(() => {});
      onDeactivate(() => {});
      
      expect(LifecycleUtils.getLifecycleInfo()).toEqual({
        activate: 2,
        deactivate: 1
      });
    });
  });

  describe('clearLifecycleHandlers', () => {
    it('should clear all registered handlers', () => {
      onActivate(() => {});
      onDeactivate(() => {});
      
      expect(LifecycleUtils.getLifecycleInfo().activate).toBe(1);
      expect(LifecycleUtils.getLifecycleInfo().deactivate).toBe(1);
      
      LifecycleUtils.clearLifecycleHandlers();
      
      expect(LifecycleUtils.getLifecycleInfo()).toEqual({
        activate: 0,
        deactivate: 0
      });
    });
  });
});

describe('Integration Tests', () => {
  beforeEach(() => {
    contextManager.clearContext();
    LifecycleUtils.clearLifecycleHandlers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    contextManager.clearContext();
    LifecycleUtils.clearLifecycleHandlers();
  });

  it('should work with realistic plugin scenario', async () => {
    // Simulate plugin activation
    contextManager.setContext(mockContext, 'test-plugin');
    
    // Register lifecycle handlers
    const activateHandler = vi.fn(async () => {
      const app = useApp();
      await app.showNotification('Plugin activated');
      
      const storage = useStorage();
      await storage.set('initialized', true);
    });
    
    const deactivateHandler = vi.fn(async () => {
      const events = useEvents();
      events.off('test-event', () => {});
    });
    
    onActivate(activateHandler);
    onDeactivate(deactivateHandler);
    
    // Execute activation
    await LifecycleUtils.executeActivation();
    
    expect(activateHandler).toHaveBeenCalledOnce();
    expect(mockApp.showNotification).toHaveBeenCalledWith('Plugin activated');
    expect(mockStorage.set).toHaveBeenCalledWith('initialized', true);
    
    // Execute deactivation
    await LifecycleUtils.executeDeactivation();
    
    expect(deactivateHandler).toHaveBeenCalledOnce();
    expect(mockEvents.off).toHaveBeenCalledWith('test-event', expect.any(Function));
  });

  it('should handle mixed sync and async handlers', async () => {
    contextManager.setContext(mockContext, 'test-plugin');
    
    const syncHandler = vi.fn(() => {
      const app = useApp();
      // Sync operation
    });
    
    const asyncHandler = vi.fn(async () => {
      const storage = useStorage();
      await storage.set('async-data', 'value');
    });
    
    onActivate(syncHandler);
    onActivate(asyncHandler);
    
    await LifecycleUtils.executeActivation();
    
    expect(syncHandler).toHaveBeenCalledOnce();
    expect(asyncHandler).toHaveBeenCalledOnce();
    expect(mockStorage.set).toHaveBeenCalledWith('async-data', 'value');
  });

  it('should maintain context during handler execution', async () => {
    contextManager.setContext(mockContext, 'test-plugin');
    
    let capturedContext: PluginContext | null = null;
    
    const handler = vi.fn(() => {
      capturedContext = useContext();
    });
    
    onActivate(handler);
    await LifecycleUtils.executeActivation();
    
    expect(handler).toHaveBeenCalledOnce();
    expect(capturedContext).toBe(mockContext);
  });
});