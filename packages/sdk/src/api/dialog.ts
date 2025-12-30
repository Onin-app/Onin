import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';
import { errorUtils } from '../types/errors';
import { parseDialogError } from '../utils/error-parser';

/**
 * Message dialog options
 * @interface MessageDialogOptions
 * @since 0.1.0
 * @group Types
 */
export interface MessageDialogOptions {
  /** Dialog title */
  title?: string;
  /** Main message to display */
  message: string;
  /** Dialog type, affects the displayed icon */
  kind?: 'info' | 'warning' | 'error';
  /** Custom label for the "OK" button */
  okLabel?: string;
}

/**
 * Confirmation dialog options
 * @interface ConfirmDialogOptions
 * @since 0.1.0
 * @group Types
 */
export interface ConfirmDialogOptions {
  /** Dialog title */
  title?: string;
  /** Main message to display */
  message: string;
  /** Dialog type */
  kind?: 'info' | 'warning' | 'error';
  /** Custom label for the "OK" button */
  okLabel?: string;
  /** Custom label for the "Cancel" button */
  cancelLabel?: string;
}

/**
 * File type filter for file dialogs
 * @interface DialogFilter
 * @since 0.1.0
 * @group Types
 */
export interface DialogFilter {
  /** Filter name (e.g., "Image Files") */
  name: string;
  /** Array of file extensions associated with this filter (e.g., ['png', 'jpg']) */
  extensions: string[];
}

/**
 * Open file dialog options
 * @interface OpenDialogOptions
 * @since 0.1.0
 * @group Types
 */
export interface OpenDialogOptions {
  /** Dialog title */
  title?: string;
  /** Default path that should be displayed when the dialog opens */
  defaultPath?: string;
  /** Array of file type filters to apply to file selection */
  filters?: DialogFilter[];
  /** Whether to allow selecting multiple files */
  multiple?: boolean;
  /** Whether to open a directory selector instead of a file selector */
  directory?: boolean;
}

/**
 * Save file dialog options
 * @interface SaveDialogOptions
 * @since 0.1.0
 * @group Types
 */
export interface SaveDialogOptions {
  /** Dialog title */
  title?: string;
  /** Default suggested file path or name in the dialog */
  defaultPath?: string;
  /** Array of file type filters to apply to file saving */
  filters?: DialogFilter[];
}

/**
 * Generic dialog API call helper function
 * @typeParam T - The expected return type
 * @param method - The dialog method to call
 * @param args - Optional arguments for the method
 * @returns Promise resolving to the method result
 * @internal
 * @group Core
 */
async function callDialogApi<T = any>(method: string, args?: any): Promise<T> {
  try {
    return await dispatch({
      webview: () => invoke<T>(method, args),
      headless: () => invoke<T>(method, args),
    });
  } catch (error: any) {
    if (errorUtils.isPluginError(error)) {
      throw error;
    }

    // Use unified error parser
    throw parseDialogError(error, {
      method,
      args
    });
  }
}

/**
 * Shows a standard message dialog.
 * @param options - Dialog configuration options
 * @returns Promise that resolves when the dialog is closed
 * @throws {PluginError} With code `DIALOG_UNAVAILABLE` when dialog system is not available
 * @throws {PluginError} With code `DIALOG_INVALID_OPTIONS` when options are malformed
 * @throws {PluginError} With code `PERMISSION_DENIED` when dialog permission is denied
 * @example
 * ```typescript
 * // Simple info message
 * await dialog.showMessage({
 *   title: 'Info',
 *   message: 'This is an informational message.'
 * });
 * 
 * // Warning message with custom styling
 * await dialog.showMessage({
 *   title: 'Warning',
 *   message: 'This action cannot be undone.',
 *   kind: 'warning',
 *   okLabel: 'I Understand'
 * });
 * 
 * // Error message
 * await dialog.showMessage({
 *   title: 'Error',
 *   message: 'An unexpected error occurred.',
 *   kind: 'error'
 * });
 * ```
 * @since 0.1.0
 * @group API
 */
export function showMessage(options: MessageDialogOptions): Promise<void> {
  return callDialogApi("plugin_dialog_message", options);
}

/**
 * Shows a confirmation dialog with "OK" and "Cancel" buttons.
 * @param options - Dialog configuration options
 * @returns Promise that resolves to true if user clicks "OK", false otherwise
 * @example
 * const confirmed = await dialog.showConfirm({
 *   title: 'Confirm Action',
 *   message: 'Are you sure you want to proceed?'
 * });
 * if (confirmed) {
 *   console.log('User confirmed.');
 * }
 */
export function showConfirm(options: ConfirmDialogOptions): Promise<boolean> {
  return callDialogApi<boolean>("plugin_dialog_confirm", options);
}

/**
 * Shows an open dialog for selecting files or directories.
 * @param options - Dialog configuration options
 * @returns Promise that resolves to the selected file/directory path(s), or null if user cancels
 * @example
 * // Select single file
 * const filePath = await dialog.showOpen({
 *   filters: [{ name: 'Text Files', extensions: ['txt'] }]
 * });
 * if (filePath) {
 *   console.log('Selected file:', filePath);
 * }
 *
 * // Select multiple files
 * const filePaths = await dialog.showOpen({ multiple: true });
 * if (filePaths) {
 *   console.log('Selected files:', filePaths);
 * }
 */
