/**
 * Baize Plugin SDK - Main entry point providing unified API access
 * 
 * This SDK provides a comprehensive set of APIs for developing plugins for the Baize application.
 * It includes file system operations, HTTP requests, clipboard management, system dialogs,
 * notifications, storage, and command handling - all within a secure sandboxed environment.
 * 
 * **Key Features:**
 * - Cross-platform compatibility (Windows, macOS, Linux)
 * - Type-safe APIs with comprehensive TypeScript definitions
 * - Robust error handling with detailed error codes
 * - Environment-aware execution (webview and headless)
 * - Sandboxed file system access
 * - Persistent storage with plugin isolation
 * 
 * **Security Model:**
 * - All APIs respect plugin permissions
 * - File system access is restricted to plugin data directory
 * - Network requests can be controlled by permission policies
 * - Storage is isolated per plugin
 * 
 * @fileoverview Main entry point for the plugin SDK, exports all available APIs
 * @version 0.1.0
 * @since 0.1.0
 * @author Baize Team
 * @see {@link https://github.com/baize-team/plugin-sdk} - Official repository
 * @example
 * ```typescript
 * import { 
 *   http, 
 *   storage, 
 *   fs, 
 *   notification, 
 *   dialog, 
 *   clipboard, 
 *   command 
 * } from 'baize-plugin-sdk';
 * 
 * // HTTP requests
 * const response = await http.get('https://api.example.com/data');
 * 
 * // Persistent storage
 * await storage.setItem('config', { theme: 'dark' });
 * const config = await storage.getItem('config');
 * 
 * // File operations
 * await fs.writeFile('data.json', JSON.stringify(response.body));
 * 
 * // User notifications
 * await notification.show({
 *   title: 'Data Updated',
 *   body: 'Successfully fetched and saved new data'
 * });
 * 
 * // Command handling
 * await command.register(async (cmd, args) => {
 *   if (cmd === 'get-status') {
 *     return { status: 'ready', config };
 *   }
 * });
 * ```
 */
// Direct import of namespace objects from each module
import { http } from './api/request';
import { storage } from './api/storage';
import { notification } from './api/notification';
import { command } from './api/command';
import { fs } from './api/fs';
import { dialog } from './api/dialog';
import { clipboard } from './api/clipboard';

import { invoke, listen } from './core/ipc';
import { debug } from './utils/debug'
// Export error handling utilities
import * as error from './types/errors';
// Export retry mechanism utilities
import * as retry from './utils/retry';

// Import all types for organization
import type * as Permissions from './types/permissions';
import type * as Errors from './types/errors';
import type * as Retry from './utils/retry';
import type * as Fs from './api/fs';
import type * as Dialog from './api/dialog';

/**
 * Contains all available type definitions for the SDK
 * 
 * Provides access to TypeScript type definitions for all SDK components.
 * Use these types for better type safety and IDE support when working with
 * the SDK APIs.
 * 
 * @namespace types
 * @version 0.1.0
 * @since 0.1.0
 * @group Types
 * @example
 * ```typescript
 * import { types } from 'baize-plugin-sdk';
 * 
 * // Use error types
 * function handleError(error: types.Errors.PluginError) {
 *   console.log('Error code:', error.code);
 * }
 * 
 * // Use file system types
 * function processFile(info: types.Fs.FileInfo) {
 *   console.log('File size:', info.size);
 * }
 * 
 * // Use dialog types
 * const filters: types.Dialog.DialogFilter[] = [
 *   { name: 'Text Files', extensions: ['txt', 'md'] }
 * ];
 * ```
 */
const types = {
  Permissions: {} as typeof Permissions,
  Errors: {} as typeof Errors,
  Retry: {} as typeof Retry,
  Fs: {} as typeof Fs,
  Dialog: {} as typeof Dialog,
};

export {
  http,
  storage,
  notification,
  command,
  fs,
  dialog,
  clipboard,
  invoke,
  listen,
  debug,
  error,
  retry,
  types,
};
