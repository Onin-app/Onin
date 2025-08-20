/**
 * Plugin Lifecycle Management System
 * 
 * This module provides the PluginLifecycleManager class that handles
 * plugin activation and deactivation with proper context management
 * and error handling.
 */

import { Plugin, PluginContext, PluginError, PluginErrorCode } from './types';
import { contextManager, createPluginContext, PluginContextConfig } from './context';
import { injectPluginContext } from './plugin';

/**
 * Configuration options for the lifecycle manager
 */
export interface LifecycleManagerConfig {
  /** Maximum time to wait for plugin activation/deactivation (ms) */
  timeout?: number;
  /** Whether to enable debug logging */
  debug?: boolean;
  /** Default context configuration for plugins */
  defaultContextConfig?: Partial<PluginContextConfig>;
}

/**
 * Plugin lifecycle state tracking
 */
interface PluginLifecycleState {
  /** Plugin instance */
  plugin: Plugin;
  /** Plugin context */
  context: PluginContext;
  /** Plugin identifier */
  pluginId: string;
  /** Whether plugin is currently active */
  isActive: boolean;
  /** Activation timestamp */
  activatedAt?: Date;
  /** Last error encountered */
  lastError?: Error;
}

/**
 * Plugin Lifecycle Manager
 * 
 * Manages the activation and deactivation of functional plugins with
 * proper context management, error handling, and state tracking.
 */
export class PluginLifecycleManager {
  private config: LifecycleManagerConfig;
  private pluginStates = new Map<string, PluginLifecycleState>();
  
  constructor(config: LifecycleManagerConfig = {}) {
    this.config = {
      timeout: 30000, // 30 seconds default timeout
      debug: false,
      ...config
    };
  }

  /**
   * Activate a plugin with proper context management
   * @param plugin - Plugin to activate
   * @param pluginId - Unique identifier for the plugin
   * @param contextConfig - Optional context configuration override
   * @returns Promise that resolves when plugin is activated
   */
  async activatePlugin(
    plugin: Plugin, 
    pluginId: string, 
    contextConfig?: Partial<PluginContextConfig>
  ): Promise<void> {
    this.debugLog(`Activating plugin: ${pluginId}`);

    // Check if plugin is already active
    const existingState = this.pluginStates.get(pluginId);
    if (existingState?.isActive) {
      throw new PluginError(
        `Plugin ${pluginId} is already active`,
        PluginErrorCode.LIFECYCLE_ERROR,
        pluginId
      );
    }

    try {
      // Create plugin context
      const fullContextConfig: PluginContextConfig = {
        pluginId,
        ...this.config.defaultContextConfig,
        ...contextConfig
      };
      
      const context = createPluginContext(fullContextConfig);

      // Inject context into plugin if needed
      const contextAwarePlugin = injectPluginContext(plugin, context);

      // Create lifecycle state
      const state: PluginLifecycleState = {
        plugin: contextAwarePlugin,
        context,
        pluginId,
        isActive: false
      };

      // Set context in global manager
      contextManager.setContext(context, pluginId);

      try {
        // Execute activation with timeout
        await this.executeWithTimeout(
          async () => {
            if (contextAwarePlugin.onActivate) {
              await contextAwarePlugin.onActivate();
            }
          },
          this.config.timeout!,
          `Plugin ${pluginId} activation timed out`
        );

        // Mark as active and store state
        state.isActive = true;
        state.activatedAt = new Date();
        this.pluginStates.set(pluginId, state);

        this.debugLog(`Plugin ${pluginId} activated successfully`);

      } catch (error) {
        // Clear context on activation failure
        contextManager.clearContext();
        
        // Store error state even on failure
        const pluginError = this.wrapError(error, pluginId, 'activation');
        state.lastError = pluginError;
        state.isActive = false;
        this.pluginStates.set(pluginId, state);
        
        throw pluginError;
      }

    } catch (error) {
      // This catch is for context creation errors
      const pluginError = this.wrapError(error, pluginId, 'activation');
      
      // Store error state if we have an existing state
      if (existingState) {
        existingState.lastError = pluginError;
        existingState.isActive = false;
      }
      
      throw pluginError;
    }
  }

