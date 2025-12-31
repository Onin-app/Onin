import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

/**
 * Storage configuration options
 * @interface StorageOptions
 */
export interface StorageOptions {
  /** Reserved for future configuration options */
}

export interface StorageError extends Error {
  name: 'StorageError';
  key?: string;
}

/**
 * Storage error factory function
 * @param message - Error message
 * @param key - Optional storage key that caused the error
 * @returns StorageError instance
 */
export function createStorageError(
  message: string,
  key?: string,
): StorageError {
  const error = new Error(message) as StorageError;
  error.name = 'StorageError';
  error.key = key;
  return error;
}

/**
 * Type guard function to check if an error is a StorageError
 * @param error - The error to check
 * @returns True if the error is a StorageError
 */
export function isStorageError(error: any): error is StorageError {
  return error && error.name === 'StorageError';
}

/**
 * Generic storage API call helper function
 * @typeParam T - The expected return type
 * @param method - The storage method to call
 * @param args - Optional arguments for the method
 * @returns Promise resolving to the method result
 * @internal
 * @group Core
 */
function callStorageApi<T = any>(method: string, args?: any): Promise<T> {
  return dispatch({
    webview: () => invoke<T>(method, args),
    headless: () => invoke<T>(method, args),
  });
}

/**
 * Saves a key-value pair to the plugin's persistent storage.
 * The value can be any data that is serializable to JSON.
 *
 * @param key - Unique key for storing the value
 * @param value - The value to store
 * @returns Promise that resolves when the operation is complete
 * @throws {PluginError} With code `STORAGE_QUOTA_EXCEEDED` when storage limit is reached
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Store simple values
 * await storage.setItem('username', 'JohnDoe');
 * await storage.setItem('theme', 'dark');
 *
 * // Store complex objects
 * const user = { name: 'John', level: 10, preferences: { theme: 'dark' } };
 * await storage.setItem('playerProfile', user);
 *
 * // Handle storage errors
 * try {
 *   await storage.setItem('large-data', hugDataObject);
 * } catch (error) {
 *   if (errorUtils.isErrorCode(error, 'STORAGE_QUOTA_EXCEEDED')) {
 *     console.error('Storage quota exceeded, please clear some data');
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function setItem(key: string, value: any): Promise<void> {
  return callStorageApi('plugin_storage_set', { key, value });
}

/**
 * Retrieves a value from the plugin's persistent storage by the specified key.
 * @typeParam T - The expected type of the stored value
 * @param key - The key of the item to retrieve
 * @returns Promise that resolves to the stored value, or null if the key doesn't exist
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Get simple values
 * const username = await storage.getItem<string>('username');
 * if (username) {
 *   console.log(`Hello, ${username}`);
 * }
 *
 * // Get complex objects with type safety
 * interface UserProfile {
 *   name: string;
 *   level: number;
 *   preferences?: { theme: string };
 * }
 *
 * const profile = await storage.getItem<UserProfile>('playerProfile');
 * if (profile) {
 *   console.log(`Player: ${profile.name}, Level: ${profile.level}`);
 *   console.log(`Theme: ${profile.preferences?.theme || 'default'}`);
 * }
 *
 * // Handle missing keys gracefully
 * const config = await storage.getItem('appConfig') || { defaultSetting: true };
 * ```
 * @since 0.1.0
 * @group API
 */
export function getItem<T = any>(key: string): Promise<T | null> {
  return callStorageApi<T | null>('plugin_storage_get', { key });
}

