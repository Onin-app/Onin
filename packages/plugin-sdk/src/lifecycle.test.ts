/**
 * Tests for Plugin Lifecycle Management System
 */

import { describe, it, expect, beforeEach, afterEach, vi, Mock } from 'vitest';
import { 
  PluginLifecycleManager, 
  defaultLifecycleManager,
  activatePlugin,
  deactivatePlugin,
  isPluginActive,
  getActivePlugins
} from './lifecycle';
import { Plugin, PluginError, PluginErrorCode } from './types';
import { contextManager } from './context';
import { definePlugin } from './plugin';

// Mock the context creation
vi.mock('./context', async () => {
  const actual = await vi.importActual('./context');
  return {
    ...actual,
    createPluginContext: vi.fn(() => ({
      app: {
        showNotification: vi.fn(),
        getAppVersion: vi.fn(),
        openDialog: vi.fn()
      },
      events: {
        on: vi.fn(),
        emit: vi.fn(),
        off: vi.fn()
      },
      storage: {
        get: vi.fn(),
        set: vi.fn(),
        remove: vi.fn(),
        clear: vi.fn(),
        keys: vi.fn()
      }
    })),
    contextManager: {
      setContext: vi.fn(),
      getContext: vi.fn(),
      clearContext: vi.fn(),
      hasContext: vi.fn(),
      getCurrentPluginId: vi.fn(),
      withContext: vi.fn()
    }
  };
});

