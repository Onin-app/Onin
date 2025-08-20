/**
 * Simplified Plugin Loader for Functional Plugins
 * 
 * This module provides a simplified plugin loader that only supports functional-style plugins.
 * It removes complex type detection and focuses on loading and validating functional plugins.
 */

import { Plugin, PluginError, PluginErrorCode, PluginContext } from './types';
import { isValidPlugin, injectPluginContext } from './plugin';
import * as path from 'path';
import * as fs from 'fs/promises';

/**
 * Plugin loading result interface
 */
export interface PluginLoadResult {
  /** Loaded plugin instance */
  plugin: Plugin;
  /** Plugin file path */
  path: string;
  /** Plugin module exports */
  module: any;
}

/**
 * Plugin loader options
 */
export interface PluginLoaderOptions {
  /** Whether to validate plugin structure strictly */
  strictValidation?: boolean;
  /** Maximum time to wait for plugin loading (ms) */
  timeout?: number;
}

/**
 * Simplified Plugin Loader
 * 
 * This loader is designed specifically for functional plugins and removes
 * the complexity of supporting multiple plugin formats. It focuses on:
 * - Loading ES modules and CommonJS modules
 * - Validating functional plugin structure
 * - Providing clear error messages
 * - Supporting context injection
 */
export class PluginLoader {
  private options: Required<PluginLoaderOptions>;

  constructor(options: PluginLoaderOptions = {}) {
    this.options = {
      strictValidation: true,
      timeout: 5000,
      ...options
    };
  }

  /**
   * Load a plugin from the specified path
   * 
   * @param pluginPath - Path to the plugin file or directory
   * @param context - Optional plugin context to inject
   * @returns Promise resolving to loaded plugin result
   * @throws PluginError if loading fails
   */
  async loadPlugin(pluginPath: string, context?: PluginContext): Promise<PluginLoadResult> {
    try {
      // Normalize and validate the plugin path
      const normalizedPath = await this.validatePluginPath(pluginPath);
      
      // Load the plugin module with timeout
      const module = await this.loadPluginModule(normalizedPath);
      
      // Extract plugin from module exports
      const plugin = this.extractPluginFromModule(module, normalizedPath);
      
      // Validate plugin structure
      this.validatePluginStructure(plugin, normalizedPath);
      
      // Inject context if provided
      const finalPlugin = context ? injectPluginContext(plugin, context) : plugin;
      
      return {
        plugin: finalPlugin,
        path: normalizedPath,
        module
      };
      
    } catch (error) {
      if (error instanceof PluginError) {
        throw error;
      }
      
      throw new PluginError(
        `Failed to load plugin from ${pluginPath}: ${error instanceof Error ? error.message : String(error)}`,
        PluginErrorCode.LOAD_FAILED
      );
    }
  }

  /**
   * Load multiple plugins from an array of paths
   * 
   * @param pluginPaths - Array of plugin paths to load
   * @param context - Optional plugin context to inject
   * @returns Promise resolving to array of loaded plugin results
   */
  async loadPlugins(pluginPaths: string[], context?: PluginContext): Promise<PluginLoadResult[]> {
    const results: PluginLoadResult[] = [];
    const errors: Array<{ path: string; error: Error }> = [];

    // Load plugins in parallel
    const loadPromises = pluginPaths.map(async (pluginPath) => {
      try {
        const result = await this.loadPlugin(pluginPath, context);
        results.push(result);
      } catch (error) {
        errors.push({
          path: pluginPath,
          error: error instanceof Error ? error : new Error(String(error))
        });
      }
    });

    await Promise.all(loadPromises);

    // If there were errors, log them but don't fail the entire operation
    if (errors.length > 0) {
      console.warn(`Failed to load ${errors.length} plugins:`, errors);
    }

    return results;
  }

