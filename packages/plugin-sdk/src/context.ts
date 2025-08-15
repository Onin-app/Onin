/**
 * Plugin Context Factory
 * 
 * This module provides utilities to create plugin contexts with proper API implementations
 * using the communication bridge.
 */

import { PluginContext } from './types';
import { EventManager } from './events';
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
  const eventManager = new EventManager(config.events);
  
  // Return complete context
  return {
    app: apis.app,
    events: eventManager,
    storage: apis.storage
  };
}

/**
 * Plugin initialization helper
 * 
 * This class helps manage the plugin lifecycle and provides utilities
 * for common plugin operations.
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
    if (typeof window === 'undefined' || !window.__TAURI__) {
      throw new Error('Plugin SDK requires Tauri environment');
    }

    // Create context
    this.context = createPluginContext(this.config);
    this.bridge = createCommunicationBridge(this.config.pluginId, this.config.communication);

    // Perform initial setup
    await this.performInitialSetup();

    return this.context;
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
      const hasBasicPermission = await this.context.app.hasPermission('basic');
      if (!hasBasicPermission) {
        console.warn(`[Plugin ${this.config.pluginId}] Basic permission not granted`);
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

    this.context = null;
    this.bridge = null;
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
}