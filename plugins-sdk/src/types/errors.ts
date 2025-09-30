/**
 * Error code definitions organized by namespace
 * 
 * Provides a comprehensive set of error codes organized by functional area.
 * Each error code follows a consistent naming pattern: `CATEGORY_SPECIFIC_ERROR`.
 * This organization helps with error handling and debugging.
 * 
 * @fileoverview Defines all error codes and error handling utilities for the plugin SDK
 * @version 0.1.0
 * @since 0.1.0
 * @group Types
 */
export const errorCode = {
  /** Common errors */
  common: {
    UNKNOWN: 'COMMON_UNKNOWN',
    PERMISSION_DENIED: 'COMMON_PERMISSION_DENIED',
    INVALID_ARGUMENT: 'COMMON_INVALID_ARGUMENT',
  },

  /** HTTP related errors */
  http: {
    NETWORK_ERROR: 'HTTP_NETWORK_ERROR',
    TIMEOUT: 'HTTP_TIMEOUT',
    HTTP_ERROR: 'HTTP_HTTP_ERROR', // Generic HTTP error with status code information
  },

  /** File system errors */
  fs: {
    FILE_NOT_FOUND: 'FS_FILE_NOT_FOUND',
    FILE_ACCESS_DENIED: 'FS_FILE_ACCESS_DENIED',
    DIRECTORY_NOT_FOUND: 'FS_DIRECTORY_NOT_FOUND',
    DISK_FULL: 'FS_DISK_FULL',
    FILE_ALREADY_EXISTS: 'FS_FILE_ALREADY_EXISTS',
    INVALID_PATH: 'FS_INVALID_PATH',
    READ_ONLY_FILESYSTEM: 'FS_READ_ONLY_FILESYSTEM',
  },

  /** Clipboard errors */
  clipboard: {
    UNAVAILABLE: 'CLIPBOARD_UNAVAILABLE',
    FORMAT_UNSUPPORTED: 'CLIPBOARD_FORMAT_UNSUPPORTED',
    EMPTY: 'CLIPBOARD_EMPTY',
    ACCESS_DENIED: 'CLIPBOARD_ACCESS_DENIED',
  },

  /** Dialog errors */
  dialog: {
    CANCELLED: 'DIALOG_CANCELLED',
    UNAVAILABLE: 'DIALOG_UNAVAILABLE',
    INVALID_OPTIONS: 'DIALOG_INVALID_OPTIONS',
  },

  /** Storage errors */
  storage: {
    QUOTA_EXCEEDED: 'STORAGE_QUOTA_EXCEEDED',
    UNAVAILABLE: 'STORAGE_UNAVAILABLE',
  },
} as const;

/**
 * Flattened error code type union
 */
export type ErrorCode =
  | typeof errorCode.common[keyof typeof errorCode.common]
  | typeof errorCode.http[keyof typeof errorCode.http]
  | typeof errorCode.fs[keyof typeof errorCode.fs]
  | typeof errorCode.clipboard[keyof typeof errorCode.clipboard]
  | typeof errorCode.dialog[keyof typeof errorCode.dialog]
  | typeof errorCode.storage[keyof typeof errorCode.storage];

/**
 * Plugin error interface - uses plain objects instead of classes
 * 
 * Represents errors that occur within the plugin system. These errors provide
 * structured information including error codes, context data, and human-readable
 * messages for better debugging and error handling.
 * 
 * @interface PluginError
 * @extends Error
 * @since 0.1.0
 * @group Types
 */
export interface PluginError extends Error {
  readonly name: 'PluginError';
  readonly code: ErrorCode;
  readonly context?: Record<string, any>;
}

/**
 * Factory function for creating plugin errors
 * @param code - The error code
 * @param message - The error message
 * @param context - Additional context information
 * @returns A new PluginError instance
 * @internal
 * @group Error Factories
 */
function createPluginError(
  code: ErrorCode,
  message: string,
  context?: Record<string, any>
): PluginError {
  const error = new Error(message) as PluginError;
  Object.defineProperty(error, 'name', { value: 'PluginError', writable: false });
  (error as any).code = code;
  (error as any).context = context;
  return error;
}

/**
 * Error factory functions organized by namespace
 * 
 * Provides convenient factory functions for creating specific types of plugin errors.
 * Each namespace corresponds to a functional area (HTTP, file system, clipboard, etc.)
 * and contains functions for creating the most common error scenarios.
 * 
 * @namespace createError
 * @version 0.1.0
 * @since 0.1.0
 * @group Error Factories
 * @example
 * ```typescript
 * // Create HTTP errors
 * throw createError.http.networkError('Connection failed');
 * throw createError.http.timeout('https://api.example.com', 5000);
 * 
 * // Create file system errors
 * throw createError.fs.fileNotFound('/path/to/file.txt');
 * throw createError.fs.accessDenied('/restricted/path');
 * 
 * // Create clipboard errors
 * throw createError.clipboard.formatUnsupported('image/gif');
 * 
 * // Create with additional context
 * throw createError.common.unknown('Unexpected error', {
 *   operation: 'data-processing',
 *   timestamp: Date.now()
 * });
 * ```
 */
