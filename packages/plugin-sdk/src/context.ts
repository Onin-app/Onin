/**
 * Plugin Context Management System
 * 
 * This module provides a global context management system for functional plugins.
 * It manages the current plugin context and provides utilities for context access.
 */

import { PluginContext, PluginError, PluginErrorCode } from './types';
import { EventEmitter } from './events';
import { createAPIImplementations } from './api';
import { CommunicationBridge, createCommunicationBridge, setGlobalBridge } from './communication';

/**
 * Configuration for creating a plugin context
 */
export interface PluginContextConfig {
  /** Unique plugin identifier */
  pluginId: string;
  /** Communication configuration */
  communication?: {
    maxRetries?: number;
    retryDelay?: number;
    timeout?: number;
    debug?: boolean;
  };
  /** Event system configuration */
  events?: {
    maxListeners?: number;
    debug?: boolean;
  };
}

/**
 * Global context manager for functional plugins
 * Manages the current plugin context during plugin execution
 */
class ContextManager {
  private currentContext: PluginContext | null = null;
  private currentPluginId: string | null = null;
  
  /**
   * Set the current plugin context
   * @param context - Plugin context to set as current
   * @param pluginId - ID of the plugin this context belongs to
   */
  setContext(context: PluginContext, pluginId?: string): void {
    this.currentContext = context;
    this.currentPluginId = pluginId || null;
  }
  
  /**
   * Get the current plugin context
   * @returns Current plugin context
   * @throws PluginError if no context is available
   */
  getContext(): PluginContext {
    if (!this.currentContext) {
      throw new PluginError(
        'Plugin context not available. Ensure you are calling this within a plugin lifecycle function.',
        PluginErrorCode.CONTEXT_NOT_AVAILABLE
      );
    }
    return this.currentContext;
  }
  
  /**
   * Check if a context is currently available
   * @returns True if context is available, false otherwise
   */
  hasContext(): boolean {
    return this.currentContext !== null;
  }
  
  /**
   * Get the current plugin ID
   * @returns Current plugin ID or null if not available
   */
  getCurrentPluginId(): string | null {
    return this.currentPluginId;
  }
  
  /**
   * Clear the current context
   */
  clearContext(): void {
    this.currentContext = null;
    this.currentPluginId = null;
  }
  
  /**
   * Execute a function with a specific context
   * @param context - Context to use during execution
   * @param pluginId - Plugin ID for the context
   * @param fn - Function to execute
   * @returns Result of the function execution
   */
  async withContext<T>(
    context: PluginContext, 
    pluginId: string, 
    fn: () => Promise<T> | T
  ): Promise<T> {
    const previousContext = this.currentContext;
    const previousPluginId = this.currentPluginId;
    
    try {
      this.setContext(context, pluginId);
      return await fn();
    } finally {
      // Restore previous context
      this.currentContext = previousContext;
      this.currentPluginId = previousPluginId;
    }
  }
}

/**
 * Global context manager instance
 */
export const contextManager = new ContextManager();

/**
 * Create a complete plugin context with all API implementations
 * @param config - Configuration for the plugin context
 * @returns Complete plugin context
 */
export function createPluginContext(config: PluginContextConfig): PluginContext {
  // Create communication bridge
  const bridge = createCommunicationBridge(config.pluginId, config.communication);
  
  // Set as global bridge for utility functions
  setGlobalBridge(bridge);
  
  // Create API implementations
  const apis = createAPIImplementations(bridge);
  
  // Create event manager
  const eventManager = new EventEmitter();
  
  // Return complete context
  return {
    app: apis.app,
    events: eventManager,
    storage: apis.storage
  };
}

/**
 * Utility function to get the current plugin context
 * This is a convenience function that wraps the context manager
 * @returns Current plugin context
 * @throws PluginError if no context is available
 */
export function getCurrentContext(): PluginContext {
  return contextManager.getContext();
}

/**
 * Utility function to check if a plugin context is available
 * @returns True if context is available, false otherwise
 */
export function hasCurrentContext(): boolean {
  return contextManager.hasContext();
}

/**
 * Plugin initialization helper
 * 
 * This class helps manage the plugin lifecycle and provides utilities
 * for common plugin operations. It integrates with the global context manager.
 */
export class PluginInitializer {
  private context: PluginContext | null = null;
  private bridge: CommunicationBridge | null = null;
  private config: PluginContextConfig;

  constructor(config: PluginContextConfig) {
    this.config = config;
  }

  /**
   * Initialize the plugin context
   * @returns Plugin context
   */
  async initialize(): Promise<PluginContext> {
    if (this.context) {
      return this.context;
    }

    // Check if Tauri is available
    if (typeof window === 'undefined' || !(window as any).__TAURI__) {
      throw new PluginError(
        'Plugin SDK requires Tauri environment',
        PluginErrorCode.LOAD_FAILED,
        this.config.pluginId
      );
    }

    try {
      // Create context
      this.context = createPluginContext(this.config);
      this.bridge = createCommunicationBridge(this.config.pluginId, this.config.communication);

      // Set context in global manager
      contextManager.setContext(this.context, this.config.pluginId);

      // Perform initial setup
      await this.performInitialSetup();

      return this.context;
    } catch (error) {
      throw new PluginError(
        `Failed to initialize plugin context: ${error instanceof Error ? error.message : 'Unknown error'}`,
        PluginErrorCode.LIFECYCLE_ERROR,
        this.config.pluginId
      );
    }
  }

