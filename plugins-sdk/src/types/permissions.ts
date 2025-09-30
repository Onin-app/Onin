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
  /** Permission configuration */
  permissions?: PluginPermissions;
  /** Command definitions */
  commands?: Array<{
    id: string;
    name: string;
    description: string;
  }>;
}