  /**
   * Deactivate a plugin with proper cleanup
   * @param pluginId - Identifier of the plugin to deactivate
   * @returns Promise that resolves when plugin is deactivated
   */
  async deactivatePlugin(pluginId: string): Promise<void> {
    this.debugLog(`Deactivating plugin: ${pluginId}`);

    const state = this.pluginStates.get(pluginId);
    if (!state) {
      throw new PluginError(
        `Plugin ${pluginId} is not registered`,
        PluginErrorCode.NOT_FOUND,
        pluginId
      );
    }

    if (!state.isActive) {
      this.debugLog(`Plugin ${pluginId} is already inactive`);
      return;
    }

    try {
      // Set context for deactivation
      contextManager.setContext(state.context, pluginId);

      try {
        // Execute deactivation with timeout
        await this.executeWithTimeout(
          async () => {
            if (state.plugin.onDeactivate) {
              await state.plugin.onDeactivate();
            }
          },
          this.config.timeout!,
          `Plugin ${pluginId} deactivation timed out`
        );

        this.debugLog(`Plugin ${pluginId} deactivated successfully`);

      } finally {
        // Always clear context and mark as inactive
        contextManager.clearContext();
        state.isActive = false;
        state.activatedAt = undefined;
      }

    } catch (error) {
      const pluginError = this.wrapError(error, pluginId, 'deactivation');
      state.lastError = pluginError;
      state.isActive = false;
      
      throw pluginError;
    }
  }

  /**
   * Check if a plugin is currently active
   * @param pluginId - Plugin identifier
   * @returns True if plugin is active, false otherwise
   */
  isPluginActive(pluginId: string): boolean {
    const state = this.pluginStates.get(pluginId);
    return state?.isActive ?? false;
  }

  /**
   * Get the context for an active plugin
   * @param pluginId - Plugin identifier
   * @returns Plugin context or null if not active
   */
  getPluginContext(pluginId: string): PluginContext | null {
    const state = this.pluginStates.get(pluginId);
    return state?.isActive ? state.context : null;
  }

  /**
   * Get information about a plugin's lifecycle state
   * @param pluginId - Plugin identifier
   * @returns Plugin lifecycle information or null if not found
   */
  getPluginState(pluginId: string): Readonly<PluginLifecycleState> | null {
    const state = this.pluginStates.get(pluginId);
    return state ? { ...state } : null;
  }

  /**
   * Get all registered plugin IDs
   * @returns Array of plugin identifiers
   */
  getRegisteredPlugins(): string[] {
    return Array.from(this.pluginStates.keys());
  }

  /**
   * Get all active plugin IDs
   * @returns Array of active plugin identifiers
   */
  getActivePlugins(): string[] {
    return Array.from(this.pluginStates.entries())
      .filter(([, state]) => state.isActive)
      .map(([pluginId]) => pluginId);
  }

  /**
   * Deactivate all active plugins
   * @returns Promise that resolves when all plugins are deactivated
   */
  async deactivateAllPlugins(): Promise<void> {
    const activePlugins = this.getActivePlugins();
    
    if (activePlugins.length === 0) {
      this.debugLog('No active plugins to deactivate');
      return;
    }

    this.debugLog(`Deactivating ${activePlugins.length} active plugins`);

    // Deactivate plugins in parallel but handle errors individually
    const deactivationPromises = activePlugins.map(async (pluginId) => {
      try {
        await this.deactivatePlugin(pluginId);
      } catch (error) {
        console.error(`Failed to deactivate plugin ${pluginId}:`, error);
        // Continue with other plugins even if one fails
      }
    });

    await Promise.allSettled(deactivationPromises);
    this.debugLog('All plugins deactivation completed');
  }

