/**
 * API implementations for plugin-app communication
 * This file contains the concrete implementations of the API interfaces
 */

import type { AppAPI, DialogOptions, StorageAPI } from './types';
import { CommunicationBridge, invokeApi } from './communication';
import { PluginError, PluginErrorCode } from './types';

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

/**
 * Concrete implementation of the AppAPI interface
 */
export class AppAPIImpl implements ExtendedAppAPI {
  constructor(private bridge: CommunicationBridge) {}

  async showNotification(message: string): Promise<void> {
    await this.bridge.invoke('show_notification', { message });
  }

  async getAppVersion(): Promise<string> {
    return await this.bridge.invoke<string>('get_app_version');
  }

  async openDialog(options: DialogOptions): Promise<string | null> {
    return await this.bridge.invoke<string | null>('open_dialog', { options });
  }

  async getAppInfo(): Promise<AppInfo> {
    return await this.bridge.invoke<AppInfo>('get_app_info');
  }

  async hasPermission(permission: string): Promise<boolean> {
    return await this.bridge.invoke<boolean>('has_permission', { permission });
  }

  async requestPermission(permission: string): Promise<boolean> {
    return await this.bridge.invoke<boolean>('request_permission', { permission });
  }
}

/**
 * Concrete implementation of the StorageAPI interface
 */
export class StorageAPIImpl implements ExtendedStorageAPI {
  constructor(private bridge: CommunicationBridge) {}

  async get(key: string): Promise<any> {
    return await this.bridge.invoke('storage_get', { key });
  }

  async set(key: string, value: any): Promise<void> {
    await this.bridge.invoke('storage_set', { key, value });
  }

  async remove(key: string): Promise<void> {
    await this.bridge.invoke('storage_remove', { key });
  }

  async clear(): Promise<void> {
    await this.bridge.invoke('storage_clear');
  }

  async keys(): Promise<string[]> {
    return await this.bridge.invoke<string[]>('storage_keys');
  }

  async getMany(keys: string[]): Promise<Record<string, any>> {
    return await this.bridge.invoke<Record<string, any>>('storage_get_many', { keys });
  }

  async setMany(data: Record<string, any>): Promise<void> {
    await this.bridge.invoke('storage_set_many', { data });
  }

  async has(key: string): Promise<boolean> {
    return await this.bridge.invoke<boolean>('storage_has', { key });
  }

  async getSize(): Promise<StorageSize> {
    return await this.bridge.invoke<StorageSize>('storage_get_size');
  }
}

/**
 * Factory function to create API implementations
 * @param bridge - Communication bridge instance
 * @returns Object with API implementations
 */
export function createAPIImplementations(bridge: CommunicationBridge) {
  return {
    app: new AppAPIImpl(bridge),
    storage: new StorageAPIImpl(bridge)
  };
}

/**
 * Utility functions for common API operations
 */
export const ApiUtils = {
  /**
   * Show a notification with error handling
   * @param message - Notification message
   * @param fallback - Fallback function if API fails
   */
  async showNotificationSafe(message: string, fallback?: (msg: string) => void): Promise<void> {
    try {
      await invokeApi('show_notification', { message });
    } catch (error) {
      console.warn('[Plugin SDK] Failed to show notification:', error);
      if (fallback) {
        fallback(message);
      } else {
        // Fallback to console log
        console.log(`[Notification] ${message}`);
      }
    }
  },

  /**
   * Get app version with fallback
   * @param fallback - Fallback version string
   * @returns App version or fallback
   */
  async getAppVersionSafe(fallback: string = 'unknown'): Promise<string> {
    try {
      return await invokeApi<string>('get_app_version');
    } catch (error) {
      console.warn('[Plugin SDK] Failed to get app version:', error);
      return fallback;
    }
  },

  /**
   * Check if a permission is available
   * @param permission - Permission to check
   * @returns True if permission is available, false otherwise
   */
  async checkPermissionSafe(permission: string): Promise<boolean> {
    try {
      return await invokeApi<boolean>('has_permission', { permission });
    } catch (error) {
      console.warn('[Plugin SDK] Failed to check permission:', error);
      return false;
    }
  }
};