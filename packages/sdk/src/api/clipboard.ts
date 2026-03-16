import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';
import { errorUtils } from '../types/errors';
import { parseClipboardError } from '../utils/error-parser';

/**
 * Generic clipboard API call helper function
 * @typeParam T - The expected return type
 * @param method - The clipboard method to call
 * @param args - Optional arguments for the method
 * @returns Promise resolving to the method result
 * @internal
 * @group Core
 */
async function callClipboardApi<T = any>(
  method: string,
  args?: any,
): Promise<T> {
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
    throw parseClipboardError(error, {
      method,
      args,
    });
  }
}

/**
 * Reads text content from the clipboard.
 * @returns Promise that resolves to the text in the clipboard, or null if the clipboard is empty or doesn't contain text.
 * @throws {PluginError} With code `CLIPBOARD_UNAVAILABLE` when clipboard is not accessible
 * @throws {PluginError} With code `CLIPBOARD_ACCESS_DENIED` when permission is denied
 * @throws {PluginError} With code `PERMISSION_DENIED` for general permission issues
 * @example
 * ```typescript
 * async function getClipboardText() {
 *   try {
 *     const text = await clipboard.readText();
 *     if (text) {
 *       console.log('Clipboard text:', text);
 *     } else {
 *       console.log('Clipboard is empty or does not contain text.');
 *     }
 *   } catch (error) {
 *     if (errorUtils.isErrorCode(error, 'CLIPBOARD_ACCESS_DENIED')) {
 *       console.error('Clipboard access denied');
 *     }
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function readText(): Promise<string | null> {
  return callClipboardApi<string | null>('plugin_clipboard_read_text');
}

/**
 * Writes the specified text to the clipboard.
 * @param text - The text to write to the clipboard.
 * @returns Promise that resolves when the operation is complete.
 * @throws {PluginError} With code `CLIPBOARD_UNAVAILABLE` when clipboard is not accessible
 * @throws {PluginError} With code `CLIPBOARD_ACCESS_DENIED` when permission is denied
 * @throws {PluginError} With code `PERMISSION_DENIED` for general permission issues
 * @example
 * ```typescript
 * async function setClipboardText() {
 *   try {
 *     await clipboard.writeText('Hello from the plugin!');
 *     console.log('Text written to clipboard.');
 *   } catch (error) {
 *     console.error('Failed to write to clipboard:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function writeText(text: string): Promise<void> {
  return callClipboardApi('plugin_clipboard_write_text', { text });
}

/**
 * Reads image data from the clipboard and returns it as a Base64 string.
 * @returns Promise that resolves to the Base64 encoded string of the image, or null if the clipboard is empty or doesn't contain an image.
 * @throws {PluginError} With code `CLIPBOARD_FORMAT_UNSUPPORTED` when image format is not supported
 * @throws {PluginError} With code `CLIPBOARD_UNAVAILABLE` when clipboard is not accessible
 * @throws {PluginError} With code `CLIPBOARD_ACCESS_DENIED` when permission is denied
 * @example
 * ```typescript
 * async function getClipboardImage() {
 *   try {
 *     const imageBase64 = await clipboard.readImage();
 *     if (imageBase64) {
 *       const imgElement = document.createElement('img');
 *       imgElement.src = `data:image/png;base64,${imageBase64}`;
 *       document.body.appendChild(imgElement);
 *     } else {
 *       console.log('Clipboard does not contain an image.');
 *     }
 *   } catch (error) {
 *     if (errorUtils.isErrorCode(error, 'CLIPBOARD_FORMAT_UNSUPPORTED')) {
 *       console.error('Image format not supported');
 *     }
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function readImage(): Promise<string | null> {
  return callClipboardApi<string | null>('plugin_clipboard_read_image');
}

/**
 * Writes image data to the clipboard.
 * @param imageData - The image's Base64 encoded string or Uint8Array data.
 * @returns Promise that resolves when the operation is complete.
 * @throws {PluginError} With code `CLIPBOARD_FORMAT_UNSUPPORTED` when image format is not supported
 * @throws {PluginError} With code `CLIPBOARD_UNAVAILABLE` when clipboard is not accessible
 * @throws {PluginError} With code `CLIPBOARD_ACCESS_DENIED` when permission is denied
 * @example
 * ```typescript
 * async function setClipboardImage(base64Data: string) {
 *   try {
 *     await clipboard.writeImage(base64Data);
 *     console.log('Image written to clipboard.');
 *   } catch (error) {
 *     console.error('Failed to write image to clipboard:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function writeImage(imageData: string | Uint8Array): Promise<void> {
  const data =
    typeof imageData === 'string' ? imageData : Array.from(imageData);
  return callClipboardApi('plugin_clipboard_write_image', { imageData: data });
}

/**
 * Clears all content from the clipboard.
 * @returns Promise that resolves when the operation is complete.
 * @throws {PluginError} With code `CLIPBOARD_UNAVAILABLE` when clipboard is not accessible
 * @throws {PluginError} With code `CLIPBOARD_ACCESS_DENIED` when permission is denied
 * @example
 * ```typescript
 * async function clearClipboard() {
 *   try {
 *     await clipboard.clear();
 *     console.log('Clipboard cleared.');
 *   } catch (error) {
 *     console.error('Failed to clear clipboard:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function clear(): Promise<void> {
  return callClipboardApi('plugin_clipboard_clear');
}

/**
 * Checks if the clipboard currently contains text content.
 * @returns Promise that resolves to true if the clipboard contains text, false otherwise.
 * @throws {PluginError} Same error conditions as {@link readText}
 * @example
 * ```typescript
 * async function checkText() {
 *   try {
 *     if (await clipboard.hasText()) {
 *       console.log('Clipboard has text.');
 *       const text = await clipboard.readText();
 *       console.log('Text content:', text);
 *     } else {
 *       console.log('Clipboard does not have text.');
 *     }
 *   } catch (error) {
 *     console.error('Failed to check clipboard text:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export async function hasText(): Promise<boolean> {
  const text = await readText();
  return text !== null && text.length > 0;
}

/**
 * Checks if the clipboard currently contains image content.
 * @returns Promise that resolves to true if the clipboard contains an image, false otherwise.
 * @throws {PluginError} Same error conditions as {@link readImage}
 * @example
 * ```typescript
 * async function checkImage() {
 *   try {
 *     if (await clipboard.hasImage()) {
 *       console.log('Clipboard has an image.');
 *       const imageData = await clipboard.readImage();
 *       console.log('Image data available:', !!imageData);
 *     } else {
 *       console.log('Clipboard does not have an image.');
 *     }
 *   } catch (error) {
 *     console.error('Failed to check clipboard image:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export async function hasImage(): Promise<boolean> {
  const image = await readImage();
  return image !== null;
}

/**
 * Copies text to the clipboard, alias for `writeText`.
 * @param text - The text to copy.
 * @returns Promise that resolves when the operation is complete.
 * @throws {PluginError} Same error conditions as {@link writeText}
 * @see {@link writeText} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export function copy(text: string): Promise<void> {
  return writeText(text);
}

/**
 * Pastes text content from the clipboard, alias for `readText`.
 * @returns Promise that resolves to the text in the clipboard, or null if empty.
 * @throws {PluginError} Same error conditions as {@link readText}
 * @see {@link readText} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export function paste(): Promise<string | null> {
  return readText();
}

/**
 * Clipboard content type enumeration
 * @since 0.1.0
 * @group Types
 */