export const createError = {
  /** Common error factories */
  common: {
    unknown: (message: string, context?: Record<string, any>) =>
      createPluginError(errorCode.common.UNKNOWN, message, context),

    permissionDenied: (resource: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.common.PERMISSION_DENIED,
        `Permission denied for ${resource}`,
        context
      ),

    invalidArgument: (argument: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.common.INVALID_ARGUMENT,
        `Invalid argument: ${argument}`,
        context
      ),
  },

  /** HTTP error factories */
  http: {
    networkError: (message: string, context?: Record<string, any>) =>
      createPluginError(errorCode.http.NETWORK_ERROR, message, context),

    timeout: (url: string, timeout: number, context?: Record<string, any>) =>
      createPluginError(
        errorCode.http.TIMEOUT,
        `Request to ${url} timed out after ${timeout}ms`,
        { url, timeout, ...context }
      ),

    httpError: (status: number, statusText: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.http.HTTP_ERROR,
        `HTTP ${status}: ${statusText}`,
        { status, statusText, ...context }
      ),
  },

  /** File system error factories */
  fs: {
    fileNotFound: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_NOT_FOUND,
        `File not found: ${path}`,
        { path, ...context }
      ),

    fileAccessDenied: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_ACCESS_DENIED,
        `Access denied: ${path}`,
        { path, ...context }
      ),

    diskFull: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.DISK_FULL,
        `Disk full while accessing: ${path}`,
        { path, ...context }
      ),

    fileAlreadyExists: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_ALREADY_EXISTS,
        `File already exists: ${path}`,
        { path, ...context }
      ),

    invalidPath: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.INVALID_PATH,
        `Invalid path: ${path}`,
        { path, ...context }
      ),
  },

  /** Clipboard error factories */
  clipboard: {
    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.UNAVAILABLE,
        'Clipboard is not available',
        context
      ),

    formatUnsupported: (format?: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.FORMAT_UNSUPPORTED,
        `Clipboard format not supported${format ? `: ${format}` : ''}`,
        { format, ...context }
      ),

    empty: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.EMPTY,
        'Clipboard is empty',
        context
      ),

    accessDenied: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.ACCESS_DENIED,
        'Clipboard access denied',
        context
      ),
  },

  /** Dialog error factories */
  dialog: {
    cancelled: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.CANCELLED,
        'Dialog was cancelled by user',
        context
      ),

    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.UNAVAILABLE,
        'Dialog is not available',
        context
      ),

    invalidOptions: (reason?: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.INVALID_OPTIONS,
        `Invalid dialog options${reason ? `: ${reason}` : ''}`,
        { reason, ...context }
      ),
  },

  /** Storage error factories */
  storage: {
    quotaExceeded: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.storage.QUOTA_EXCEEDED,
        'Storage quota exceeded',
        context
      ),

    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.storage.UNAVAILABLE,
        'Storage is not available',
        context
      ),
  },
};

/**
 * Error checking utility functions
 * 
 * Provides utility functions for checking and analyzing plugin errors.
 * These functions help identify error types, extract error information,
 * and implement error-specific handling logic.
 * 
 * @namespace errorUtils
 * @version 0.1.0
 * @since 0.1.0
 * @group Error Utilities
 * @example
 * ```typescript
 * try {
 *   await someOperation();
 * } catch (error) {
 *   if (errorUtils.isPluginError(error)) {
 *     console.log('Plugin error:', error.code);
 *     
 *     if (errorUtils.isErrorCode(error, 'HTTP_NETWORK_ERROR')) {
 *       console.log('Network error occurred');
 *     }
 *     
 *     const info = errorUtils.getErrorInfo(error);
 *     console.log('Error details:', info);
 *   } else {
 *     console.log('Unknown error:', error);
 *   }
 * }
 * ```
 */
export const errorUtils = {
  /**
   * Checks if an error is a plugin error
   * @param error - The error to check
   * @returns True if the error is a PluginError
   */
  isPluginError: (error: any): error is PluginError => {
    return error && error.name === 'PluginError' && typeof error.code === 'string';
  },

  /**
   * Checks if an error is of a specific type
   * @param error - The error to check
   * @param code - The error code to match
   * @returns True if the error matches the specified code
   */
  isErrorCode: (error: any, code: ErrorCode): boolean => {
    return errorUtils.isPluginError(error) && error.code === code;
  },

  /**
   * Checks if an error is one of the specified types
   * @param error - The error to check
   * @param codes - Array of error codes to match against
   * @returns True if the error matches any of the specified codes
   */
  isOneOfErrorCodes: (error: any, codes: ErrorCode[]): boolean => {
    return errorUtils.isPluginError(error) && codes.includes(error.code);
  },

  /**
   * Gets detailed information about an error
   * @param error - The error to analyze
   * @returns Error information object or null if not a plugin error
   */
  getErrorInfo: (error: any) => {
    if (errorUtils.isPluginError(error)) {
      return {
        name: error.name,
        code: error.code,
        message: error.message,
        context: error.context,
      };
    }
    return null;
  },
};