  /**
   * Validate and normalize plugin path
   * 
   * @param pluginPath - Raw plugin path
   * @returns Normalized absolute path
   * @throws PluginError if path is invalid
   */
  private async validatePluginPath(pluginPath: string): Promise<string> {
    if (!pluginPath || typeof pluginPath !== 'string') {
      throw new PluginError(
        'Plugin path must be a non-empty string',
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    // Resolve to absolute path
    const absolutePath = path.resolve(pluginPath);
    
    try {
      const stats = await fs.stat(absolutePath);
      
      if (stats.isDirectory()) {
        // If it's a directory, look for index.js, index.ts, or package.json main field
        const indexPath = await this.findPluginEntryPoint(absolutePath);
        return indexPath;
      } else if (stats.isFile()) {
        // If it's a file, use it directly
        return absolutePath;
      } else {
        throw new PluginError(
          `Plugin path is neither a file nor a directory: ${pluginPath}`,
          PluginErrorCode.NOT_FOUND
        );
      }
    } catch (error) {
      if ((error as any).code === 'ENOENT') {
        throw new PluginError(
          `Plugin not found: ${pluginPath}`,
          PluginErrorCode.NOT_FOUND
        );
      }
      throw error;
    }
  }

  /**
   * Find the entry point for a plugin directory
   * 
   * @param pluginDir - Plugin directory path
   * @returns Entry point file path
   * @throws PluginError if no entry point found
   */
  private async findPluginEntryPoint(pluginDir: string): Promise<string> {
    // Check for package.json main field
    const packageJsonPath = path.join(pluginDir, 'package.json');
    try {
      const packageJson = JSON.parse(await fs.readFile(packageJsonPath, 'utf-8'));
      if (packageJson.main) {
        const mainPath = path.resolve(pluginDir, packageJson.main);
        await fs.stat(mainPath); // Verify file exists
        return mainPath;
      }
    } catch {
      // package.json doesn't exist or is invalid, continue with default checks
    }

    // Check for common entry point files
    const entryPoints = ['index.js', 'index.ts', 'index.mjs', 'main.js', 'main.ts'];
    
    for (const entryPoint of entryPoints) {
      const entryPath = path.join(pluginDir, entryPoint);
      try {
        await fs.stat(entryPath);
        return entryPath;
      } catch {
        // File doesn't exist, try next
      }
    }

    throw new PluginError(
      `No entry point found in plugin directory: ${pluginDir}. Expected one of: ${entryPoints.join(', ')} or package.json main field`,
      PluginErrorCode.NOT_FOUND
    );
  }

  /**
   * Load plugin module with timeout support
   * 
   * @param pluginPath - Absolute path to plugin file
   * @returns Loaded module
   * @throws PluginError if loading fails or times out
   */
  private async loadPluginModule(pluginPath: string): Promise<any> {
    return new Promise(async (resolve, reject) => {
      // Set up timeout
      const timeoutId = setTimeout(() => {
        reject(new PluginError(
          `Plugin loading timed out after ${this.options.timeout}ms: ${pluginPath}`,
          PluginErrorCode.LOAD_FAILED
        ));
      }, this.options.timeout);

      try {
        // Clear module cache to ensure fresh load
        delete require.cache[pluginPath];
        
        // Load the module - use global import function for testability
        const importFn = (globalThis as any).import;
        const module = importFn ? await importFn(pluginPath) : await import(pluginPath);
        
        clearTimeout(timeoutId);
        resolve(module);
      } catch (error) {
        clearTimeout(timeoutId);
        
        // Provide more specific error messages for common issues
        if (error instanceof SyntaxError) {
          reject(new PluginError(
            `Plugin has syntax errors: ${error.message}`,
            PluginErrorCode.LOAD_FAILED
          ));
        } else if ((error as any).code === 'MODULE_NOT_FOUND') {
          reject(new PluginError(
            `Plugin module or its dependencies not found: ${error instanceof Error ? error.message : String(error)}`,
            PluginErrorCode.NOT_FOUND
          ));
        } else {
          reject(new PluginError(
            `Failed to load plugin module: ${error instanceof Error ? error.message : String(error)}`,
            PluginErrorCode.LOAD_FAILED
          ));
        }
      }
    });
  }

  /**
   * Extract plugin from module exports
   * 
   * @param module - Loaded module
   * @param pluginPath - Plugin path for error reporting
   * @returns Extracted plugin
   * @throws PluginError if no valid plugin found
   */
  private extractPluginFromModule(module: any, pluginPath: string): Plugin {
    // Try default export first
    if (module.default) {
      return module.default;
    }
    
    // Try named exports
    if (module.plugin) {
      return module.plugin;
    }
    
    // Try the module itself (for CommonJS)
    if (this.isValidPluginCandidate(module)) {
      return module;
    }
    
    // Look for any function-like exports that might be plugins
    const exportKeys = Object.keys(module);
    for (const key of exportKeys) {
      const candidate = module[key];
      if (this.isValidPluginCandidate(candidate)) {
        return candidate;
      }
    }
    
    throw new PluginError(
      `No valid functional plugin found in module: ${pluginPath}. Expected default export, named 'plugin' export, or object with onActivate/onDeactivate functions`,
      PluginErrorCode.INVALID_MANIFEST
    );
  }

  /**
   * Check if an object could be a valid plugin (basic check)
   * 
   * @param candidate - Object to check
   * @returns True if it looks like a plugin
   */
  private isValidPluginCandidate(candidate: any): boolean {
    return candidate && 
           typeof candidate === 'object' && 
           (typeof candidate.onActivate === 'function' || typeof candidate.onDeactivate === 'function');
  }

  /**
   * Validate plugin structure according to functional plugin requirements
   * 
   * @param plugin - Plugin to validate
   * @param pluginPath - Plugin path for error reporting
   * @throws PluginError if validation fails
   */
  private validatePluginStructure(plugin: Plugin, pluginPath: string): void {
    if (!this.options.strictValidation) {
      return;
    }

    if (!isValidPlugin(plugin)) {
      throw new PluginError(
        `Invalid functional plugin structure in ${pluginPath}. Plugin must be an object with at least one of: onActivate, onDeactivate functions`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    // Additional validation for functional plugins
    if (plugin.onActivate && typeof plugin.onActivate !== 'function') {
      throw new PluginError(
        `Plugin onActivate must be a function in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    if (plugin.onDeactivate && typeof plugin.onDeactivate !== 'function') {
      throw new PluginError(
        `Plugin onDeactivate must be a function in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    // Validate metadata if present
    if (plugin.meta) {
      this.validatePluginMeta(plugin.meta, pluginPath);
    }
  }

  /**
   * Validate plugin metadata
   * 
   * @param meta - Plugin metadata
   * @param pluginPath - Plugin path for error reporting
   * @throws PluginError if validation fails
   */
  private validatePluginMeta(meta: any, pluginPath: string): void {
    if (typeof meta !== 'object') {
      throw new PluginError(
        `Plugin meta must be an object in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    const { name, version, description, author } = meta;

    if (name !== undefined && typeof name !== 'string') {
      throw new PluginError(
        `Plugin meta.name must be a string in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    if (version !== undefined && typeof version !== 'string') {
      throw new PluginError(
        `Plugin meta.version must be a string in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    if (description !== undefined && typeof description !== 'string') {
      throw new PluginError(
        `Plugin meta.description must be a string in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }

    if (author !== undefined && typeof author !== 'string') {
      throw new PluginError(
        `Plugin meta.author must be a string in ${pluginPath}`,
        PluginErrorCode.INVALID_MANIFEST
      );
    }
  }

  /**
   * Check if a plugin is already loaded
   * 
   * @param pluginPath - Path to check
   * @returns True if plugin module is in cache
   */
  isPluginLoaded(pluginPath: string): boolean {
    const absolutePath = path.resolve(pluginPath);
    return absolutePath in require.cache;
  }

  /**
   * Unload a plugin from module cache
   * 
   * @param pluginPath - Path to plugin to unload
   */
  unloadPlugin(pluginPath: string): void {
    const absolutePath = path.resolve(pluginPath);
    delete require.cache[absolutePath];
  }

  /**
   * Get loader statistics
   * 
   * @returns Loader statistics
   */
  getStats(): { cachedModules: number; options: PluginLoaderOptions } {
    return {
      cachedModules: Object.keys(require.cache).length,
      options: this.options
    };
  }
}

/**
 * Default plugin loader instance
 */
export const defaultPluginLoader = new PluginLoader();

/**
 * Convenience function to load a single plugin
 * 
 * @param pluginPath - Path to plugin
 * @param context - Optional plugin context
 * @returns Promise resolving to loaded plugin result
 */
export async function loadPlugin(pluginPath: string, context?: PluginContext): Promise<PluginLoadResult> {
  return defaultPluginLoader.loadPlugin(pluginPath, context);
}

/**
 * Convenience function to load multiple plugins
 * 
 * @param pluginPaths - Array of plugin paths
 * @param context - Optional plugin context
 * @returns Promise resolving to array of loaded plugin results
 */
export async function loadPlugins(pluginPaths: string[], context?: PluginContext): Promise<PluginLoadResult[]> {
  return defaultPluginLoader.loadPlugins(pluginPaths, context);
}