export type ClipboardContentType = 'text' | 'image' | 'files' | 'empty';

/**
 * File information in clipboard
 * @interface ClipboardFile
 * @since 0.1.0
 * @group Types
 */
export interface ClipboardFile {
  /** Full file path */
  path: string;
  /** File name */
  name: string;
  /** Whether this is a directory */
  is_directory: boolean;
}

/**
 * Enhanced metadata about clipboard content including type, timestamp, and age information
 * @interface ClipboardMetadata
 * @since 0.1.0
 * @group Types
 */
export interface ClipboardMetadata {
  /** The text content in the clipboard, or null if no text */
  text: string | null;
  /** List of file paths if clipboard contains files, or null if no files */
  files: ClipboardFile[] | null;
  /** The type of content in the clipboard */
  contentType: ClipboardContentType;
  /**
   * Unix timestamp (in seconds) when the clipboard content was last updated.
   *
   * Note: This timestamp is independent from the app's auto_clear_time_limit setting.
   * Even if the app clears its internal state, this timestamp will still reflect
   * when the system clipboard was actually modified.
   *
   * May be null only if:
   * - Clipboard monitoring hasn't started yet
   * - The clipboard has never been modified since app start
   */
  timestamp: number | null;
  /**
   * Age of the clipboard content in seconds (time since it was copied).
   * Calculated as: current_time - timestamp.
   * Will be null if timestamp is null.
   */
  age: number | null;
}

