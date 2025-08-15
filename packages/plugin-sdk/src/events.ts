/**
 * Event system definitions for plugin communication
 * This file contains the interface definitions that will be implemented in task 2.2
 */

import type { EventAPI, EventHandler } from './types';

// Re-export the interfaces from types for convenience
export type { EventAPI, EventHandler } from './types';

/**
 * Extended event API with additional features
 */
export interface ExtendedEventAPI extends EventAPI {
  /**
   * Subscribe to an event with options
   * @param event - Event name to listen for
   * @param handler - Function to call when event is triggered
   * @param options - Subscription options
   */
  subscribe(event: string, handler: EventHandler, options?: EventSubscriptionOptions): EventSubscription;
  
  /**
   * Emit an event with metadata
   * @param event - Event name to emit
   * @param data - Event data
   * @param metadata - Event metadata
   */
  emitWithMetadata(event: string, data?: any, metadata?: EventMetadata): void;
  
  /**
   * Get list of active event listeners
   */
  getListeners(): EventListenerInfo[];
  
  /**
   * Remove all listeners for a specific event
   * @param event - Event name to clear
   */
  removeAllListeners(event?: string): void;
}

/**
 * Event subscription options
 */
export interface EventSubscriptionOptions {
  /** Whether to call handler only once */
  once?: boolean;
  /** Priority for handler execution order */
  priority?: number;
  /** Filter function to conditionally handle events */
  filter?: (data: any) => boolean;
}

/**
 * Event subscription handle
 */
export interface EventSubscription {
  /** Event name */
  event: string;
  /** Subscription ID */
  id: string;
  /** Unsubscribe from the event */
  unsubscribe(): void;
}

/**
 * Event metadata
 */
export interface EventMetadata {
  /** Event timestamp */
  timestamp?: number;
  /** Event source plugin */
  source?: string;
  /** Event priority */
  priority?: number;
  /** Whether event can be cancelled */
  cancellable?: boolean;
}

/**
 * Event listener information
 */
export interface EventListenerInfo {
  /** Event name */
  event: string;
  /** Number of listeners */
  listenerCount: number;
  /** Listener details */
  listeners: {
    id: string;
    priority: number;
    once: boolean;
  }[];
}

/**
 * Built-in system events that plugins can listen to
 */
export enum SystemEvents {
  /** Application is starting up */
  APP_STARTUP = 'app:startup',
  /** Application is shutting down */
  APP_SHUTDOWN = 'app:shutdown',
  /** Plugin was loaded */
  PLUGIN_LOADED = 'plugin:loaded',
  /** Plugin was unloaded */
  PLUGIN_UNLOADED = 'plugin:unloaded',
  /** Plugin encountered an error */
  PLUGIN_ERROR = 'plugin:error',
  /** Settings were changed */
  SETTINGS_CHANGED = 'settings:changed',
  /** Theme was changed */
  THEME_CHANGED = 'theme:changed'
}

/**
 * Event data for plugin-related events
 */
export interface PluginEventData {
  /** Plugin name */
  pluginName: string;
  /** Plugin version */
  version: string;
  /** Additional event-specific data */
  data?: any;
}