export async function showOpen(options?: OpenDialogOptions): Promise<string | string[] | null> {
  const result = await callDialogApi<any>("plugin_dialog_open", options || {});
  
  // Handle return value type conversion
  if (result === null || result === undefined) {
    return null;
  }
  
  // If it's an array, it means multiple file selection
  if (Array.isArray(result)) {
    return result as string[];
  }
  
  // If it's a string, it means single file selection
  if (typeof result === 'string') {
    return result;
  }
  
  return null;
}

/**
 * Shows a save dialog for selecting a file save path.
 * @param options - Dialog configuration options
 * @returns Promise that resolves to the user-selected save path, or null if user cancels
 * @example
 * const savePath = await dialog.showSave({
 *   defaultPath: 'new-file.txt',
 *   filters: [{ name: 'Text Files', extensions: ['txt'] }]
 * });
 * if (savePath) {
 *   console.log('File will be saved to:', savePath);
 * }
 */
export function showSave(options?: SaveDialogOptions): Promise<string | null> {
  return callDialogApi<string | null>("plugin_dialog_save", options || {});
}

/**
 * Shows an info message
 * @param message - Message content
 * @param title - Title (optional)
 */
export function info(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'info'
  });
}

/**
 * Shows a warning message
 * @param message - Message content
 * @param title - Title (optional)
 */
export function warning(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'warning'
  });
}

/**
 * Shows an error message
 * @param message - Message content
 * @param title - Title (optional)
 */
export function error(message: string, title?: string): Promise<void> {
  return showMessage({
    message,
    title,
    kind: 'error'
  });
}

/**
 * Shows a confirmation dialog (simplified version)
 * @param message - Message content
 * @param title - Title (optional)
 * @returns Whether the user clicked the confirm button
 */
export function confirm(message: string, title?: string): Promise<boolean> {
  return showConfirm({
    message,
    title
  });
}

/**
 * Selects a single file
 * @param filters - File filters (optional)
 * @param defaultPath - Default path (optional)
 * @returns Selected file path, or null if cancelled
 */
export function selectFile(filters?: DialogFilter[], defaultPath?: string): Promise<string | null> {
  return showOpen({
    filters,
    defaultPath,
    multiple: false,
    directory: false
  }) as Promise<string | null>;
}

/**
 * Selects multiple files
 * @param filters - File filters (optional)
 * @param defaultPath - Default path (optional)
 * @returns Array of selected file paths, or null if cancelled
 */
export function selectFiles(filters?: DialogFilter[], defaultPath?: string): Promise<string[] | null> {
  return showOpen({
    filters,
    defaultPath,
    multiple: true,
    directory: false
  }) as Promise<string[] | null>;
}

/**
 * Selects a folder
 * @param defaultPath - Default path (optional)
 * @returns Selected folder path, or null if cancelled
 */
export function selectFolder(defaultPath?: string): Promise<string | null> {
  return showOpen({
    defaultPath,
    multiple: false,
    directory: true
  }) as Promise<string | null>;
}

/**
 * Save file dialog (simplified version)
 * @param defaultName - Default file name (optional)
 * @param filters - File filters (optional)
 * @returns Selected save path, or null if cancelled
 */
export function saveFile(defaultName?: string, filters?: DialogFilter[]): Promise<string | null> {
  return showSave({
    defaultPath: defaultName,
    filters
  });
}

/**
 * Dialog API namespace - provides native system dialog functionality
 * 
 * Supports various types of system dialogs including message boxes, file pickers,
 * and confirmation dialogs. All dialogs are native system dialogs that respect
 * the user's operating system theme and accessibility settings.
 * 
 * **Cross-Platform Support**: All dialog functions work consistently across
 * Windows, macOS, and Linux with platform-appropriate styling.
 * 
 * **Accessibility**: Native dialogs automatically support screen readers and
 * keyboard navigation according to system accessibility settings.
 * 
 * **User Experience**: Dialogs are modal and will block plugin execution until
 * the user responds, ensuring proper user interaction flow.
 * 
 * @namespace dialog
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @see {@link parseDialogError} - For dialog error handling utilities
 * @example
 * ```typescript
 * import { dialog } from 'baize-plugin-sdk';
 * 
 * // Show information message
 * await dialog.info('Operation completed successfully!');
 * 
 * // Get user confirmation
 * const confirmed = await dialog.confirm(
 *   'Are you sure you want to delete this item?'
 * );
 * if (confirmed) {
 *   // Proceed with deletion
 * }
 * 
 * // File selection
 * const filePath = await dialog.selectFile([
 *   { name: 'Text Files', extensions: ['txt', 'md'] },
 *   { name: 'All Files', extensions: ['*'] }
 * ]);
 * if (filePath) {
 *   console.log('Selected file:', filePath);
 * }
 * 
 * // Save file dialog
 * const savePath = await dialog.saveFile('document.txt', [
 *   { name: 'Text Files', extensions: ['txt'] }
 * ]);
 * if (savePath) {
 *   // Save file to the selected path
 * }
 * ```
 */
export const dialog = {
  /** Core methods */
  showMessage,
  showConfirm,
  showOpen,
  showSave,
  
  /** Convenience methods */
  info,
  warning,
  error,
  confirm,
  selectFile,
  selectFiles,
  selectFolder,
  saveFile,
  
  /** Error handling tools */
  parseDialogError,
};