/**
 * Removes a specified key and its associated value from the plugin's persistent storage.
 * @param key - The key of the item to delete
 * @returns Promise that resolves when the operation is complete
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Remove specific items
 * await storage.removeItem('sessionToken');
 * await storage.removeItem('temporaryData');
 *
 * // Remove with error handling
 * try {
 *   await storage.removeItem('userCache');
 *   console.log('Cache cleared successfully');
 * } catch (error) {
 *   console.error('Failed to clear cache:', error.message);
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function removeItem(key: string): Promise<void> {
  return callStorageApi('plugin_storage_remove', { key });
}

/**
 * Clears all persistent storage data for the current plugin.
 * This operation is irreversible and will delete all stored key-value pairs.
 *
 * @returns Promise that resolves when the operation is complete
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Clear all data with confirmation
 * const userConfirmed = await dialog.confirm(
 *   'This will delete all plugin data. Are you sure?'
 * );
 * if (userConfirmed) {
 *   await storage.clear();
 *   console.log('All plugin data has been cleared.');
 * }
 *
 * // Clear data on logout
 * async function logout() {
 *   try {
 *     await storage.clear();
 *     console.log('User data cleared on logout');
 *   } catch (error) {
 *     console.error('Failed to clear data:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function clear(): Promise<void> {
  return callStorageApi('plugin_storage_clear');
}

/**
 * Gets a list of all keys used by the current plugin in persistent storage.
 * @returns Promise that resolves to a string array containing all keys
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // List all stored keys
 * const allKeys = await storage.keys();
 * console.log('All stored keys:', allKeys);
 *
 * // Check if specific data exists
 * const keys = await storage.keys();
 * if (keys.includes('userPreferences')) {
 *   const prefs = await storage.getItem('userPreferences');
 *   console.log('User preferences found:', prefs);
 * }
 *
 * // Data migration example
 * const keys = await storage.keys();
 * const oldVersionKeys = keys.filter(key => key.startsWith('v1_'));
 * if (oldVersionKeys.length > 0) {
 *   console.log('Found old version data, migrating...');
 *   // Perform migration logic
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function keys(): Promise<string[]> {
  return callStorageApi<string[]>('plugin_storage_keys');
}

/**
 * Batch sets multiple key-value pairs to storage.
 * @param items - An object containing multiple key-value pairs
 * @returns Promise that resolves when the operation is complete
 * @throws {PluginError} With code `STORAGE_QUOTA_EXCEEDED` when storage limit is reached
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Batch save settings
 * await storage.setItems({
 *   'theme': 'dark',
 *   'fontSize': 14,
 *   'language': 'en',
 *   'notifications': true
 * });
 *
 * // Save user profile in one operation
 * await storage.setItems({
 *   'user.name': 'John Doe',
 *   'user.email': 'john@example.com',
 *   'user.lastLogin': new Date().toISOString(),
 *   'user.preferences': { theme: 'dark', lang: 'en' }
 * });
 * ```
 * @since 0.1.0
 * @group API
 */
export function setItems(items: Record<string, any>): Promise<void> {
  return callStorageApi('plugin_storage_set_items', { items });
}

/**
 * Batch retrieves values for multiple keys from storage.
 * @typeParam T - The expected type of the retrieved values
 * @param keys - Array of keys to retrieve
 * @returns Promise that resolves to an object containing the requested keys and their corresponding values
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Get multiple settings at once
 * const settings = await storage.getItems(['theme', 'fontSize', 'language']);
 * console.log('Theme:', settings.theme); // 'dark'
 * console.log('Font size:', settings.fontSize); // 14
 * console.log('Language:', settings.language); // 'en'
 *
 * // Get user profile components
 * const userKeys = ['user.name', 'user.email', 'user.preferences'];
 * const userData = await storage.getItems<string | object>(userKeys);
 *
 * // Handle missing keys gracefully
 * const requiredKeys = ['apiKey', 'endpoint', 'timeout'];
 * const config = await storage.getItems(requiredKeys);
 * const apiKey = config.apiKey || 'default-key';
 * const endpoint = config.endpoint || 'https://api.default.com';
 * const timeout = config.timeout || 5000;
 * ```
 * @since 0.1.0
 * @group API
 */
export function getItems<T = any>(keys: string[]): Promise<Record<string, T>> {
  return callStorageApi<Record<string, T>>('plugin_storage_get_items', {
    keys,
  });
}

