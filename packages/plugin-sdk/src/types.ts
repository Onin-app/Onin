/**
 * Core type definitions for the Baize Plugin SDK
 */

/**
 * Plugin manifest structure that defines plugin metadata and requirements
 */
export interface PluginManifest {
  /** Unique plugin name */
  name: string;
  /** Semantic version string */
  version: string;
  /** Human-readable description of the plugin */
  description: string;
  /** Plugin author information */
  author: string;
  /** Entry point file path relative to plugin root */
  main: string;
  /** List of permissions required by the plugin */
  permissions: string[];
  /** Engine compatibility requirements */
  engines: {
    /** Minimum Baize version required */
    baize: string;
  };
  /** Optional keywords for plugin discovery */
  keywords?: string[];
  /** Optional repository URL */
  repository?: string;
}

/**
 * Plugin context provided to plugins during activation
 * Contains all APIs available to the plugin
 */
export interface PluginContext {
  /** Application API for interacting with the main app */
  app: AppAPI;
  /** Event system API for pub/sub messaging */
  events: EventAPI;
  /** Storage API for persistent data */
  storage: StorageAPI;
}

/**
 * Main plugin interface that all plugins must implement
 */
export interface Plugin {
  /**
   * Called when the plugin is activated
   * @param context - Plugin context with available APIs
   */
  activate(context: PluginContext): Promise<void>;
  
  /**
   * Called when the plugin is deactivated
   */
  deactivate(): Promise<void>;
}

/**
 * Application API interface for main app interactions
 */
export interface AppAPI {
  /**
   * Show a notification to the user
   * @param message - Notification message
   */
  showNotification(message: string): Promise<void>;
  
  /**
   * Get the current application version
   * @returns Application version string
   */
  getAppVersion(): Promise<string>;
  
  /**
   * Open a dialog for user interaction
   * @param options - Dialog configuration options
   * @returns Selected value or null if cancelled
   */
  openDialog(options: DialogOptions): Promise<string | null>;
}

/**
 * Event system API interface for pub/sub messaging
 */
export interface EventAPI {
  /**
   * Subscribe to an event
   * @param event - Event name to listen for
   * @param handler - Function to call when event is triggered
   */
  on(event: string, handler: EventHandler): void;
  
  /**
   * Emit an event to all subscribers
   * @param event - Event name to emit
   * @param data - Optional data to send with the event
   */
  emit(event: string, data?: any): void;
  
  /**
   * Unsubscribe from an event
   * @param event - Event name to stop listening to
   * @param handler - Specific handler to remove
   */
  off(event: string, handler: EventHandler): void;
}

/**
 * Storage API interface for persistent data management
 */
export interface StorageAPI {
  /**
   * Get a value from storage
   * @param key - Storage key
   * @returns Stored value or undefined if not found
   */
  get(key: string): Promise<any>;
  
  /**
   * Set a value in storage
   * @param key - Storage key
   * @param value - Value to store
   */
  set(key: string, value: any): Promise<void>;
  
  /**
   * Remove a value from storage
   * @param key - Storage key to remove
   */
  remove(key: string): Promise<void>;
  
  /**
   * Clear all storage for this plugin
   */
  clear(): Promise<void>;
  
  /**
   * Get all keys in storage for this plugin
   * @returns Array of storage keys
   */
  keys(): Promise<string[]>;
}

/**
 * Dialog options for user interaction
 */
export interface DialogOptions {
  /** Dialog title */
  title?: string;
  /** Dialog message or content */
  message: string;
  /** Dialog type */
  type?: 'info' | 'warning' | 'error' | 'question';
  /** Available buttons */
  buttons?: string[];
  /** Default button index */
  defaultButton?: number;
}

/**
 * Event handler function type
 */
export type EventHandler = (data?: any) => void;

/**
 * Plugin status enumeration
 */
export enum PluginStatus {
  /** Plugin is not loaded */
  INACTIVE = 'inactive',
  /** Plugin is loaded and running */
  ACTIVE = 'active',
  /** Plugin encountered an error */
  ERROR = 'error',
  /** Plugin is in the process of loading */
  LOADING = 'loading'
}

/**
 * Plugin information structure used by the plugin manager
 */
export interface PluginInfo {
  /** Plugin manifest data */
  manifest: PluginManifest;
  /** Current plugin status */
  status: PluginStatus;
  /** Whether the plugin is enabled by user */
  enabled: boolean;
  /** Error message if plugin is in error state */
  error?: string;
  /** Plugin file path */
  path?: string;
}

/**
 * Plugin error types
 */
export class PluginError extends Error {
  constructor(
    message: string,
    public code: PluginErrorCode,
    public pluginName?: string
  ) {
    super(message);
    this.name = 'PluginError';
  }
}

/**
 * Plugin error codes
 */
export enum PluginErrorCode {
  /** Plugin not found */
  NOT_FOUND = 'NOT_FOUND',
  /** Invalid plugin manifest */
  INVALID_MANIFEST = 'INVALID_MANIFEST',
  /** Plugin load failed */
  LOAD_FAILED = 'LOAD_FAILED',
  /** Permission denied */
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  /** Version incompatible */
  VERSION_INCOMPATIBLE = 'VERSION_INCOMPATIBLE',
  /** API call failed */
  API_CALL_FAILED = 'API_CALL_FAILED',
  /** Plugin activation failed */
  ACTIVATION_FAILED = 'ACTIVATION_FAILED'
}