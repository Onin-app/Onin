/**
 * Functional Plugin Definition
 * 
 * This module provides the core `definePlugin` function for creating functional-style plugins.
 * It handles plugin metadata validation, default value setting, and type checking.
 */

import { Plugin, PluginDefinition, PluginMeta, PluginError, PluginErrorCode } from './types';

/**
 * Default plugin metadata values
 */
const DEFAULT_PLUGIN_META: Required<PluginMeta> = {
  name: 'Unnamed Plugin',
  version: '1.0.0',
  description: 'A Baize plugin',
  author: 'Unknown'
};

/**
 * Validates plugin metadata
 * @param meta - Plugin metadata to validate
 * @returns Validated and normalized metadata
 * @throws PluginError if validation fails
 */
function validatePluginMeta(meta?: PluginMeta): PluginMeta {
  if (!meta) {
    return { ...DEFAULT_PLUGIN_META };
  }

  const validated: PluginMeta = { ...DEFAULT_PLUGIN_META, ...meta };

  // Validate name
  if (validated.name && typeof validated.name !== 'string') {
    throw new PluginError(
      'Plugin name must be a string',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  // Validate version format (basic semver check)
  if (validated.version && typeof validated.version === 'string') {
    const semverRegex = /^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$/;
    if (!semverRegex.test(validated.version)) {
      throw new PluginError(
        `Invalid version format: ${validated.version}. Expected semantic version (e.g., 1.0.0)`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }
  }

  // Validate description
  if (validated.description && typeof validated.description !== 'string') {
    throw new PluginError(
      'Plugin description must be a string',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  // Validate author
  if (validated.author && typeof validated.author !== 'string') {
    throw new PluginError(
      'Plugin author must be a string',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  return validated;
}

/**
 * Validates plugin definition structure
 * @param definition - Plugin definition to validate
 * @throws PluginError if validation fails
 */
function validatePluginDefinition(definition: PluginDefinition): void {
  if (!definition || typeof definition !== 'object') {
    throw new PluginError(
      'Plugin definition must be an object',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  // Validate onActivate callback
  if (definition.onActivate !== undefined && typeof definition.onActivate !== 'function') {
    throw new PluginError(
      'onActivate must be a function',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  // Validate onDeactivate callback
  if (definition.onDeactivate !== undefined && typeof definition.onDeactivate !== 'function') {
    throw new PluginError(
      'onDeactivate must be a function',
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  // Validate that at least one lifecycle function is provided
  if (!definition.onActivate && !definition.onDeactivate) {
    throw new PluginError(
      'Plugin must define at least one lifecycle function (onActivate or onDeactivate)',
      PluginErrorCode.INVALID_MANIFEST
    );
  }
}

/**
 * Creates a functional plugin wrapper that handles context management
 * @param originalActivate - Original activation function from definition
 * @param context - Plugin context to inject
 * @returns Wrapped activation function
 */
function createActivationWrapper(
  originalActivate: (context: any) => Promise<void> | void,
  context: any
): () => Promise<void> | void {
  return async () => {
    try {
      const result = originalActivate(context);
      if (result instanceof Promise) {
        await result;
      }
    } catch (error) {
      throw new PluginError(
        `Plugin activation failed: ${error instanceof Error ? error.message : String(error)}`,
        PluginErrorCode.ACTIVATION_FAILED
      );
    }
  };
}

/**
 * Creates a functional plugin wrapper for deactivation
 * @param originalDeactivate - Original deactivation function from definition
 * @returns Wrapped deactivation function
 */
function createDeactivationWrapper(
  originalDeactivate: () => Promise<void> | void
): () => Promise<void> | void {
  return async () => {
    try {
      const result = originalDeactivate();
      if (result instanceof Promise) {
        await result;
      }
    } catch (error) {
      throw new PluginError(
        `Plugin deactivation failed: ${error instanceof Error ? error.message : String(error)}`,
        PluginErrorCode.LIFECYCLE_ERROR
      );
    }
  };
}

/**
 * Define a functional plugin
 * 
 * This function creates a plugin object from a functional definition.
 * It validates the plugin metadata, sets default values, and provides
 * type checking and error handling.
 * 
 * @param definition - Plugin definition configuration
 * @returns Plugin object ready for use by the plugin system
 * 
 * @example
 * ```typescript
 * const myPlugin = definePlugin({
 *   meta: {
 *     name: 'My Plugin',
 *     version: '1.0.0',
 *     description: 'A sample plugin',
 *     author: 'Plugin Developer'
 *   },
 *   onActivate: async (context) => {
 *     await context.app.showNotification('Plugin activated!');
 *   },
 *   onDeactivate: async () => {
 *     console.log('Plugin deactivated');
 *   }
 * });
 * ```
 */
export function definePlugin(definition: PluginDefinition): Plugin {
  try {
    // Validate the plugin definition structure
    validatePluginDefinition(definition);

    // Validate and normalize metadata
    const validatedMeta = validatePluginMeta(definition.meta);

    // Create the plugin object
    const plugin: Plugin = {
      meta: validatedMeta
    };

    // Handle activation function
    if (definition.onActivate) {
      // Store the original function for context injection during activation
      (plugin as any)._originalOnActivate = definition.onActivate;
      
      // Create a placeholder that will be replaced during plugin loading
      plugin.onActivate = () => {
        throw new PluginError(
          'Plugin context not available. Plugin must be properly loaded by the plugin system.',
          PluginErrorCode.CONTEXT_NOT_AVAILABLE,
          validatedMeta.name
        );
      };
    }

    // Handle deactivation function
    if (definition.onDeactivate) {
      plugin.onDeactivate = createDeactivationWrapper(definition.onDeactivate);
    }

    return plugin;

  } catch (error) {
    if (error instanceof PluginError) {
      throw error;
    }
    
    throw new PluginError(
      `Failed to define plugin: ${error instanceof Error ? error.message : String(error)}`,
      PluginErrorCode.INVALID_MANIFEST
    );
  }
}

/**
 * Utility function to inject context into a plugin's activation function
 * This is used internally by the plugin system during plugin loading
 * 
 * @param plugin - Plugin object created by definePlugin
 * @param context - Plugin context to inject
 * @returns Plugin with context-aware activation function
 */
export function injectPluginContext(plugin: Plugin, context: any): Plugin {
  const originalActivate = (plugin as any)._originalOnActivate;
  
  if (originalActivate) {
    return {
      ...plugin,
      onActivate: createActivationWrapper(originalActivate, context)
    };
  }
  
  return plugin;
}

/**
 * Utility function to check if an object is a valid plugin
 * @param obj - Object to check
 * @returns True if object is a valid plugin
 */
export function isValidPlugin(obj: any): obj is Plugin {
  if (!obj || typeof obj !== 'object') {
    return false;
  }

  // Check if it has at least one lifecycle function
  const hasActivate = typeof obj.onActivate === 'function';
  const hasDeactivate = typeof obj.onDeactivate === 'function';
  
  if (!hasActivate && !hasDeactivate) {
    return false;
  }

  // Check metadata structure if present
  if (obj.meta) {
    if (typeof obj.meta !== 'object') {
      return false;
    }
    
    // Validate meta fields if they exist
    const meta = obj.meta;
    if (meta.name !== undefined && typeof meta.name !== 'string') {
      return false;
    }
    if (meta.version !== undefined && typeof meta.version !== 'string') {
      return false;
    }
    if (meta.description !== undefined && typeof meta.description !== 'string') {
      return false;
    }
    if (meta.author !== undefined && typeof meta.author !== 'string') {
      return false;
    }
  }

  return true;
}

/**
 * Utility function to get plugin information for debugging
 * @param plugin - Plugin object
 * @returns Plugin information string
 */
export function getPluginInfo(plugin: Plugin): string {
  const meta = plugin.meta || {};
  const name = meta.name || 'Unknown Plugin';
  const version = meta.version || 'Unknown Version';
  const author = meta.author || 'Unknown Author';
  
  return `${name} v${version} by ${author}`;
}