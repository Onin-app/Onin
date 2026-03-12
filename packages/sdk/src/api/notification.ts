import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

/**
 * System notification options
 * @interface NotificationOptions
 * @since 0.1.0
 * @group Types
 */
export interface NotificationOptions {
  /** Notification title */
  title: string;
  /** Notification body content */
  body: string;
  /** Optional icon path or URL */
  icon?: string;
  /** Optional sound name (e.g. 'default') */
  sound?: string;
}

/**
 * Shows a system notification.
 * @param options - Notification options including title and content
 * @returns Promise that resolves when the notification display operation is complete
 * @throws {PluginError} With code `PERMISSION_DENIED` when notification permission is denied
 * @throws {PluginError} With code `COMMON_UNKNOWN` for other notification failures
 * @example
 * ```typescript
 * import { notification } from 'onin-plugin-sdk';
 *
 * // Simple notification
 * await notification.show({
 *   title: 'Task Complete',
 *   body: 'Your data processing task has finished successfully.'
 * });
 *
 * // Notification with error handling
 * try {
 *   await notification.show({
 *     title: 'Update Available',
 *     body: 'A new version of the plugin is available for download.'
 *   });
 * } catch (error) {
 *   if (errorUtils.isErrorCode(error, 'PERMISSION_DENIED')) {
 *     console.error('Notification permission denied');
 *   } else {
 *     console.error('Failed to show notification:', error.message);
 *   }
 * }
 *
 * // Progress notification
 * await notification.show({
 *   title: 'Processing',
 *   body: `Processing file ${currentFile} of ${totalFiles}...`
 * });
 * ```
 * @since 0.1.0
 * @group API
 */
export function showNotification(options: NotificationOptions): Promise<void> {
  return dispatch({
    // Webview environment: Tauri expects parameter structure { options: { ... } }
    webview: () => invoke('show_notification', { options }),
    // Headless environment: pass options directly
    headless: () => invoke('show_notification', options),
  });
}

/**
 * Checks if notification permission is granted.
 * @returns Promise that resolves to true if granted, false otherwise
 * @since 0.1.0
 * @group API
 */
export function isPermissionGranted(): Promise<boolean> {
  return invoke<boolean>('is_permission_granted');
}

/**
 * Requests notification permission.
 * @returns Promise that resolves to the permission state ('granted', 'denied', or 'default')
 * @since 0.1.0
 * @group API
 */
export function requestPermission(): Promise<'granted' | 'denied' | 'default'> {
  return invoke<'granted' | 'denied' | 'default'>('request_permission');
}

/**
 * Simplified method name alias
 * @see {@link showNotification} - For detailed documentation
 * @since 0.1.0
 * @group API
 */
export const show = showNotification;

/**
 * Notification API namespace - provides system notification functionality
 *
 * Allows plugins to display native system notifications to inform users about
 * important events, task completion, or status updates. Notifications appear
 * according to the user's system notification settings.
 *
 * **Platform Support**: Works across Windows, macOS, and Linux with platform-
 * appropriate notification styling and behavior.
 *
 * **User Control**: Notifications respect user's system settings including
 * Do Not Disturb mode and notification permissions.
 *
 * **Non-Blocking**: Notification display is asynchronous and won't block plugin
 * execution. Users can dismiss notifications at any time.
 *
 * @namespace notification
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @example
 * ```typescript
 * import { notification } from 'onin-plugin-sdk';
 *
 * // Basic usage
 * await notification.show({
 *   title: 'Hello',
 *   body: 'Plugin loaded successfully!'
 * });
 *
 * // Using the alias method
 * await notification.show({
 *   title: 'Download Complete',
 *   body: 'Your file has been downloaded to the Downloads folder.'
 * });
 *
 * // Status updates
 * async function processFiles(files: string[]) {
 *   for (let i = 0; i < files.length; i++) {
 *     await processFile(files[i]);
 *
 *     // Show progress notification
 *     await notification.show({
 *       title: 'Processing Files',
 *       body: `Completed ${i + 1} of ${files.length} files`
 *     });
 *   }
 *
 *   // Show completion notification
 *   await notification.show({
 *     title: 'All Done!',
 *     body: `Successfully processed ${files.length} files`
 *   });
 * }
 * ```
 */
export const notification = {
  show: showNotification,
  isPermissionGranted,
  requestPermission,
};
