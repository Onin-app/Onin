/**
 * Plugin Lifecycle API
 * 
 * Provides lifecycle hooks for plugin initialization and cleanup.
 * Similar to Vue's lifecycle hooks or React's useEffect.
 * 
 * Callbacks are executed automatically at the end of the current event loop tick.
 * 
 * @module api/lifecycle
 * @example
 * ```typescript
 * import { lifecycle, settings, command } from 'baize-plugin-sdk';
 * 
 * // Register lifecycle hooks - they execute automatically!
 * lifecycle.onLoad(async () => {
 *   // Register settings
 *   await settings.useSettingsSchema([...]);
 *   
 *   // Register commands
 *   command.register(async (cmd, args) => {...});
 *   
 *   console.log('Plugin loaded!');
 * });
 * ```
 */

import { createError } from '../types/errors';

type LifecycleCallback = () => void | Promise<void>;

let loadCallbacks: LifecycleCallback[] = [];
let activateCallbacks: LifecycleCallback[] = [];
let deactivateCallbacks: LifecycleCallback[] = [];
let loadExecutionScheduled = false;

/**
 * Register a callback to run when the plugin is loaded
 * 
 * This hook runs automatically when the plugin is loaded by the system,
 * before any user interaction. The callback is executed at the end of the
 * current event loop tick, so you can register multiple callbacks and they
 * will all be executed together.
 * 
 * Use it to:
 * - Register settings schema
 * - Register command handlers
 * - Initialize default data
 * - Set up plugin state
 * 
 * @param callback - Function to execute on plugin load
 * 
 * @example
 * ```typescript
 * import { lifecycle, settings, command, storage } from 'baize-plugin-sdk';
 * 
 * lifecycle.onLoad(async () => {
 *   // 1. Register settings
 *   await settings.useSettingsSchema([
 *     {
 *       key: 'apiKey',
 *       label: 'API Key',
 *       type: 'password',
 *       required: true
 *     }
 *   ]);
 *   
 *   // 2. Register command handler
 *   command.register(async (cmd, args) => {
 *     if (cmd === 'get-status') {
 *       return { status: 'ready' };
 *     }
 *   });
 *   
 *   // 3. Initialize first-run data
 *   const firstRun = await storage.getItem('first-run');
 *   if (firstRun === null) {
 *     await storage.setItem('first-run', false);
 *     await storage.setItem('install-time', new Date().toISOString());
 *   }
 *   
 *   console.log('Plugin initialized successfully');
 * });
 * ```
 */
function onLoad(callback: LifecycleCallback): void {
  loadCallbacks.push(callback);
  
  // Schedule execution if not already scheduled
  if (!loadExecutionScheduled) {
    loadExecutionScheduled = true;
    // Use queueMicrotask to execute at the end of current event loop tick
    queueMicrotask(() => {
      executeLoadCallbacks().catch(error => {
        console.error('[Lifecycle] Failed to execute onLoad callbacks:', error);
      });
    });
  }
}

/**
 * Register a callback to run when the plugin is activated by the user
 * 
 * This hook runs when the user explicitly executes the plugin.
 * Use it for:
 * - Showing UI
 * - Performing main plugin functionality
 * - User interactions
 * 
 * @param callback - Function to execute on plugin activation
 * 
 * @example
 * ```typescript
 * import { lifecycle, notification } from 'baize-plugin-sdk';
 * 
 * lifecycle.onActivate(async () => {
 *   await notification.show({
 *     title: 'Plugin Activated',
 *     body: 'The plugin is now running'
 *   });
 * });
 * ```
 */
function onActivate(callback: LifecycleCallback): void {
  activateCallbacks.push(callback);
}

/**
 * Register a callback to run when the plugin is deactivated
 * 
 * This hook runs when the plugin is disabled or unloaded.
 * Use it for:
 * - Cleanup resources
 * - Save state
 * - Cancel pending operations
 * 
 * @param callback - Function to execute on plugin deactivation
 * 
 * @example
 * ```typescript
 * import { lifecycle, storage } from 'baize-plugin-sdk';
 * 
 * let intervalId: number;
 * 
 * lifecycle.onLoad(() => {
 *   intervalId = setInterval(() => {
 *     console.log('Background task running...');
 *   }, 5000);
 * });
 * 
 * lifecycle.onDeactivate(async () => {
 *   clearInterval(intervalId);
 *   await storage.setItem('last-deactivate', new Date().toISOString());
 *   console.log('Plugin cleaned up');
 * });
 * ```
 */
function onDeactivate(callback: LifecycleCallback): void {
  deactivateCallbacks.push(callback);
}

/**
 * Internal function to execute all registered load callbacks
 * Called automatically by the plugin system
 * @internal
 */
async function executeLoadCallbacks(): Promise<void> {
  for (const callback of loadCallbacks) {
    try {
      await callback();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('[Lifecycle] Error in onLoad callback:', errorMessage);
      throw createError.common.unknown(`onLoad callback failed: ${errorMessage}`);
    }
  }
}

/**
 * Internal function to execute all registered activate callbacks
 * Called automatically by the plugin system
 * @internal
 */
async function executeActivateCallbacks(): Promise<void> {
  for (const callback of activateCallbacks) {
    try {
      await callback();
    } catch (error) {
      console.error('[Lifecycle] Error in onActivate callback:', error);
      throw createError.common.unknown(`onActivate callback failed: ${error}`);
    }
  }
}

/**
 * Internal function to execute all registered deactivate callbacks
 * Called automatically by the plugin system
 * @internal
 */
async function executeDeactivateCallbacks(): Promise<void> {
  for (const callback of deactivateCallbacks) {
    try {
      await callback();
    } catch (error) {
      console.error('[Lifecycle] Error in onDeactivate callback:', error);
      // Don't throw on deactivate errors, just log them
    }
  }
}

/**
 * Internal function to reset all lifecycle callbacks
 * Used for testing and plugin reloading
 * @internal
 */
function resetCallbacks(): void {
  loadCallbacks = [];
  activateCallbacks = [];
  deactivateCallbacks = [];
}

/**
 * Lifecycle API namespace
 */
export const lifecycle = {
  onLoad,
  onActivate,
  onDeactivate,
};