describe('PluginLifecycleManager', () => {
  let lifecycleManager: PluginLifecycleManager;
  let mockPlugin: Plugin;
  let mockActivate: Mock;
  let mockDeactivate: Mock;

  beforeEach(() => {
    lifecycleManager = new PluginLifecycleManager({ debug: true });
    
    mockActivate = vi.fn();
    mockDeactivate = vi.fn();
    
    mockPlugin = definePlugin({
      meta: {
        name: 'Test Plugin',
        version: '1.0.0',
        description: 'A test plugin',
        author: 'Test Author'
      },
      onActivate: mockActivate,
      onDeactivate: mockDeactivate
    });

    // Reset all mocks
    vi.clearAllMocks();
  });

  afterEach(async () => {
    // Clean up any active plugins
    await lifecycleManager.deactivateAllPlugins();
  });

  describe('Plugin Activation', () => {
    it('should activate a plugin successfully', async () => {
      const pluginId = 'test-plugin-1';
      
      await lifecycleManager.activatePlugin(mockPlugin, pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
      expect(contextManager.setContext).toHaveBeenCalled();
      expect(mockActivate).toHaveBeenCalled();
    });

    it('should create and inject plugin context during activation', async () => {
      const pluginId = 'test-plugin-2';
      const contextConfig = { 
        communication: { timeout: 5000 },
        events: { maxListeners: 20 }
      };
      
      await lifecycleManager.activatePlugin(mockPlugin, pluginId, contextConfig);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
      const context = lifecycleManager.getPluginContext(pluginId);
      expect(context).toBeDefined();
      expect(context?.app).toBeDefined();
      expect(context?.events).toBeDefined();
      expect(context?.storage).toBeDefined();
    });

    it('should handle plugin activation without onActivate function', async () => {
      const pluginWithoutActivate = definePlugin({
        meta: { name: 'No Activate Plugin' },
        onDeactivate: mockDeactivate
      });
      
      const pluginId = 'no-activate-plugin';
      
      await lifecycleManager.activatePlugin(pluginWithoutActivate, pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
    });

    it('should throw error when activating already active plugin', async () => {
      const pluginId = 'duplicate-plugin';
      
      await lifecycleManager.activatePlugin(mockPlugin, pluginId);
      
      await expect(
        lifecycleManager.activatePlugin(mockPlugin, pluginId)
      ).rejects.toThrow(PluginError);
      
      await expect(
        lifecycleManager.activatePlugin(mockPlugin, pluginId)
      ).rejects.toThrow('already active');
    });

    it('should handle activation errors properly', async () => {
      const errorPlugin = definePlugin({
        meta: { name: 'Error Plugin' },
        onActivate: () => {
          throw new Error('Activation failed');
        }
      });
      
      const pluginId = 'error-plugin';
      
      await expect(
        lifecycleManager.activatePlugin(errorPlugin, pluginId)
      ).rejects.toThrow(PluginError);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
      expect(contextManager.clearContext).toHaveBeenCalled();
    });

    it('should handle async activation functions', async () => {
      const asyncPlugin = definePlugin({
        meta: { name: 'Async Plugin' },
        onActivate: async () => {
          await new Promise(resolve => setTimeout(resolve, 10));
        }
      });
      
      const pluginId = 'async-plugin';
      
      await lifecycleManager.activatePlugin(asyncPlugin, pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
    });

    it('should timeout on slow activation', async () => {
      const slowPlugin = definePlugin({
        meta: { name: 'Slow Plugin' },
        onActivate: async () => {
          await new Promise(resolve => setTimeout(resolve, 100));
        }
      });
      
      const fastManager = new PluginLifecycleManager({ timeout: 50 });
      const pluginId = 'slow-plugin';
      
      await expect(
        fastManager.activatePlugin(slowPlugin, pluginId)
      ).rejects.toThrow('timed out');
    });
  });

  describe('Plugin Deactivation', () => {
    beforeEach(async () => {
      await lifecycleManager.activatePlugin(mockPlugin, 'test-plugin');
    });

    it('should deactivate an active plugin successfully', async () => {
      const pluginId = 'test-plugin';
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
      
      await lifecycleManager.deactivatePlugin(pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
      expect(mockDeactivate).toHaveBeenCalled();
      expect(contextManager.clearContext).toHaveBeenCalled();
    });

    it('should handle deactivation of inactive plugin gracefully', async () => {
      const pluginId = 'test-plugin';
      
      await lifecycleManager.deactivatePlugin(pluginId);
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
      
      // Should not throw when deactivating again
      await expect(
        lifecycleManager.deactivatePlugin(pluginId)
      ).resolves.not.toThrow();
    });

    it('should handle plugin deactivation without onDeactivate function', async () => {
      const pluginWithoutDeactivate = definePlugin({
        meta: { name: 'No Deactivate Plugin' },
        onActivate: mockActivate
      });
      
      const pluginId = 'no-deactivate-plugin';
      
      await lifecycleManager.activatePlugin(pluginWithoutDeactivate, pluginId);
      await lifecycleManager.deactivatePlugin(pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
    });

    it('should throw error when deactivating non-existent plugin', async () => {
      await expect(
        lifecycleManager.deactivatePlugin('non-existent-plugin')
      ).rejects.toThrow(PluginError);
      
      await expect(
        lifecycleManager.deactivatePlugin('non-existent-plugin')
      ).rejects.toThrow('not registered');
    });

    it('should handle deactivation errors properly', async () => {
      const errorPlugin = definePlugin({
        meta: { name: 'Error Plugin' },
        onActivate: mockActivate,
        onDeactivate: () => {
          throw new Error('Deactivation failed');
        }
      });
      
      const pluginId = 'error-deactivate-plugin';
      
      await lifecycleManager.activatePlugin(errorPlugin, pluginId);
      
      await expect(
        lifecycleManager.deactivatePlugin(pluginId)
      ).rejects.toThrow(PluginError);
      
      // Plugin should be marked as inactive even if deactivation failed
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
    });

    it('should handle async deactivation functions', async () => {
      const asyncPlugin = definePlugin({
        meta: { name: 'Async Plugin' },
        onActivate: mockActivate,
        onDeactivate: async () => {
          await new Promise(resolve => setTimeout(resolve, 10));
        }
      });
      
      const pluginId = 'async-deactivate-plugin';
      
      await lifecycleManager.activatePlugin(asyncPlugin, pluginId);
      await lifecycleManager.deactivatePlugin(pluginId);
      
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
    });
  });

  describe('Plugin State Management', () => {
    it('should track plugin states correctly', async () => {
      const pluginId = 'state-test-plugin';
      
      // Initially no state
      expect(lifecycleManager.getPluginState(pluginId)).toBeNull();
      
      // After activation
      await lifecycleManager.activatePlugin(mockPlugin, pluginId);
      const activeState = lifecycleManager.getPluginState(pluginId);
      
      expect(activeState).toBeDefined();
      expect(activeState?.isActive).toBe(true);
      expect(activeState?.pluginId).toBe(pluginId);
      expect(activeState?.activatedAt).toBeInstanceOf(Date);
      
      // After deactivation
      await lifecycleManager.deactivatePlugin(pluginId);
      const inactiveState = lifecycleManager.getPluginState(pluginId);
      
      expect(inactiveState?.isActive).toBe(false);
      expect(inactiveState?.activatedAt).toBeUndefined();
    });

    it('should return correct list of registered plugins', async () => {
      const plugin1Id = 'plugin-1';
      const plugin2Id = 'plugin-2';
      
      expect(lifecycleManager.getRegisteredPlugins()).toHaveLength(0);
      
      await lifecycleManager.activatePlugin(mockPlugin, plugin1Id);
      expect(lifecycleManager.getRegisteredPlugins()).toContain(plugin1Id);
      
      await lifecycleManager.activatePlugin(mockPlugin, plugin2Id);
      expect(lifecycleManager.getRegisteredPlugins()).toHaveLength(2);
      expect(lifecycleManager.getRegisteredPlugins()).toContain(plugin2Id);
    });

    it('should return correct list of active plugins', async () => {
      const plugin1Id = 'active-plugin-1';
      const plugin2Id = 'active-plugin-2';
      
      expect(lifecycleManager.getActivePlugins()).toHaveLength(0);
      
      await lifecycleManager.activatePlugin(mockPlugin, plugin1Id);
      expect(lifecycleManager.getActivePlugins()).toContain(plugin1Id);
      
      await lifecycleManager.activatePlugin(mockPlugin, plugin2Id);
      expect(lifecycleManager.getActivePlugins()).toHaveLength(2);
      
      await lifecycleManager.deactivatePlugin(plugin1Id);
      expect(lifecycleManager.getActivePlugins()).toHaveLength(1);
      expect(lifecycleManager.getActivePlugins()).toContain(plugin2Id);
    });

    it('should store error information in plugin state', async () => {
      const errorPlugin = definePlugin({
        meta: { name: 'Error Plugin' },
        onActivate: () => {
          throw new Error('Test error');
        }
      });
      
      const pluginId = 'error-state-plugin';
      
      try {
        await lifecycleManager.activatePlugin(errorPlugin, pluginId);
      } catch (error) {
        // Expected to throw
      }
      
      const state = lifecycleManager.getPluginState(pluginId);
      expect(state?.lastError).toBeInstanceOf(PluginError);
      expect(state?.isActive).toBe(false);
    });
  });

  describe('Bulk Operations', () => {
    it('should deactivate all active plugins', async () => {
      const plugin1Id = 'bulk-plugin-1';
      const plugin2Id = 'bulk-plugin-2';
      const plugin3Id = 'bulk-plugin-3';
      
      await lifecycleManager.activatePlugin(mockPlugin, plugin1Id);
      await lifecycleManager.activatePlugin(mockPlugin, plugin2Id);
      await lifecycleManager.activatePlugin(mockPlugin, plugin3Id);
      
      expect(lifecycleManager.getActivePlugins()).toHaveLength(3);
      
      await lifecycleManager.deactivateAllPlugins();
      
      expect(lifecycleManager.getActivePlugins()).toHaveLength(0);
      expect(lifecycleManager.isPluginActive(plugin1Id)).toBe(false);
      expect(lifecycleManager.isPluginActive(plugin2Id)).toBe(false);
      expect(lifecycleManager.isPluginActive(plugin3Id)).toBe(false);
    });

    it('should handle errors during bulk deactivation', async () => {
      const goodPlugin = definePlugin({
        meta: { name: 'Good Plugin' },
        onActivate: mockActivate,
        onDeactivate: mockDeactivate
      });
      
      const errorPlugin = definePlugin({
        meta: { name: 'Error Plugin' },
        onActivate: mockActivate,
        onDeactivate: () => {
          throw new Error('Deactivation error');
        }
      });
      
      await lifecycleManager.activatePlugin(goodPlugin, 'good-plugin');
      await lifecycleManager.activatePlugin(errorPlugin, 'error-plugin');
      
      // Should not throw even if one plugin fails
      await expect(
        lifecycleManager.deactivateAllPlugins()
      ).resolves.not.toThrow();
      
      // Both should be marked as inactive
      expect(lifecycleManager.isPluginActive('good-plugin')).toBe(false);
      expect(lifecycleManager.isPluginActive('error-plugin')).toBe(false);
    });

    it('should remove plugins properly', async () => {
      const pluginId = 'removable-plugin';
      
      await lifecycleManager.activatePlugin(mockPlugin, pluginId);
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(true);
      
      const removed = await lifecycleManager.removePlugin(pluginId);
      
      expect(removed).toBe(true);
      expect(lifecycleManager.isPluginActive(pluginId)).toBe(false);
      expect(lifecycleManager.getPluginState(pluginId)).toBeNull();
      expect(lifecycleManager.getRegisteredPlugins()).not.toContain(pluginId);
    });

    it('should return false when removing non-existent plugin', async () => {
      const removed = await lifecycleManager.removePlugin('non-existent');
      expect(removed).toBe(false);
    });
  });

  describe('Configuration Management', () => {
    it('should update configuration correctly', () => {
      const newConfig = {
        timeout: 60000,
        debug: false,
        defaultContextConfig: {
          communication: { timeout: 10000 }
        }
      };
      
      lifecycleManager.updateConfig(newConfig);
      
      const config = lifecycleManager.getConfig();
      expect(config.timeout).toBe(60000);
      expect(config.debug).toBe(false);
      expect(config.defaultContextConfig).toEqual(newConfig.defaultContextConfig);
    });

    it('should merge configuration updates', () => {
      const initialConfig = lifecycleManager.getConfig();
      
      lifecycleManager.updateConfig({ timeout: 45000 });
      
      const updatedConfig = lifecycleManager.getConfig();
      expect(updatedConfig.timeout).toBe(45000);
      expect(updatedConfig.debug).toBe(initialConfig.debug); // Should preserve other values
    });
  });
});

describe('Utility Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(async () => {
    await defaultLifecycleManager.deactivateAllPlugins();
  });

  it('should activate plugin using utility function', async () => {
    const plugin = definePlugin({
      meta: { name: 'Utility Test Plugin' },
      onActivate: vi.fn()
    });
    
    const pluginId = 'utility-plugin';
    
    await activatePlugin(plugin, pluginId);
    
    expect(isPluginActive(pluginId)).toBe(true);
    expect(getActivePlugins()).toContain(pluginId);
  });

  it('should deactivate plugin using utility function', async () => {
    const plugin = definePlugin({
      meta: { name: 'Utility Test Plugin' },
      onActivate: vi.fn(),
      onDeactivate: vi.fn()
    });
    
    const pluginId = 'utility-deactivate-plugin';
    
    await activatePlugin(plugin, pluginId);
    expect(isPluginActive(pluginId)).toBe(true);
    
    await deactivatePlugin(pluginId);
    expect(isPluginActive(pluginId)).toBe(false);
  });

  it('should return correct active plugins list', async () => {
    const plugin1 = definePlugin({
      meta: { name: 'Plugin 1' },
      onActivate: vi.fn()
    });
    
    const plugin2 = definePlugin({
      meta: { name: 'Plugin 2' },
      onActivate: vi.fn()
    });
    
    expect(getActivePlugins()).toHaveLength(0);
    
    await activatePlugin(plugin1, 'util-plugin-1');
    await activatePlugin(plugin2, 'util-plugin-2');
    
    const activePlugins = getActivePlugins();
    expect(activePlugins).toHaveLength(2);
    expect(activePlugins).toContain('util-plugin-1');
    expect(activePlugins).toContain('util-plugin-2');
  });
});