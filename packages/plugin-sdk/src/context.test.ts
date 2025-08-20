/**
 * Tests for the context management system
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { 
  contextManager, 
  getCurrentContext, 
  hasCurrentContext,
  validatePluginContext,
  withContextErrorHandling,
  createPluginContext,
  PluginInitializer
} from './context';
import { PluginContext, PluginError, PluginErrorCode } from './types';

// Mock the communication module
vi.mock('./communication', () => ({
  createCommunicationBridge: vi.fn(() => ({
    invoke: vi.fn(),
    updateConfig: vi.fn()
  })),
  setGlobalBridge: vi.fn()
}));

// Mock the events module
vi.mock('./events', () => ({
  EventManager: vi.fn(() => ({
    on: vi.fn(),
    off: vi.fn(),
    emit: vi.fn(),
    removeAllListeners: vi.fn()
  }))
}));

// Mock the api module
vi.mock('./api', () => ({
  createAPIImplementations: vi.fn(() => ({
    app: {
      showNotification: vi.fn(),
      getAppVersion: vi.fn(),
      openDialog: vi.fn(),
      hasPermission: vi.fn(() => Promise.resolve(true))
    },
    storage: {
      get: vi.fn(),
      set: vi.fn(),
      remove: vi.fn(),
      clear: vi.fn(),
      keys: vi.fn()
    }
  }))
}));

describe('ContextManager', () => {
  beforeEach(() => {
    contextManager.clearContext();
  });

  it('should start with no context', () => {
    expect(hasCurrentContext()).toBe(false);
    expect(() => getCurrentContext()).toThrow(PluginError);
  });

  it('should set and get context correctly', () => {
    const mockContext = createMockContext();
    
    contextManager.setContext(mockContext, 'test-plugin');
    
    expect(hasCurrentContext()).toBe(true);
    expect(getCurrentContext()).toBe(mockContext);
    expect(contextManager.getCurrentPluginId()).toBe('test-plugin');
  });

  it('should clear context correctly', () => {
    const mockContext = createMockContext();
    
    contextManager.setContext(mockContext, 'test-plugin');
    contextManager.clearContext();
    
    expect(hasCurrentContext()).toBe(false);
    expect(contextManager.getCurrentPluginId()).toBe(null);
    expect(() => getCurrentContext()).toThrow(PluginError);
  });

  it('should execute function with context correctly', async () => {
    const mockContext = createMockContext();
    let capturedContext: PluginContext | null = null;
    
    const result = await contextManager.withContext(
      mockContext,
      'test-plugin',
      () => {
        capturedContext = getCurrentContext();
        return 'test-result';
      }
    );
    
    expect(result).toBe('test-result');
    expect(capturedContext).toBe(mockContext);
    expect(hasCurrentContext()).toBe(false); // Context should be cleared after execution
  });

  it('should restore previous context after withContext', async () => {
    const context1 = createMockContext();
    const context2 = createMockContext();
    
    contextManager.setContext(context1, 'plugin-1');
    
    await contextManager.withContext(context2, 'plugin-2', () => {
      expect(getCurrentContext()).toBe(context2);
      expect(contextManager.getCurrentPluginId()).toBe('plugin-2');
    });
    
    expect(getCurrentContext()).toBe(context1);
    expect(contextManager.getCurrentPluginId()).toBe('plugin-1');
  });

  it('should handle errors in withContext and restore previous context', async () => {
    const context1 = createMockContext();
    const context2 = createMockContext();
    
    contextManager.setContext(context1, 'plugin-1');
    
    await expect(
      contextManager.withContext(context2, 'plugin-2', () => {
        throw new Error('Test error');
      })
    ).rejects.toThrow('Test error');
    
    expect(getCurrentContext()).toBe(context1);
    expect(contextManager.getCurrentPluginId()).toBe('plugin-1');
  });
});

describe('validatePluginContext', () => {
  it('should validate correct context', () => {
    const mockContext = createMockContext();
    expect(() => validatePluginContext(mockContext)).not.toThrow();
  });

  it('should throw for null context', () => {
    expect(() => validatePluginContext(null)).toThrow(PluginError);
  });

  it('should throw for context missing app API', () => {
    const invalidContext = { events: {}, storage: {} };
    expect(() => validatePluginContext(invalidContext)).toThrow(PluginError);
  });

  it('should throw for context missing events API', () => {
    const invalidContext = { app: {}, storage: {} };
    expect(() => validatePluginContext(invalidContext)).toThrow(PluginError);
  });

  it('should throw for context missing storage API', () => {
    const invalidContext = { app: {}, events: {} };
    expect(() => validatePluginContext(invalidContext)).toThrow(PluginError);
  });
});

describe('withContextErrorHandling', () => {
  it('should execute function successfully', async () => {
    const result = await withContextErrorHandling(() => 'success');
    expect(result).toBe('success');
  });

  it('should pass through PluginError unchanged', async () => {
    const originalError = new PluginError('Test error', PluginErrorCode.CONTEXT_NOT_AVAILABLE);
    
    await expect(
      withContextErrorHandling(() => {
        throw originalError;
      })
    ).rejects.toBe(originalError);
  });

  it('should wrap other errors as PluginError', async () => {
    await expect(
      withContextErrorHandling(() => {
        throw new Error('Regular error');
      }, 'test-plugin')
    ).rejects.toMatchObject({
      name: 'PluginError',
      code: PluginErrorCode.CONTEXT_NOT_AVAILABLE,
      pluginName: 'test-plugin'
    });
  });
});

describe('PluginInitializer', () => {
  beforeEach(() => {
    // Mock window object and __TAURI__
    Object.defineProperty(global, 'window', {
      value: {
        __TAURI__: {}
      },
      writable: true
    });
  });

  it('should initialize context and set it in context manager', async () => {
    const initializer = new PluginInitializer({ pluginId: 'test-plugin' });
    
    const context = await initializer.initialize();
    
    expect(context).toBeDefined();
    expect(getCurrentContext()).toBe(context);
    expect(contextManager.getCurrentPluginId()).toBe('test-plugin');
  });

  it('should throw PluginError when Tauri is not available', async () => {
    // Remove window object to simulate non-Tauri environment
    Object.defineProperty(global, 'window', {
      value: undefined,
      writable: true
    });
    
    const initializer = new PluginInitializer({ pluginId: 'test-plugin' });
    
    await expect(initializer.initialize()).rejects.toMatchObject({
      name: 'PluginError',
      code: PluginErrorCode.LOAD_FAILED,
      pluginName: 'test-plugin'
    });
  });

  it('should clear context on cleanup', async () => {
    const initializer = new PluginInitializer({ pluginId: 'test-plugin' });
    
    await initializer.initialize();
    expect(hasCurrentContext()).toBe(true);
    
    await initializer.cleanup();
    expect(hasCurrentContext()).toBe(false);
  });
});

// Helper function to create mock context
function createMockContext(): PluginContext {
  return {
    app: {
      showNotification: vi.fn(),
      getAppVersion: vi.fn(),
      openDialog: vi.fn()
    },
    events: {
      on: vi.fn(),
      off: vi.fn(),
      emit: vi.fn()
    },
    storage: {
      get: vi.fn(),
      set: vi.fn(),
      remove: vi.fn(),
      clear: vi.fn(),
      keys: vi.fn()
    }
  };
}