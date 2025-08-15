/**
 * Baize Plugin SDK
 * 
 * This SDK provides the core interfaces and types for developing plugins
 * for the Baize application. It includes type definitions, API interfaces,
 * and event system definitions.
 * 
 * @version 0.1.0
 */

// Export all core types and interfaces
export * from './types';
export * from './api';
export * from './events';
export * from './communication';
export * from './context';

// Import specific classes for utility functions
import { PluginError, PluginErrorCode, type PluginManifest } from './types';

// SDK metadata
export const SDK_VERSION = '0.1.0';
export const SDK_NAME = '@baize/plugin-sdk';

/**
 * SDK information
 */
export const SDK_INFO = {
  name: SDK_NAME,
  version: SDK_VERSION,
  description: 'SDK for developing Baize plugins',
  author: 'Baize Team'
} as const;

/**
 * Utility function to validate plugin manifest
 * @param manifest - Plugin manifest to validate
 * @returns True if manifest is valid
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

/**
 * Utility function to create a plugin error
 * @param message - Error message
 * @param code - Error code
 * @param pluginName - Optional plugin name
 * @returns PluginError instance
 */
export function createPluginError(
  message: string, 
  code: PluginErrorCode, 
  pluginName?: string
): PluginError {
  return new PluginError(message, code, pluginName);
}