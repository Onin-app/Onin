/**
 * Unit tests for the definePlugin function and related utilities
 */

import { describe, it, expect, vi } from 'vitest';
import { 
  definePlugin, 
  injectPluginContext, 
  isValidPlugin, 
  getPluginInfo 
} from './plugin';
import { PluginError, PluginErrorCode, PluginDefinition, Plugin } from './types';

describe('definePlugin', () => {
  it('should create a valid plugin with minimal definition', () => {
    const plugin = definePlugin({
      onActivate: async () => {
        console.log('Plugin activated');
      }
    });

    expect(plugin).toBeDefined();
    expect(plugin.onActivate).toBeDefined();
    expect(plugin.meta).toBeDefined();
    expect(plugin.meta?.name).toBe('Unnamed Plugin');
    expect(plugin.meta?.version).toBe('1.0.0');
    expect(plugin.meta?.description).toBe('A Baize plugin');
    expect(plugin.meta?.author).toBe('Unknown');
  });

  it('should create a plugin with custom metadata', () => {
    const definition: PluginDefinition = {
      meta: {
        name: 'Test Plugin',
        version: '2.1.0',
        description: 'A test plugin for unit testing',
        author: 'Test Author'
      },
      onActivate: async () => {
        console.log('Test plugin activated');
      }
    };

    const plugin = definePlugin(definition);

    expect(plugin.meta?.name).toBe('Test Plugin');
    expect(plugin.meta?.version).toBe('2.1.0');
    expect(plugin.meta?.description).toBe('A test plugin for unit testing');
    expect(plugin.meta?.author).toBe('Test Author');
  });

  it('should create a plugin with both activation and deactivation functions', () => {
    const onActivate = vi.fn();
    const onDeactivate = vi.fn();

    const plugin = definePlugin({
      onActivate,
      onDeactivate
    });

    expect(plugin.onActivate).toBeDefined();
    expect(plugin.onDeactivate).toBeDefined();
  });

  it('should create a plugin with only deactivation function', () => {
    const onDeactivate = vi.fn();

    const plugin = definePlugin({
      onDeactivate
    });

    expect(plugin.onActivate).toBeUndefined();
    expect(plugin.onDeactivate).toBeDefined();
  });

  it('should merge partial metadata with defaults', () => {
    const plugin = definePlugin({
      meta: {
        name: 'Partial Plugin',
        version: '1.5.0'
        // description and author should use defaults
      },
      onActivate: async () => {}
    });

    expect(plugin.meta?.name).toBe('Partial Plugin');
    expect(plugin.meta?.version).toBe('1.5.0');
    expect(plugin.meta?.description).toBe('A Baize plugin');
    expect(plugin.meta?.author).toBe('Unknown');
  });

  it('should validate semantic version format', () => {
    expect(() => {
      definePlugin({
        meta: {
          version: 'invalid-version'
        },
        onActivate: async () => {}
      });
    }).toThrow(PluginError);

    expect(() => {
      definePlugin({
        meta: {
          version: '1.0'
        },
        onActivate: async () => {}
      });
    }).toThrow(PluginError);

    // Valid versions should not throw
    expect(() => {
      definePlugin({
        meta: {
          version: '1.0.0'
        },
        onActivate: async () => {}
      });
    }).not.toThrow();

    expect(() => {
      definePlugin({
        meta: {
          version: '1.0.0-alpha.1'
        },
        onActivate: async () => {}
      });
    }).not.toThrow();

    expect(() => {
      definePlugin({
        meta: {
          version: '1.0.0+build.1'
        },
        onActivate: async () => {}
      });
    }).not.toThrow();
  });

  it('should throw error for invalid definition structure', () => {
    expect(() => {
      definePlugin(null as any);
    }).toThrow(PluginError);

    expect(() => {
      definePlugin('invalid' as any);
    }).toThrow(PluginError);

    expect(() => {
      definePlugin({} as any);
    }).toThrow(PluginError);
  });

  it('should throw error for invalid lifecycle functions', () => {
    expect(() => {
      definePlugin({
        onActivate: 'not a function' as any
      });
    }).toThrow(PluginError);

    expect(() => {
      definePlugin({
        onDeactivate: 123 as any
      });
    }).toThrow(PluginError);
  });

  it('should throw error for invalid metadata types', () => {
    expect(() => {
      definePlugin({
        meta: {
          name: 123 as any
        },
        onActivate: async () => {}
      });
    }).toThrow(PluginError);

    expect(() => {
      definePlugin({
        meta: {
          description: [] as any
        },
        onActivate: async () => {}
      });
    }).toThrow(PluginError);

    expect(() => {
      definePlugin({
        meta: {
          author: {} as any
        },
        onActivate: async () => {}
      });
    }).toThrow(PluginError);
  });

  it('should throw context error when activation is called without context injection', async () => {
    const plugin = definePlugin({
      onActivate: async () => {
        console.log('Should not reach here');
      }
    });

    await expect(async () => {
      await plugin.onActivate?.();
    }).rejects.toThrow(PluginError);
  });
});