  /**
   * Perform initial setup tasks
   */
  private async performInitialSetup(): Promise<void> {
    if (!this.bridge || !this.context) {
      return;
    }

    try {
      // Register plugin with the main application
      await this.bridge.invoke('register_plugin', {
        plugin_id: this.config.pluginId
      });

      // Verify required permissions
      // This could be expanded to check specific permissions
      // Note: hasPermission is not part of the current AppAPI interface
      // This is a placeholder for future permission checking
      try {
        // Basic connectivity test
        await this.context.app.getAppVersion();
      } catch (error) {
        console.warn(`[Plugin ${this.config.pluginId}] App API connectivity test failed:`, error);
      }

    } catch (error) {
      console.warn(`[Plugin ${this.config.pluginId}] Initial setup failed:`, error);
      // Don't throw here - allow plugin to continue with limited functionality
    }
  }

  /**
   * Clean up resources when plugin is deactivated
   */
  async cleanup(): Promise<void> {
    try {
      if (this.bridge) {
        try {
          await this.bridge.invoke('unregister_plugin', {
            plugin_id: this.config.pluginId
          });
        } catch (error) {
          console.warn(`[Plugin ${this.config.pluginId}] Cleanup failed:`, error);
        }
      }

      // Clear event listeners if using EventManager
      if (this.context?.events && 'removeAllListeners' in this.context.events) {
        (this.context.events as any).removeAllListeners();
      }

      // Clear context from global manager if it's the current one
      if (contextManager.getCurrentPluginId() === this.config.pluginId) {
        contextManager.clearContext();
      }

      this.context = null;
      this.bridge = null;
    } catch (error) {
      throw new PluginError(
        `Failed to cleanup plugin: ${error instanceof Error ? error.message : 'Unknown error'}`,
        PluginErrorCode.LIFECYCLE_ERROR,
        this.config.pluginId
      );
    }
  }

  /**
   * Get the current context (if initialized)
   */
  getContext(): PluginContext | null {
    return this.context;
  }

  /**
   * Check if the plugin is initialized
   */
  isInitialized(): boolean {
    return this.context !== null;
  }

  /**
   * Update plugin configuration
   * @param config - New configuration options
   */
  updateConfig(config: Partial<PluginContextConfig>): void {
    this.config = { ...this.config, ...config };
    
    if (this.bridge && config.communication) {
      this.bridge.updateConfig(config.communication);
    }
  }
}

/**
 * Utility function to create a simple plugin context for basic use cases
 * @param pluginId - Plugin identifier
 * @returns Plugin context
 */
export function createSimplePluginContext(pluginId: string): PluginContext {
  return createPluginContext({ pluginId });
}

/**
 * Global plugin initializer registry
 * Useful for managing multiple plugins or for debugging
 */
const pluginRegistry = new Map<string, PluginInitializer>();

/**
 * Register a plugin initializer
 * @param pluginId - Plugin identifier
 * @param initializer - Plugin initializer instance
 */
export function registerPlugin(pluginId: string, initializer: PluginInitializer): void {
  pluginRegistry.set(pluginId, initializer);
}

/**
 * Get a registered plugin initializer
 * @param pluginId - Plugin identifier
 * @returns Plugin initializer or undefined
 */
export function getPlugin(pluginId: string): PluginInitializer | undefined {
  return pluginRegistry.get(pluginId);
}

/**
 * Get all registered plugins
 * @returns Array of plugin IDs
 */
export function getRegisteredPlugins(): string[] {
  return Array.from(pluginRegistry.keys());
}

/**
 * Clean up all registered plugins
 */
export async function cleanupAllPlugins(): Promise<void> {
  const cleanupPromises = Array.from(pluginRegistry.values()).map(
    initializer => initializer.cleanup()
  );
  
  await Promise.allSettled(cleanupPromises);
  pluginRegistry.clear();
  
  // Clear global context
  contextManager.clearContext();
}

/**
 * Validate that a plugin context is properly formed
 * @param context - Context to validate
 * @throws PluginError if context is invalid
 */
export function validatePluginContext(context: any): asserts context is PluginContext {
  if (!context || typeof context !== 'object') {
    throw new PluginError(
      'Invalid plugin context: context must be an object',
      PluginErrorCode.CONTEXT_NOT_AVAILABLE
    );
  }
  
  if (!context.app || typeof context.app !== 'object') {
    throw new PluginError(
      'Invalid plugin context: missing or invalid app API',
      PluginErrorCode.CONTEXT_NOT_AVAILABLE
    );
  }
  
  if (!context.events || typeof context.events !== 'object') {
    throw new PluginError(
      'Invalid plugin context: missing or invalid events API',
      PluginErrorCode.CONTEXT_NOT_AVAILABLE
    );
  }
  
  if (!context.storage || typeof context.storage !== 'object') {
    throw new PluginError(
      'Invalid plugin context: missing or invalid storage API',
      PluginErrorCode.CONTEXT_NOT_AVAILABLE
    );
  }
}

/**
 * Execute a function with error handling for context operations
 * @param fn - Function to execute
 * @param pluginId - Optional plugin ID for error context
 * @returns Result of function execution
 */
export async function withContextErrorHandling<T>(
  fn: () => Promise<T> | T,
  pluginId?: string
): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    if (error instanceof PluginError) {
      throw error;
    }
    
    throw new PluginError(
      `Context operation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      PluginErrorCode.CONTEXT_NOT_AVAILABLE,
      pluginId
    );
  }
}