  /**
   * Remove a plugin from the lifecycle manager
   * @param pluginId - Plugin identifier
   * @returns True if plugin was removed, false if not found
   */
  async removePlugin(pluginId: string): Promise<boolean> {
    const state = this.pluginStates.get(pluginId);
    if (!state) {
      return false;
    }

    // Deactivate if active
    if (state.isActive) {
      try {
        await this.deactivatePlugin(pluginId);
      } catch (error) {
        console.error(`Error deactivating plugin ${pluginId} during removal:`, error);
      }
    }

    // Remove from state tracking
    this.pluginStates.delete(pluginId);
    this.debugLog(`Plugin ${pluginId} removed from lifecycle manager`);
    
    return true;
  }

  /**
   * Update configuration for the lifecycle manager
   * @param config - New configuration options
   */
  updateConfig(config: Partial<LifecycleManagerConfig>): void {
    this.config = { ...this.config, ...config };
    this.debugLog('Lifecycle manager configuration updated');
  }

  /**
   * Get current configuration
   * @returns Current configuration
   */
  getConfig(): Readonly<LifecycleManagerConfig> {
    return { ...this.config };
  }

  /**
   * Execute a function with a timeout
   * @param fn - Function to execute
   * @param timeoutMs - Timeout in milliseconds
   * @param timeoutMessage - Error message for timeout
   * @returns Promise that resolves with function result or rejects on timeout
   */
  private async executeWithTimeout<T>(
    fn: () => Promise<T>,
    timeoutMs: number,
    timeoutMessage: string
  ): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new PluginError(timeoutMessage, PluginErrorCode.LIFECYCLE_ERROR));
      }, timeoutMs);

      fn()
        .then((result) => {
          clearTimeout(timeoutId);
          resolve(result);
        })
        .catch((error) => {
          clearTimeout(timeoutId);
          reject(error);
        });
    });
  }

  /**
   * Wrap an error with plugin context information
   * @param error - Original error
   * @param pluginId - Plugin identifier
   * @param operation - Operation that failed
   * @returns Wrapped PluginError
   */
  private wrapError(error: unknown, pluginId: string, operation: string): PluginError {
    if (error instanceof PluginError) {
      return error;
    }

    const message = error instanceof Error ? error.message : String(error);
    return new PluginError(
      `Plugin ${operation} failed: ${message}`,
      PluginErrorCode.LIFECYCLE_ERROR,
      pluginId
    );
  }

  /**
   * Debug logging utility
   * @param message - Message to log
   */
  private debugLog(message: string): void {
    if (this.config.debug) {
      console.log(`[PluginLifecycleManager] ${message}`);
    }
  }
}

/**
 * Default global lifecycle manager instance
 */
export const defaultLifecycleManager = new PluginLifecycleManager();

/**
 * Utility function to activate a plugin using the default manager
 * @param plugin - Plugin to activate
 * @param pluginId - Plugin identifier
 * @param contextConfig - Optional context configuration
 * @returns Promise that resolves when plugin is activated
 */
export async function activatePlugin(
  plugin: Plugin,
  pluginId: string,
  contextConfig?: Partial<PluginContextConfig>
): Promise<void> {
  return defaultLifecycleManager.activatePlugin(plugin, pluginId, contextConfig);
}

/**
 * Utility function to deactivate a plugin using the default manager
 * @param pluginId - Plugin identifier
 * @returns Promise that resolves when plugin is deactivated
 */
export async function deactivatePlugin(pluginId: string): Promise<void> {
  return defaultLifecycleManager.deactivatePlugin(pluginId);
}

/**
 * Utility function to check if a plugin is active using the default manager
 * @param pluginId - Plugin identifier
 * @returns True if plugin is active
 */
export function isPluginActive(pluginId: string): boolean {
  return defaultLifecycleManager.isPluginActive(pluginId);
}

/**
 * Utility function to get all active plugins using the default manager
 * @returns Array of active plugin identifiers
 */
export function getActivePlugins(): string[] {
  return defaultLifecycleManager.getActivePlugins();
}