/**
 * Gets all key-value pairs from the plugin's persistent storage.
 * Returns the complete JSON storage object for the current plugin.
 *
 * @returns Promise that resolves to an object containing all stored key-value pairs
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Get all stored data
 * const allData = await storage.getAll();
 * console.log('All plugin data:', allData);
 *
 * // Check if data exists before processing
 * const data = await storage.getAll();
 * if (Object.keys(data).length > 0) {
 *   console.log('Found', Object.keys(data).length, 'stored items');
 *   // Process data
 * } else {
 *   console.log('No data stored yet');
 * }
 *
 * // Backup all data
 * const backup = await storage.getAll();
 * await fs.writeFile('backup.json', JSON.stringify(backup, null, 2));
 * ```
 * @since 0.1.0
 * @group API
 */
export function getAll(): Promise<Record<string, any>> {
  return callStorageApi<Record<string, any>>('plugin_storage_get_all');
}

/**
 * Replaces all storage data with the provided object.
 * This operation clears existing data and sets new data in a single atomic operation.
 *
 * @param data - An object containing all key-value pairs to store
 * @returns Promise that resolves when the operation is complete
 * @throws {PluginError} With code `STORAGE_QUOTA_EXCEEDED` when storage limit is reached
 * @throws {PluginError} With code `STORAGE_UNAVAILABLE` when storage is not accessible
 * @throws {PluginError} With code `PERMISSION_DENIED` when storage permission is denied
 * @example
 * ```typescript
 * // Replace all data with new configuration
 * await storage.setAll({
 *   version: '2.0',
 *   theme: 'dark',
 *   language: 'en',
 *   user: {
 *     name: 'John Doe',
 *     preferences: { notifications: true }
 *   }
 * });
 *
 * // Restore from backup
 * const backupData = JSON.parse(await fs.readFile('backup.json'));
 * await storage.setAll(backupData);
 *
 * // Migration from old data format
 * const oldData = await storage.getAll();
 * const newData = {
 *   version: '2.0',
 *   settings: {
 *     theme: oldData.theme || 'light',
 *     fontSize: oldData.fontSize || 12
 *   },
 *   migrated: true
 * };
 * await storage.setAll(newData);
 * ```
 * @since 0.1.0
 * @group API
 */
export function setAll(data: Record<string, any>): Promise<void> {
  return callStorageApi('plugin_storage_set_all', { data });
}

/**
 * Storage API namespace - provides persistent key-value storage for plugins
 *
 * Each plugin has its own isolated storage space that persists across application restarts.
 * All data is automatically serialized to JSON format. The storage system supports both
 * individual key operations and batch operations for better performance.
 *
 * **Storage Isolation**: Each plugin can only access its own storage data, ensuring
 * data privacy and security between different plugins.
 *
 * **Data Persistence**: All stored data persists across application restarts and updates,
 * but may be cleared when the plugin is uninstalled.
 *
 * **Storage Limits**: Storage space is limited to prevent abuse. Monitor storage usage
 * and handle quota exceeded errors gracefully.
 *
 * @namespace storage
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @see {@link createStorageError} - For creating storage-specific errors
 * @see {@link isStorageError} - For checking storage error types
 * @example
 * ```typescript
 * import { storage } from 'onin-plugin-sdk';
 *
 * // Basic usage
 * await storage.setItem('username', 'john');
 * const username = await storage.getItem('username');
 *
 * // Type-safe complex data
 * interface UserSettings {
 *   theme: 'light' | 'dark';
 *   fontSize: number;
 *   notifications: boolean;
 * }
 *
 * const settings: UserSettings = {
 *   theme: 'dark',
 *   fontSize: 14,
 *   notifications: true
 * };
 *
 * await storage.setItem('settings', settings);
 * const savedSettings = await storage.getItem<UserSettings>('settings');
 *
 * // Batch operations for better performance
 * await storage.setItems({
 *   'config.apiKey': 'abc123',
 *   'config.endpoint': 'https://api.example.com',
 *   'config.timeout': 5000
 * });
 *
 * const config = await storage.getItems(['config.apiKey', 'config.endpoint']);
 *
 * // Full storage operations
 * const allData = await storage.getAll();
 * await storage.setAll({ version: '2.0', ...newData });
 * ```
 */
export const storage = {
  setItem,
  getItem,
  removeItem,
  clear,
  keys,
  setItems,
  getItems,
  getAll,
  setAll,
};