/**
 * Gets comprehensive clipboard metadata including content type, timestamp, and age.
 * This enhanced API provides detailed information about clipboard content:
 * - Content type detection (text, image, files, or empty)
 * - Text content (if available)
 * - Image detection
 * - File paths (if files were copied)
 * - Timestamp when content was copied
 * - Age in seconds (automatically calculated)
 *
 * @returns Promise that resolves to comprehensive clipboard metadata
 * @throws {PluginError} Same error conditions as {@link readText}
 * @example
 * ```typescript
 * // Basic usage - check content type and age
 * const metadata = await clipboard.getMetadata();
 * console.log('Content type:', metadata.contentType);
 * console.log('Age:', metadata.age, 'seconds');
 *
 * // Handle different content types
 * switch (metadata.contentType) {
 *   case 'text':
 *     console.log('Text:', metadata.text);
 *     break;
 *   case 'image':
 *     console.log('Clipboard contains an image');
 *     break;
 *   case 'files':
 *     console.log('Files:', metadata.files?.map(f => f.name));
 *     break;
 *   case 'empty':
 *     console.log('Clipboard is empty');
 *     break;
 * }
 *
 * // Time-based filtering using age
 * if (metadata.age !== null && metadata.age < 10) {
 *   console.log('Content is fresh (less than 10 seconds old)');
 *   await processContent(metadata);
 * }
 *
 * // Check for image
 * if (metadata.contentType === 'image') {
 *   console.log('Image detected in clipboard');
 * }
 *
 * if (metadata.files && metadata.files.length > 0) {
 *   console.log('Files copied:', metadata.files.length);
 *   metadata.files.forEach(file => {
 *     console.log(`- ${file.name} (${file.is_directory ? 'dir' : 'file'})`);
 *   });
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export function getMetadata(): Promise<ClipboardMetadata> {
  return callClipboardApi<ClipboardMetadata>('plugin_clipboard_get_metadata');
}

/**
 * Clipboard API namespace - provides functions for interacting with the system clipboard
 *
 * Supports reading and writing text and image data to/from the system clipboard.
 * All operations require appropriate permissions and handle various clipboard states gracefully.
 *
 * @namespace clipboard
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @see {@link parseClipboardError} - For error handling utilities
 * @example
 * ```typescript
 * import { clipboard } from 'onin-sdk';
 *
 * // Read text from clipboard
 * const text = await clipboard.readText();
 *
 * // Write text to clipboard
 * await clipboard.writeText('Hello World');
 *
 * // Check if clipboard has content
 * if (await clipboard.hasText()) {
 *   console.log('Clipboard has text content');
 * }
 *
 * // Get metadata with timestamp
 * const metadata = await clipboard.getMetadata();
 * console.log('Content age:', Date.now() / 1000 - metadata.timestamp);
 * ```
 */
export const clipboard = {
  /** Core methods */
  readText,
  writeText,
  readImage,
  writeImage,
  clear,

  /** Check methods */
  hasText,
  hasImage,

  /** Metadata methods */
  getMetadata,

  /** Convenience methods */
  copy,
  paste,

  /** Error handling tools */
  parseClipboardError,
};
