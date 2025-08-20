/**
 * Hook-style API functions for functional plugins
 * 
 * This module provides React-style hooks for accessing plugin APIs and managing
 * plugin lifecycle in a functional programming style.
 */

import { AppAPI, EventAPI, StorageAPI, PluginContext, PluginError, PluginErrorCode } from './types';
import { contextManager } from './context';

/**
 * Lifecycle handler function type
 */
export type LifecycleHandler = () => Promise<void> | void;

/**
 * Global storage for lifecycle handlers
 * This allows plugins to register lifecycle handlers that will be called
 * during plugin activation and deactivation
 */
class LifecycleRegistry {
  private activateHandlers: LifecycleHandler[] = [];
  private deactivateHandlers: LifecycleHandler[] = [];

  /**
   * Register an activation handler
   * @param handler - Function to call when plugin is activated
   */
  addActivateHandler(handler: LifecycleHandler): void {
    this.activateHandlers.push(handler);
  }

  /**
   * Register a deactivation handler
   * @param handler - Function to call when plugin is deactivated
   */
  addDeactivateHandler(handler: LifecycleHandler): void {
    this.deactivateHandlers.push(handler);
  }

  /**
   * Execute all activation handlers
   */
  async executeActivateHandlers(): Promise<void> {
    for (const handler of this.activateHandlers) {
      try {
        await handler();
      } catch (error) {
        throw new PluginError(
          `Plugin activation handler failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
          PluginErrorCode.LIFECYCLE_ERROR
        );
      }
    }
  }

  /**
   * Execute all deactivation handlers
   */
  async executeDeactivateHandlers(): Promise<void> {
    for (const handler of this.deactivateHandlers) {
      try {
        await handler();
      } catch (error) {
        console.warn('Plugin deactivation handler failed:', error);
        // Don't throw during deactivation to allow cleanup to continue
      }
    }
  }

  /**
   * Clear all registered handlers
   */
  clear(): void {
    this.activateHandlers = [];
    this.deactivateHandlers = [];
  }

  /**
   * Get the number of registered handlers
   */
  getHandlerCounts(): { activate: number; deactivate: number } {
    return {
      activate: this.activateHandlers.length,
      deactivate: this.deactivateHandlers.length
    };
  }
}

/**
 * Global lifecycle registry instance
 */
export const lifecycleRegistry = new LifecycleRegistry();

/**
 * Hook to access the application API
 * 
 * This hook provides access to the main application's API for showing notifications,
 * opening dialogs, and other app-level interactions.
 * 
 * @returns Application API object
 * @throws PluginError if called outside of plugin context
 * 
 * @example
 * ```typescript
 * const app = useApp();
 * await app.showNotification('Hello from plugin!');
 * ```
 */
export function useApp(): AppAPI {
  try {
    const context = contextManager.getContext();
    return context.app;
  } catch (error) {
    throw new PluginError(
      'useApp() can only be called within a plugin lifecycle function. Ensure you are calling this from within onActivate, onDeactivate, or other plugin callbacks.',
      PluginErrorCode.HOOK_ERROR
    );
  }
}

/**
 * Hook to access the event system API
 * 
 * This hook provides access to the pub/sub event system for inter-plugin
 * communication and listening to application events.
 * 
 * @returns Event system API object
 * @throws PluginError if called outside of plugin context
 * 
 * @example
 * ```typescript
 * const events = useEvents();
 * events.on('user-action', (data) => {
 *   console.log('User action:', data);
 * });
 * ```
 */
export function useEvents(): EventAPI {
  try {
    const context = contextManager.getContext();
    return context.events;
  } catch (error) {
    throw new PluginError(
      'useEvents() can only be called within a plugin lifecycle function. Ensure you are calling this from within onActivate, onDeactivate, or other plugin callbacks.',
      PluginErrorCode.HOOK_ERROR
    );
  }
}

/**
 * Hook to access the storage API
 * 
 * This hook provides access to persistent storage for the plugin.
 * Each plugin has its own isolated storage namespace.
 * 
 * @returns Storage API object
 * @throws PluginError if called outside of plugin context
 * 
 * @example
 * ```typescript
 * const storage = useStorage();
 * await storage.set('user-preference', { theme: 'dark' });
 * const preference = await storage.get('user-preference');
 * ```
 */
export function useStorage(): StorageAPI {
  try {
    const context = contextManager.getContext();
    return context.storage;
  } catch (error) {
    throw new PluginError(
      'useStorage() can only be called within a plugin lifecycle function. Ensure you are calling this from within onActivate, onDeactivate, or other plugin callbacks.',
      PluginErrorCode.HOOK_ERROR
    );
  }
}

/**
 * Hook to access the complete plugin context
 * 
 * This hook provides access to the full plugin context object,
 * which includes all available APIs.
 * 
 * @returns Complete plugin context
 * @throws PluginError if called outside of plugin context
 * 
 * @example
 * ```typescript
 * const context = useContext();
 * await context.app.showNotification('Hello!');
 * context.events.emit('plugin-ready');
 * ```
 */
export function useContext(): PluginContext {
  try {
    return contextManager.getContext();
  } catch (error) {
    throw new PluginError(
      'useContext() can only be called within a plugin lifecycle function. Ensure you are calling this from within onActivate, onDeactivate, or other plugin callbacks.',
      PluginErrorCode.HOOK_ERROR
    );
  }
}

/**
 * Register a function to be called when the plugin is activated
 * 
 * This hook allows you to register lifecycle handlers that will be executed
 * when the plugin is activated. Multiple handlers can be registered and they
 * will be executed in the order they were registered.
 * 
 * @param handler - Function to call during plugin activation
 * 
 * @example
 * ```typescript
 * onActivate(async () => {
 *   const app = useApp();
 *   await app.showNotification('Plugin activated!');
 * });
 * ```
 */
export function onActivate(handler: LifecycleHandler): void {
  if (typeof handler !== 'function') {
    throw new PluginError(
      'onActivate() requires a function as argument',
      PluginErrorCode.HOOK_ERROR
    );
  }
  
  lifecycleRegistry.addActivateHandler(handler);
}

/**
 * Register a function to be called when the plugin is deactivated
 * 
 * This hook allows you to register cleanup handlers that will be executed
 * when the plugin is deactivated. Multiple handlers can be registered and they
 * will be executed in the order they were registered.
 * 
 * @param handler - Function to call during plugin deactivation
 * 
 * @example
 * ```typescript
 * onDeactivate(async () => {
 *   const events = useEvents();
 *   events.off('user-action', myHandler);
 *   console.log('Plugin cleaned up');
 * });
 * ```
 */
export function onDeactivate(handler: LifecycleHandler): void {
  if (typeof handler !== 'function') {
    throw new PluginError(
      'onDeactivate() requires a function as argument',
      PluginErrorCode.HOOK_ERROR
    );
  }
  
  lifecycleRegistry.addDeactivateHandler(handler);
}

/**
 * Utility function to check if hooks can be called
 * 
 * This function checks if the plugin context is available,
 * which indicates whether hooks can be safely called.
 * 
 * @returns True if hooks can be called, false otherwise
 * 
 * @example
 * ```typescript
 * if (canUseHooks()) {
 *   const app = useApp();
 *   // Safe to use hooks
 * }
 * ```
 */
export function canUseHooks(): boolean {
  return contextManager.hasContext();
}

/**
 * Utility function to safely call a hook with error handling
 * 
 * This function wraps hook calls with proper error handling and
 * provides more descriptive error messages.
 * 
 * @param hookFn - Hook function to call
 * @param hookName - Name of the hook for error messages
 * @returns Result of the hook function
 * @throws PluginError with descriptive message
 * 
 * @example
 * ```typescript
 * const app = safeHookCall(() => useApp(), 'useApp');
 * ```
 */
export function safeHookCall<T>(hookFn: () => T, hookName: string): T {
  try {
    return hookFn();
  } catch (error) {
    if (error instanceof PluginError) {
      throw error;
    }
    
    throw new PluginError(
      `Hook ${hookName}() failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      PluginErrorCode.HOOK_ERROR
    );
  }
}

/**
 * Advanced hook for conditional API access
 * 
 * This hook provides safe access to APIs with fallback handling.
 * It returns null if the context is not available instead of throwing.
 * 
 * @returns Plugin context or null if not available
 * 
 * @example
 * ```typescript
 * const context = useOptionalContext();
 * if (context) {
 *   await context.app.showNotification('Context available!');
 * }
 * ```
 */
export function useOptionalContext(): PluginContext | null {
  try {
    return contextManager.getContext();
  } catch {
    return null;
  }
}

/**
 * Hook for accessing individual APIs with optional fallback
 * 
 * These hooks provide safe access to individual APIs and return null
 * if the context is not available.
 */
export const OptionalHooks = {
  /**
   * Safely access the app API
   * @returns App API or null if not available
   */
  useOptionalApp(): AppAPI | null {
    const context = useOptionalContext();
    return context?.app || null;
  },

  /**
   * Safely access the events API
   * @returns Events API or null if not available
   */
  useOptionalEvents(): EventAPI | null {
    const context = useOptionalContext();
    return context?.events || null;
  },

  /**
   * Safely access the storage API
   * @returns Storage API or null if not available
   */
  useOptionalStorage(): StorageAPI | null {
    const context = useOptionalContext();
    return context?.storage || null;
  }
};

/**
 * Lifecycle management utilities
 * 
 * These utilities help manage the plugin lifecycle and provide
 * debugging and introspection capabilities.
 */
export const LifecycleUtils = {
  /**
   * Get information about registered lifecycle handlers
   * @returns Object with handler counts
   */
  getLifecycleInfo() {
    return lifecycleRegistry.getHandlerCounts();
  },

  /**
   * Clear all registered lifecycle handlers
   * This is useful for testing or plugin reloading
   */
  clearLifecycleHandlers() {
    lifecycleRegistry.clear();
  },

  /**
   * Execute activation handlers manually
   * This is typically called by the plugin manager
   */
  async executeActivation(): Promise<void> {
    await lifecycleRegistry.executeActivateHandlers();
  },

  /**
   * Execute deactivation handlers manually
   * This is typically called by the plugin manager
   */
  async executeDeactivation(): Promise<void> {
    await lifecycleRegistry.executeDeactivateHandlers();
  }
};

/**
 * Hook composition utilities
 * 
 * These utilities help compose multiple hooks together for common patterns.
 */
export const HookComposers = {
  /**
   * Compose app and storage hooks for common plugin initialization
   * @returns Object with app and storage APIs
   */
  useAppAndStorage(): { app: AppAPI; storage: StorageAPI } {
    return {
      app: useApp(),
      storage: useStorage()
    };
  },

  /**
   * Compose all hooks for full plugin functionality
   * @returns Object with all APIs
   */
  useAllAPIs(): { app: AppAPI; events: EventAPI; storage: StorageAPI } {
    return {
      app: useApp(),
      events: useEvents(),
      storage: useStorage()
    };
  },

  /**
   * Compose hooks with error handling
   * @returns Object with APIs or null values if not available
   */
  useSafeAPIs(): {
    app: AppAPI | null;
    events: EventAPI | null;
    storage: StorageAPI | null;
  } {
    return {
      app: OptionalHooks.useOptionalApp(),
      events: OptionalHooks.useOptionalEvents(),
      storage: OptionalHooks.useOptionalStorage()
    };
  }
};