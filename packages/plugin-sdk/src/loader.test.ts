/**
 * Integration Tests for Plugin Loader
 * 
 * These tests verify the plugin loader functionality including:
 * - Loading functional plugins from various formats
 * - Error handling and validation
 * - Context injection
 * - Module caching behavior
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import * as fs from 'fs/promises';
import * as path from 'path';
import { PluginLoader, loadPlugin, loadPlugins, defaultPluginLoader } from './loader';
import { Plugin, PluginContext, PluginError, PluginErrorCode } from './types';
import { definePlugin } from './plugin';

// Mock file system for testing
vi.mock('fs/promises');
const mockFs = vi.mocked(fs);

// Mock dynamic imports
const mockImport = vi.fn();

// Setup global mock for import function
beforeEach(() => {
  (globalThis as any).import = mockImport;
});

// Test fixtures directory
const FIXTURES_DIR = '/test/fixtures';

// Mock plugin context
const mockContext: PluginContext = {
  app: {
    showNotification: vi.fn().mockResolvedValue(undefined),
    getAppVersion: vi.fn().mockResolvedValue('1.0.0'),
    openDialog: vi.fn().mockResolvedValue('ok')
  },
  events: {
    on: vi.fn(),
    emit: vi.fn(),
    off: vi.fn()
  },
  storage: {
    get: vi.fn().mockResolvedValue(undefined),
    set: vi.fn().mockResolvedValue(undefined),
    remove: vi.fn().mockResolvedValue(undefined),
    clear: vi.fn().mockResolvedValue(undefined),
    keys: vi.fn().mockResolvedValue([])
  }
};

// Mock plugins for testing
const validFunctionalPlugin: Plugin = definePlugin({
  meta: {
    name: 'Test Plugin',
    version: '1.0.0',
    description: 'A test plugin',
    author: 'Test Author'
  },
  onActivate: async (context) => {
    await context.app.showNotification('Plugin activated');
  },
  onDeactivate: async () => {
    console.log('Plugin deactivated');
  }
});

const minimalPlugin: Plugin = {
  onActivate: async () => {
    console.log('Minimal plugin activated');
  }
};

const invalidPlugin = {
  notAPlugin: true,
  someData: 'invalid'
};

describe('PluginLoader', () => {
  let loader: PluginLoader;

  beforeEach(() => {
    loader = new PluginLoader();
    vi.clearAllMocks();
    
    // Clear require cache
    Object.keys(require.cache).forEach(key => {
      delete require.cache[key];
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('constructor', () => {
    it('should create loader with default options', () => {
      const loader = new PluginLoader();
      const stats = loader.getStats();
      
      expect(stats.options.strictValidation).toBe(true);
      expect(stats.options.timeout).toBe(5000);
    });

    it('should create loader with custom options', () => {
      const loader = new PluginLoader({
        strictValidation: false,
        timeout: 10000
      });
      const stats = loader.getStats();
      
      expect(stats.options.strictValidation).toBe(false);
      expect(stats.options.timeout).toBe(10000);
    });
  });

  describe('loadPlugin', () => {
    beforeEach(() => {
      // Setup mock import responses
      mockImport.mockImplementation((path: string) => {
        const normalizedPath = path.replace(/\\/g, '/');
        
        if (normalizedPath.includes('valid-plugin.js')) {
          return Promise.resolve({ default: validFunctionalPlugin });
        }
        if (normalizedPath.includes('minimal-plugin.js')) {
          return Promise.resolve({ default: minimalPlugin });
        }
        if (normalizedPath.includes('invalid-plugin.js')) {
          return Promise.resolve({ default: invalidPlugin });
        }
        if (normalizedPath.includes('named-export.js')) {
          return Promise.resolve({ plugin: validFunctionalPlugin });
        }
        if (normalizedPath.includes('syntax-error.js')) {
          return Promise.reject(new SyntaxError('Unexpected token'));
        }
        if (normalizedPath.includes('missing-deps.js')) {
          const error = new Error('Cannot find module \'missing-dependency\'');
          (error as any).code = 'MODULE_NOT_FOUND';
          return Promise.reject(error);
        }
        if (normalizedPath.includes('slow-plugin.js')) {
          return new Promise(resolve => 
            setTimeout(() => resolve({ default: validFunctionalPlugin }), 200)
          );
        }
        if (normalizedPath.includes('plugin-dir/index.js') || normalizedPath.includes('plugin-dir/plugin.js')) {
          return Promise.resolve({ default: validFunctionalPlugin });
        }
        
        // Default case - file not found
        return Promise.reject(new Error(`Failed to load url ${path} (resolved id: ${path}). Does the file exist?`));
      });
    });

    it('should load a valid functional plugin from file', async () => {
      // Mock file system
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      const result = await loader.loadPlugin('/test/fixtures/valid-plugin.js');

      expect(result.plugin).toBeDefined();
      expect(result.path).toBe(path.resolve('/test/fixtures/valid-plugin.js'));
      expect(result.module).toBeDefined();
      expect(result.plugin.meta?.name).toBe('Test Plugin');
    });

    it('should load plugin with context injection', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      const result = await loader.loadPlugin('/test/fixtures/valid-plugin.js', mockContext);

      expect(result.plugin).toBeDefined();
      expect(result.plugin.onActivate).toBeDefined();
      
      // Test that context is properly injected
      await result.plugin.onActivate!();
      expect(mockContext.app.showNotification).toHaveBeenCalledWith('Plugin activated');
    });

    it('should load minimal plugin without metadata', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      const result = await loader.loadPlugin('/test/fixtures/minimal-plugin.js');

      expect(result.plugin).toBeDefined();
      expect(result.plugin.onActivate).toBeDefined();
      expect(result.plugin.meta).toBeUndefined();
    });

    it('should load plugin from named export', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      const result = await loader.loadPlugin('/test/fixtures/named-export.js');

      expect(result.plugin).toBeDefined();
      expect(result.plugin.meta?.name).toBe('Test Plugin');
    });

    it('should load plugin from directory with index.js', async () => {
      mockFs.stat
        .mockResolvedValueOnce({
          isFile: () => false,
          isDirectory: () => true
        } as any)
        .mockResolvedValueOnce({
          isFile: () => true,
          isDirectory: () => false
        } as any);

      mockFs.readFile.mockRejectedValue(new Error('ENOENT')); // No package.json

      const result = await loader.loadPlugin('/test/fixtures/plugin-dir');

      expect(result.plugin).toBeDefined();
      expect(mockFs.stat).toHaveBeenCalledWith(path.resolve('/test/fixtures/plugin-dir/index.js'));
    });

    it('should load plugin from directory with package.json main field', async () => {
      mockFs.stat
        .mockResolvedValueOnce({
          isFile: () => false,
          isDirectory: () => true
        } as any)
        .mockResolvedValueOnce({
          isFile: () => true,
          isDirectory: () => false
        } as any);

      mockFs.readFile.mockResolvedValue(JSON.stringify({
        main: 'plugin.js'
      }));

      const result = await loader.loadPlugin('/test/fixtures/plugin-dir');

      expect(result.plugin).toBeDefined();
      expect(mockFs.readFile).toHaveBeenCalledWith(
        path.resolve('/test/fixtures/plugin-dir/package.json'),
        'utf-8'
      );
    });

    it('should throw error for invalid plugin path', async () => {
      await expect(loader.loadPlugin('')).rejects.toThrow(PluginError);
      await expect(loader.loadPlugin(null as any)).rejects.toThrow(PluginError);
    });

    it('should throw error for non-existent plugin', async () => {
      mockFs.stat.mockRejectedValue({ code: 'ENOENT' });

      await expect(loader.loadPlugin('/non/existent/plugin.js')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.NOT_FOUND
        })
      );
    });

    it('should throw error for invalid plugin structure', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      // Override the mock specifically for this test to return the actual invalid plugin
      mockImport.mockImplementationOnce((path: string) => {
        if (path.includes('invalid-plugin.js')) {
          return Promise.resolve({ default: invalidPlugin });
        }
        return Promise.reject(new Error(`Failed to load url ${path}`));
      });

      await expect(loader.loadPlugin('/test/fixtures/invalid-plugin.js')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.INVALID_MANIFEST
        })
      );
    });

    it('should throw error for syntax errors in plugin', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      await expect(loader.loadPlugin('/test/fixtures/syntax-error.js')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.LOAD_FAILED
        })
      );
    });

    it('should handle loading timeout', async () => {
      const slowLoader = new PluginLoader({ timeout: 100 });
      
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      // Mock a slow import
      vi.doMock('/test/fixtures/slow-plugin.js', () => 
        new Promise(resolve => setTimeout(() => resolve({ default: validFunctionalPlugin }), 200)),
        { virtual: true }
      );

      await expect(slowLoader.loadPlugin('/test/fixtures/slow-plugin.js')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.LOAD_FAILED,
          message: expect.stringContaining('timed out')
        })
      );
    });

    it('should not validate with strict validation disabled', async () => {
      const lenientLoader = new PluginLoader({ strictValidation: false });
      
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      // This should not throw even with invalid plugin when strict validation is off
      const result = await lenientLoader.loadPlugin('/test/fixtures/invalid-plugin.js');
      expect(result.plugin).toBeDefined();
    });
  });

  describe('loadPlugins', () => {
    beforeEach(() => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      // Setup additional mock responses for loadPlugins tests
      mockImport.mockImplementation((path: string) => {
        const normalizedPath = path.replace(/\\/g, '/');
        
        if (normalizedPath.includes('plugin1.js')) {
          return Promise.resolve({ default: validFunctionalPlugin });
        }
        if (normalizedPath.includes('plugin2.js')) {
          return Promise.resolve({ default: minimalPlugin });
        }
        if (normalizedPath.includes('invalid.js')) {
          return Promise.resolve({ default: invalidPlugin });
        }
        
        return Promise.reject(new Error(`Failed to load url ${path}`));
      });
    });

    it('should load multiple valid plugins', async () => {
      const results = await loader.loadPlugins([
        '/test/fixtures/plugin1.js',
        '/test/fixtures/plugin2.js'
      ]);

      expect(results).toHaveLength(2);
      expect(results[0].plugin.meta?.name).toBe('Test Plugin');
      expect(results[1].plugin.onActivate).toBeDefined();
    });

    it('should handle mixed valid and invalid plugins', async () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const results = await loader.loadPlugins([
        '/test/fixtures/plugin1.js',
        '/test/fixtures/invalid.js',
        '/test/fixtures/plugin2.js'
      ]);

      expect(results).toHaveLength(2); // Only valid plugins
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to load 1 plugins'),
        expect.any(Array)
      );

      consoleSpy.mockRestore();
    });

    it('should load plugins with context injection', async () => {
      const results = await loader.loadPlugins([
        '/test/fixtures/plugin1.js'
      ], mockContext);

      expect(results).toHaveLength(1);
      
      // Test context injection
      await results[0].plugin.onActivate!();
      expect(mockContext.app.showNotification).toHaveBeenCalled();
    });
  });

  describe('utility methods', () => {
    it('should check if plugin is loaded', () => {
      const pluginPath = '/test/plugin.js';
      
      expect(loader.isPluginLoaded(pluginPath)).toBe(false);
      
      // Simulate loading by adding to cache
      require.cache[path.resolve(pluginPath)] = {} as any;
      
      expect(loader.isPluginLoaded(pluginPath)).toBe(true);
    });

    it('should unload plugin from cache', () => {
      const pluginPath = '/test/plugin.js';
      const resolvedPath = path.resolve(pluginPath);
      
      // Add to cache
      require.cache[resolvedPath] = {} as any;
      expect(resolvedPath in require.cache).toBe(true);
      
      loader.unloadPlugin(pluginPath);
      expect(resolvedPath in require.cache).toBe(false);
    });

    it('should return loader statistics', () => {
      const stats = loader.getStats();
      
      expect(stats).toHaveProperty('cachedModules');
      expect(stats).toHaveProperty('options');
      expect(typeof stats.cachedModules).toBe('number');
    });
  });

  describe('error handling', () => {
    it('should provide detailed error messages for common issues', async () => {
      mockFs.stat.mockResolvedValue({
        isFile: () => true,
        isDirectory: () => false
      } as any);

      // Override the mock specifically for this test
      mockImport.mockImplementationOnce((path: string) => {
        if (path.includes('missing-deps.js')) {
          const error = new Error('Cannot find module \'missing-dependency\'');
          (error as any).code = 'MODULE_NOT_FOUND';
          return Promise.reject(error);
        }
        return Promise.reject(new Error(`Failed to load url ${path}`));
      });

      await expect(loader.loadPlugin('/test/fixtures/missing-deps.js')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.NOT_FOUND,
          message: expect.stringContaining('dependencies not found')
        })
      );
    });

    it('should handle directory without entry point', async () => {
      mockFs.stat
        .mockResolvedValueOnce({
          isFile: () => false,
          isDirectory: () => true
        } as any)
        .mockRejectedValue({ code: 'ENOENT' }); // No files found

      mockFs.readFile.mockRejectedValue(new Error('ENOENT')); // No package.json

      await expect(loader.loadPlugin('/test/fixtures/empty-dir')).rejects.toThrow(
        expect.objectContaining({
          code: PluginErrorCode.NOT_FOUND,
          message: expect.stringContaining('No entry point found')
        })
      );
    });
  });
});

describe('convenience functions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    mockFs.stat.mockResolvedValue({
      isFile: () => true,
      isDirectory: () => false
    } as any);

    mockImport.mockImplementation((path: string) => {
      if (path.includes('plugin.js')) {
        return Promise.resolve({ default: validFunctionalPlugin });
      }
      return Promise.reject(new Error(`Failed to load url ${path}`));
    });
  });

  it('should load single plugin using convenience function', async () => {
    const result = await loadPlugin('/test/plugin.js');
    
    expect(result.plugin).toBeDefined();
    expect(result.plugin.meta?.name).toBe('Test Plugin');
  });

  it('should load multiple plugins using convenience function', async () => {
    const results = await loadPlugins(['/test/plugin.js']);
    
    expect(results).toHaveLength(1);
    expect(results[0].plugin.meta?.name).toBe('Test Plugin');
  });

  it('should use default loader instance', () => {
    expect(defaultPluginLoader).toBeInstanceOf(PluginLoader);
  });
});

describe('integration scenarios', () => {
  let loader: PluginLoader;

  beforeEach(() => {
    loader = new PluginLoader();
    vi.clearAllMocks();
  });

  it('should handle complete plugin lifecycle', async () => {
    // Mock a complete functional plugin
    const lifecyclePlugin = definePlugin({
      meta: {
        name: 'Lifecycle Plugin',
        version: '2.0.0',
        description: 'Tests complete lifecycle',
        author: 'Test Suite'
      },
      onActivate: async (context) => {
        await context.app.showNotification('Lifecycle activated');
        context.events.emit('plugin:ready', { name: 'Lifecycle Plugin' });
      },
      onDeactivate: async () => {
        console.log('Lifecycle deactivated');
      }
    });

    mockFs.stat.mockResolvedValue({
      isFile: () => true,
      isDirectory: () => false
    } as any);

    mockImport.mockImplementation((path: string) => {
      if (path.includes('lifecycle-plugin.js')) {
        return Promise.resolve({ default: lifecyclePlugin });
      }
      return Promise.reject(new Error(`Failed to load url ${path}`));
    });

    // Load plugin with context
    const result = await loader.loadPlugin('/test/lifecycle-plugin.js', mockContext);

    // Verify plugin structure
    expect(result.plugin.meta?.name).toBe('Lifecycle Plugin');
    expect(result.plugin.onActivate).toBeDefined();
    expect(result.plugin.onDeactivate).toBeDefined();

    // Test activation
    await result.plugin.onActivate!();
    expect(mockContext.app.showNotification).toHaveBeenCalledWith('Lifecycle activated');
    expect(mockContext.events.emit).toHaveBeenCalledWith('plugin:ready', { name: 'Lifecycle Plugin' });

    // Test deactivation
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    await result.plugin.onDeactivate!();
    expect(consoleSpy).toHaveBeenCalledWith('Lifecycle deactivated');
    consoleSpy.mockRestore();
  });

  it('should handle plugin with only onDeactivate', async () => {
    const deactivateOnlyPlugin: Plugin = {
      meta: {
        name: 'Deactivate Only',
        version: '1.0.0'
      },
      onDeactivate: async () => {
        console.log('Only deactivate');
      }
    };

    mockFs.stat.mockResolvedValue({
      isFile: () => true,
      isDirectory: () => false
    } as any);

    mockImport.mockImplementation((path: string) => {
      if (path.includes('deactivate-only.js')) {
        return Promise.resolve({ default: deactivateOnlyPlugin });
      }
      return Promise.reject(new Error(`Failed to load url ${path}`));
    });

    const result = await loader.loadPlugin('/test/deactivate-only.js');

    expect(result.plugin.onActivate).toBeUndefined();
    expect(result.plugin.onDeactivate).toBeDefined();
    expect(result.plugin.meta?.name).toBe('Deactivate Only');
  });

  it('should handle concurrent plugin loading', async () => {
    mockFs.stat.mockResolvedValue({
      isFile: () => true,
      isDirectory: () => false
    } as any);

    // Setup mock for concurrent plugins
    mockImport.mockImplementation((path: string) => {
      const match = path.match(/concurrent-(\d+)\.js/);
      if (match) {
        const i = parseInt(match[1]);
        return Promise.resolve({
          default: definePlugin({
            meta: { name: `Concurrent Plugin ${i}` },
            onActivate: async () => console.log(`Plugin ${i} activated`)
          })
        });
      }
      return Promise.reject(new Error(`Failed to load url ${path}`));
    });

    const pluginPaths = Array.from({ length: 5 }, (_, i) => `/test/concurrent-${i + 1}.js`);
    
    const results = await loader.loadPlugins(pluginPaths, mockContext);

    expect(results).toHaveLength(5);
    results.forEach((result, index) => {
      expect(result.plugin.meta?.name).toBe(`Concurrent Plugin ${index + 1}`);
    });
  });
});