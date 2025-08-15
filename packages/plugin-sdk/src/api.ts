/**
 * API definitions for plugin-app communication
 * This file contains the interface definitions that will be implemented in task 2.3
 */

import type { AppAPI, DialogOptions, StorageAPI } from './types';

// Re-export the interfaces from types for convenience
export type { AppAPI, DialogOptions, StorageAPI } from './types';

/**
 * Extended app API with additional utility methods
 */
export interface ExtendedAppAPI extends AppAPI {
  /**
   * Get application information
   */
  getAppInfo(): Promise<AppInfo>;
  
  /**
   * Check if a permission is granted
   * @param permission - Permission to check
   */
  hasPermission(permission: string): Promise<boolean>;
  
  /**
   * Request a permission from the user
   * @param permission - Permission to request
   */
  requestPermission(permission: string): Promise<boolean>;
}

/**
 * Application information structure
 */
export interface AppInfo {
  /** Application name */
  name: string;
  /** Application version */
  version: string;
  /** Application build number */
  build?: string;
  /** Platform information */
  platform: string;
}

/**
 * Storage configuration options
 */
export interface StorageOptions {
  /** Storage namespace for isolation */
  namespace?: string;
  /** Whether to encrypt stored data */
  encrypted?: boolean;
}

/**
 * Extended storage API with additional features
 */
export interface ExtendedStorageAPI extends StorageAPI {
  /**
   * Get multiple values at once
   * @param keys - Array of keys to retrieve
   */
  getMany(keys: string[]): Promise<Record<string, any>>;
  
  /**
   * Set multiple values at once
   * @param data - Object with key-value pairs to store
   */
  setMany(data: Record<string, any>): Promise<void>;
  
  /**
   * Check if a key exists in storage
   * @param key - Key to check
   */
  has(key: string): Promise<boolean>;
  
  /**
   * Get storage size information
   */
  getSize(): Promise<StorageSize>;
}

/**
 * Storage size information
 */
export interface StorageSize {
  /** Number of items in storage */
  itemCount: number;
  /** Estimated size in bytes */
  estimatedSize: number;
}