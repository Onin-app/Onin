/**
 * Baize Plugin SDK - Functional API
 * 
 * This SDK provides a modern functional programming interface for developing plugins
 * for the Baize application. It includes functional plugin definitions, hook-style APIs,
 * and comprehensive type definitions with English comments.
 * 
 * @version 0.2.0
 */

// Core functional plugin API
export { definePlugin, injectPluginContext, isValidPlugin, getPluginInfo } from './plugin';

// Hook-style API functions
export {
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

// Context management system
export {
  contextManager,
  createPluginContext,
  createSimplePluginContext,
  getCurrentContext,
  hasCurrentContext,
  PluginInitializer,
  registerPlugin,
  getPlugin,
  getRegisteredPlugins,
  cleanupAllPlugins,
  validatePluginContext,
  withContextErrorHandling
} from './context';

// Lifecycle management
export {
  PluginLifecycleManager,
  defaultLifecycleManager,
  activatePlugin,
  deactivatePlugin,
  isPluginActive,
  getActivePlugins
} from './lifecycle';

// Plugin loader (simplified for functional plugins) - Node.js only
// Note: Loader functionality requires Node.js environment and is available as a separate import
// import { PluginLoader, loadPlugin } from '@baize/plugin-sdk/loader';

// Core types and interfaces
export type {
  Plugin,
  PluginDefinition,
  PluginMeta,
  PluginContext,
  AppAPI,
  EventAPI,
  StorageAPI,
  DialogOptions,
  EventHandler,
  PluginManifest,
  PluginInfo
} from './types';

// Enums and classes
export {
  PluginStatus,
  PluginError,
  PluginErrorCode
} from './types';

// Import for internal use
import { PluginError, PluginErrorCode, type PluginManifest } from './types';

// API implementations (for advanced use cases)
export * from './api';
export * from './events';
export * from './communication';

// SDK metadata
export const SDK_VERSION = '0.2.0';
export const SDK_NAME = '@baize/plugin-sdk';

/**
 * SDK information
 */
export const SDK_INFO = {
  name: SDK_NAME,
  version: SDK_VERSION,
  description: 'Functional SDK for developing Baize plugins',
  author: 'Baize Team'
} as const;

/**
 * Utility function to create a plugin error with proper context
 * @param message - Error message
 * @param code - Error code
 * @param pluginName - Optional plugin name for context
 * @returns PluginError instance
 */
export function createPluginError(
  message: string, 
  code: PluginErrorCode, 
  pluginName?: string
): PluginError {
  return new PluginError(message, code, pluginName);
}

/**
 * Utility function to validate plugin manifest (legacy support)
 * @param manifest - Plugin manifest to validate
 * @returns True if manifest is valid
 * @deprecated Use functional plugin definitions with definePlugin instead
 */
export function validatePluginManifest(manifest: any): manifest is PluginManifest {
  if (!manifest || typeof manifest !== 'object') {
    return false;
  }
  
  const required = ['name', 'version', 'description', 'author', 'main', 'permissions', 'engines'];
  
  for (const field of required) {
    if (!(field in manifest)) {
      return false;
    }
  }
  
  // Validate specific field types
  if (typeof manifest.name !== 'string' || !manifest.name.trim()) {
    return false;
  }
  
  if (typeof manifest.version !== 'string' || !manifest.version.trim()) {
    return false;
  }
  
  if (typeof manifest.description !== 'string') {
    return false;
  }
  
  if (typeof manifest.author !== 'string') {
    return false;
  }
  
  if (typeof manifest.main !== 'string' || !manifest.main.trim()) {
    return false;
  }
  
  if (!Array.isArray(manifest.permissions)) {
    return false;
  }
  
  if (!manifest.engines || typeof manifest.engines !== 'object') {
    return false;
  }
  
  if (typeof manifest.engines.baize !== 'string') {
    return false;
  }
  
  return true;
}