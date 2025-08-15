/**
 * Event system implementation for plugin communication
 * Provides pub/sub messaging between plugins and the main application
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

/**
 * Internal event listener structure
 */
interface EventListener {
  /** Unique listener ID */
  id: string;
  /** Event handler function */
  handler: EventHandler;
  /** Subscription options */
  options: EventSubscriptionOptions;
}

/**
 * Event emitter implementation for the plugin system
 * Provides a robust pub/sub mechanism with advanced features
 */
export class EventEmitter implements ExtendedEventAPI {
  private listeners: Map<string, EventListener[]> = new Map();
  private nextId = 1;

  /**
   * Subscribe to an event (basic EventAPI implementation)
   * @param event - Event name to listen for
   * @param handler - Function to call when event is triggered
   */
  on(event: string, handler: EventHandler): void {
    this.subscribe(event, handler);
  }

  /**
   * Emit an event to all subscribers (basic EventAPI implementation)
   * @param event - Event name to emit
   * @param data - Optional data to send with the event
   */
  emit(event: string, data?: any): void {
    this.emitWithMetadata(event, data);
  }

  /**
   * Unsubscribe from an event (basic EventAPI implementation)
   * @param event - Event name to stop listening to
   * @param handler - Specific handler to remove
   */
  off(event: string, handler: EventHandler): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return;

    const index = eventListeners.findIndex(listener => listener.handler === handler);
    if (index !== -1) {
      eventListeners.splice(index, 1);
      if (eventListeners.length === 0) {
        this.listeners.delete(event);
      }
    }
  }

  /**
   * Subscribe to an event with advanced options
   * @param event - Event name to listen for
   * @param handler - Function to call when event is triggered
   * @param options - Subscription options
   */
  subscribe(event: string, handler: EventHandler, options: EventSubscriptionOptions = {}): EventSubscription {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }

    const id = `listener_${this.nextId++}`;
    const listener: EventListener = {
      id,
      handler,
      options: {
        once: false,
        priority: 0,
        ...options
      }
    };

    const eventListeners = this.listeners.get(event)!;
    
    // Insert listener based on priority (higher priority first)
    const insertIndex = eventListeners.findIndex(l => l.options.priority! < listener.options.priority!);
    if (insertIndex === -1) {
      eventListeners.push(listener);
    } else {
      eventListeners.splice(insertIndex, 0, listener);
    }

    return {
      event,
      id,
      unsubscribe: () => {
        const listeners = this.listeners.get(event);
        if (listeners) {
          const index = listeners.findIndex(l => l.id === id);
          if (index !== -1) {
            listeners.splice(index, 1);
            if (listeners.length === 0) {
              this.listeners.delete(event);
            }
          }
        }
      }
    };
  }

  /**
   * Emit an event with metadata
   * @param event - Event name to emit
   * @param data - Event data
   * @param metadata - Event metadata
   */
  emitWithMetadata(event: string, data?: any, metadata: EventMetadata = {}): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners || eventListeners.length === 0) return;

    // If data is an object, add metadata to it; otherwise create a wrapper
    let eventData: any;
    if (data && typeof data === 'object' && !Array.isArray(data)) {
      eventData = {
        ...data,
        __metadata: {
          timestamp: Date.now(),
          ...metadata
        }
      };
    } else {
      eventData = {
        data,
        __metadata: {
          timestamp: Date.now(),
          ...metadata
        }
      };
    }

    // Create a copy of listeners to avoid issues if listeners are modified during emission
    const listenersToCall = [...eventListeners];

    for (const listener of listenersToCall) {
      let shouldRemoveOnceListener = false;
      
      try {
        // Apply filter if provided (filter receives the original data)
        if (listener.options.filter && !listener.options.filter(data)) {
          continue;
        }

        // Call the handler (handler receives the wrapped data with metadata)
        listener.handler(eventData);
        
        // Mark for removal if it's a one-time subscription
        if (listener.options.once) {
          shouldRemoveOnceListener = true;
        }
      } catch (error) {
        console.error(`Error in event handler for '${event}':`, error);
        
        // Still remove once listeners even if they error
        if (listener.options.once) {
          shouldRemoveOnceListener = true;
        }
        
        // Emit error event for debugging
        this.emitWithMetadata('system:error', {
          type: 'event_handler_error',
          event,
          error: error instanceof Error ? error.message : String(error),
          listenerId: listener.id
        });
      }
      
      // Remove listener if it was marked for removal
      if (shouldRemoveOnceListener) {
        const index = eventListeners.findIndex(l => l.id === listener.id);
        if (index !== -1) {
          eventListeners.splice(index, 1);
        }
      }
    }

    // Clean up empty event arrays
    if (eventListeners.length === 0) {
      this.listeners.delete(event);
    }
  }

  /**
   * Get list of active event listeners
   */
  getListeners(): EventListenerInfo[] {
    const result: EventListenerInfo[] = [];

    for (const [event, listeners] of this.listeners.entries()) {
      result.push({
        event,
        listenerCount: listeners.length,
        listeners: listeners.map(l => ({
          id: l.id,
          priority: l.options.priority || 0,
          once: l.options.once || false
        }))
      });
    }

    return result;
  }

  /**
   * Remove all listeners for a specific event or all events
   * @param event - Event name to clear, or undefined to clear all
   */
  removeAllListeners(event?: string): void {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }

  /**
   * Get the number of listeners for a specific event
   * @param event - Event name
   * @returns Number of listeners
   */
  listenerCount(event: string): number {
    const eventListeners = this.listeners.get(event);
    return eventListeners ? eventListeners.length : 0;
  }

  /**
   * Check if there are any listeners for a specific event
   * @param event - Event name
   * @returns True if there are listeners
   */
  hasListeners(event: string): boolean {
    return this.listenerCount(event) > 0;
  }

  /**
   * Get all event names that have listeners
   * @returns Array of event names
   */
  eventNames(): string[] {
    return Array.from(this.listeners.keys());
  }
}

/**
 * Create a new event emitter instance
 * @returns New EventEmitter instance
 */
export function createEventEmitter(): ExtendedEventAPI {
  return new EventEmitter();
}

/**
 * Global event emitter instance for system-wide events
 * This should be used sparingly and mainly for system events
 */
export const globalEventEmitter = createEventEmitter();