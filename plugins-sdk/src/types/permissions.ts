/**
 * Plugin permission configuration type definitions
 * @fileoverview Defines all permission interfaces for plugin capabilities
 */

/**
 * HTTP permission configuration
 * @interface HttpPermission
 */
export interface HttpPermission {
  /** Whether HTTP permission is enabled */
  enable: boolean;
  /** List of allowed URLs, supports wildcards */
  allowUrls: string[];
  /** Request timeout in milliseconds (optional) */
  timeout?: number;
  /** Maximum number of retries (optional) */
  maxRetries?: number;
}

/**
 * Storage permission configuration
 * @interface StoragePermission
 */
export interface StoragePermission {
  /** Whether storage permission is enabled */
  enable: boolean;
  /** Whether local storage is allowed */
  local: boolean;
  /** Whether session storage is allowed */
  session: boolean;
  /** Maximum storage size (optional) */
  maxSize?: string;
}

/**
 * Notification permission configuration
 * @interface NotificationPermission
 */
export interface NotificationPermission {
  /** Whether notification permission is enabled */
  enable: boolean;
  /** Whether sound notifications are allowed */
  sound: boolean;
  /** Whether badge notifications are allowed */
  badge: boolean;
}

/**
 * Command permission configuration
 * @interface CommandPermission
 */
export interface CommandPermission {
  /** Whether command permission is enabled */
  enable: boolean;
  /** List of allowed commands, supports wildcards */
  allowCommands: string[];
  /** Maximum execution time in milliseconds (optional) */
  maxExecutionTime?: number;
}

/**
 * File system permission configuration
 * @interface FileSystemPermission
 */
interface FileSystemPermission {
  /** Whether file system permission is enabled */
  enable: boolean;
  /** Whether file reading is allowed */
  read: boolean;
  /** Whether file writing is allowed */
  write: boolean;
  /** Whether file deletion is allowed */
  delete: boolean;
  /** Maximum file size limit in bytes (optional) */
  maxFileSize?: number;
}

/**
 * Dialog permission configuration
 * @interface DialogPermission
 */
interface DialogPermission {
  /** Whether dialog permission is enabled */
  enable: boolean;
  /** Whether message dialogs are allowed */
  message: boolean;
  /** Whether confirmation dialogs are allowed */
  confirm: boolean;
  /** Whether file selection dialogs are allowed */
  fileDialog: boolean;
}

/**
 * Clipboard permission configuration
 * @interface ClipboardPermission
 */
interface ClipboardPermission {
  /** Whether clipboard permission is enabled */
  enable: boolean;
  /** Whether text reading is allowed */
  readText: boolean;
  /** Whether text writing is allowed */
  writeText: boolean;
  /** Whether image reading is allowed */
  readImage: boolean;
  /** Whether image writing is allowed */
  writeImage: boolean;
  /** Whether clipboard clearing is allowed */
  clear: boolean;
}

/**
 * Plugin permissions configuration
 * @interface PluginPermissions
 */
export interface PluginPermissions {
  /** HTTP permission configuration */
  http?: HttpPermission;
  /** Storage permission configuration */
  storage?: StoragePermission;
  /** Notification permission configuration */
  notification?: NotificationPermission;
  /** Command permission configuration */
  command?: CommandPermission;
  /** File system permission configuration */
  fs?: FileSystemPermission;
  /** Dialog permission configuration */
  dialog?: DialogPermission;
  /** Clipboard permission configuration */
  clipboard?: ClipboardPermission;
}

/**
 * Plugin command keyword configuration
 * @interface PluginCommandKeyword
 */
export interface PluginCommandKeyword {
  /** Keyword name */
  name: string;
  /** Keyword type: "prefix" | "fuzzy" | "exact" */
  type: string;
}

/**
 * Plugin command match configuration
 * 
 * Three-layer graceful degradation model:
 * 1. Developer layer: Only configure extensions (e.g., [".png", ".jpg"])
 * 2. System layer: Automatically map extensions to internal MIME types
 * 3. Runtime layer: Prioritize MIME type matching, fallback to extensions
 * 
 * @interface PluginCommandMatch
 */
export interface PluginCommandMatch {
  /** Match type: "text" | "image" | "file" | "folder" */
  type: 'text' | 'image' | 'file' | 'folder';
  /** Match name */
  name: string;
  /** Match description */
  description: string;
  /** Regular expression for text matching (only for type="text", as an additional condition) */
  regexp?: string;
  /** Minimum count (text: character count, file/image/folder: file count) */
  min?: number;
  /** Maximum count (text: character count, file/image/folder: file count) */
  max?: number;
  /** 
   * File extensions filter (e.g., [".png", ".jpg"], [".pdf"], [".txt", ".md"])
   * Supports wildcards like "*"
   * Only for type="file" or "image"
   * System will automatically map extensions to MIME types for matching
   */
  extensions?: string[];
}

/**
 * Plugin command configuration
 * @interface PluginCommand
 */
export interface PluginCommand {
  /** Command code (unique identifier) */
  code: string;
  /** Command name */
  name: string;
  /** Command description */
  description: string;
  /** Command keywords */
  keywords: PluginCommandKeyword[];
  /** Command match conditions */
  matches?: PluginCommandMatch[];
}

/**
 * Plugin manifest configuration
 * @interface PluginManifest
 */
export interface PluginManifest {
  /** Plugin unique identifier */
  id: string;
  /** Plugin name */
  name: string;
  /** Plugin version */
  version: string;
  /** Plugin description */
  description: string;
  /** Entry file path */
  entry: string;
  /** Plugin type */
  type?: string;
  /** Permission configuration */
  permissions?: PluginPermissions;
  /** Command definitions */
  commands?: PluginCommand[];
  /** Display mode: "inline" | "window" */
  displayMode?: string;
  /** Auto detach to separate window */
  autoDetach?: boolean;
}