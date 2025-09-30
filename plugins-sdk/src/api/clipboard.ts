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
async function callClipboardApi<T = any>(method: string, args?: any): Promise<T> {
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
      args
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
  return callClipboardApi<string | null>("plugin_clipboard_read_text");
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
  return callClipboardApi("plugin_clipboard_write_text", { text });
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
  return callClipboardApi<string | null>("plugin_clipboard_read_image");
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
  const data = typeof imageData === 'string' ? imageData : Array.from(imageData);
  return callClipboardApi("plugin_clipboard_write_image", { imageData: data });
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
  return callClipboardApi("plugin_clipboard_clear");
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
 * import { clipboard } from 'baize-plugin-sdk';
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
  
  /** Convenience methods */
  copy,
  paste,
  
  /** Error handling tools */
  parseClipboardError,
};