describe('injectPluginContext', () => {
  it('should inject context into plugin activation function', async () => {
    const mockContext = {
      app: { showNotification: vi.fn() },
      events: { on: vi.fn(), emit: vi.fn(), off: vi.fn() },
      storage: { get: vi.fn(), set: vi.fn() }
    };

    const activationSpy = vi.fn();
    const originalPlugin = definePlugin({
      onActivate: activationSpy
    });

    const pluginWithContext = injectPluginContext(originalPlugin, mockContext);
    
    await pluginWithContext.onActivate?.();
    
    expect(activationSpy).toHaveBeenCalledWith(mockContext);
  });

  it('should handle plugins without activation function', () => {
    const mockContext = {};
    const originalPlugin = definePlugin({
      onDeactivate: async () => {}
    });

    const pluginWithContext = injectPluginContext(originalPlugin, mockContext);
    
    expect(pluginWithContext.onActivate).toBeUndefined();
    expect(pluginWithContext.onDeactivate).toBeDefined();
  });

  it('should handle activation errors properly', async () => {
    const mockContext = {};
    const errorMessage = 'Test activation error';
    
    const originalPlugin = definePlugin({
      onActivate: async () => {
        throw new Error(errorMessage);
      }
    });

    const pluginWithContext = injectPluginContext(originalPlugin, mockContext);
    
    await expect(pluginWithContext.onActivate?.()).rejects.toThrow(PluginError);
    await expect(pluginWithContext.onActivate?.()).rejects.toThrow(errorMessage);
  });
});

describe('isValidPlugin', () => {
  it('should return true for valid plugins', () => {
    const validPlugin = definePlugin({
      onActivate: async () => {}
    });

    expect(isValidPlugin(validPlugin)).toBe(true);
  });

  it('should return true for plugins with only deactivation', () => {
    const validPlugin = definePlugin({
      onDeactivate: async () => {}
    });

    expect(isValidPlugin(validPlugin)).toBe(true);
  });

  it('should return false for invalid objects', () => {
    expect(isValidPlugin(null)).toBe(false);
    expect(isValidPlugin(undefined)).toBe(false);
    expect(isValidPlugin('string')).toBe(false);
    expect(isValidPlugin(123)).toBe(false);
    expect(isValidPlugin([])).toBe(false);
  });

  it('should return false for objects without lifecycle functions', () => {
    expect(isValidPlugin({})).toBe(false);
    expect(isValidPlugin({ meta: { name: 'Test' } })).toBe(false);
  });

  it('should return false for objects with invalid lifecycle functions', () => {
    expect(isValidPlugin({ onActivate: 'not a function' })).toBe(false);
    expect(isValidPlugin({ onDeactivate: 123 })).toBe(false);
  });

  it('should return false for objects with invalid metadata', () => {
    expect(isValidPlugin({
      onActivate: () => {},
      meta: 'invalid meta'
    })).toBe(false);

    expect(isValidPlugin({
      onActivate: () => {},
      meta: { name: 123 }
    })).toBe(false);

    expect(isValidPlugin({
      onActivate: () => {},
      meta: { version: [] }
    })).toBe(false);
  });
});

describe('getPluginInfo', () => {
  it('should return formatted plugin information', () => {
    const plugin = definePlugin({
      meta: {
        name: 'Test Plugin',
        version: '1.2.3',
        author: 'Test Author'
      },
      onActivate: async () => {}
    });

    const info = getPluginInfo(plugin);
    expect(info).toBe('Test Plugin v1.2.3 by Test Author');
  });

  it('should handle missing metadata gracefully', () => {
    const plugin = definePlugin({
      onActivate: async () => {}
    });

    // Clear meta to test undefined case
    (plugin as any).meta = undefined;

    const info = getPluginInfo(plugin);
    expect(info).toBe('Unknown Plugin vUnknown Version by Unknown Author');
  });

  it('should handle partial metadata', () => {
    const plugin = definePlugin({
      meta: {
        name: 'Partial Plugin'
        // version and author missing
      },
      onActivate: async () => {}
    });

    const info = getPluginInfo(plugin);
    expect(info).toBe('Partial Plugin v1.0.0 by Unknown');
  });
});

describe('error handling', () => {
  it('should throw PluginError with correct error codes', () => {
    try {
      definePlugin({
        onActivate: 'invalid' as any
      });
    } catch (error) {
      expect(error).toBeInstanceOf(PluginError);
      expect((error as PluginError).code).toBe(PluginErrorCode.INVALID_MANIFEST);
    }
  });

  it('should wrap unknown errors in PluginError', () => {
    // Test by passing an object that will cause an unexpected error during processing
    const invalidDefinition = {
      onActivate: async () => {},
      meta: {
        get version(): string {
          throw new TypeError('Unexpected error during property access');
        }
      }
    } as any; // Use 'as any' to bypass TypeScript checking for this test

    expect(() => {
      definePlugin(invalidDefinition);
    }).toThrow(PluginError);
  });

  it('should handle deactivation errors properly', async () => {
    const plugin = definePlugin({
      onDeactivate: async () => {
        throw new Error('Deactivation failed');
      }
    });

    await expect(plugin.onDeactivate?.()).rejects.toThrow(PluginError);
    await expect(plugin.onDeactivate?.()).rejects.toThrow('Deactivation failed');
  });
});