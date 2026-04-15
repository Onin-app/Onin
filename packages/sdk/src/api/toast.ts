import { invoke } from '../core/ipc';

/**
 * Toast notification type
 * @since 0.1.0
 * @group Types
 */
export type ToastType = 'default' | 'success' | 'error' | 'warning' | 'info';

/**
 * Toast options
 * @interface ToastOptions
 * @since 0.1.0
 * @group Types
 */
export interface ToastOptions {
  /** Toast type */
  kind?: ToastType;
  /** Duration in milliseconds */
  duration?: number;
}

/**
 * Shows a toast message in the current window.
 * 
 * Toasts are non-blocking, transient UI elements that appear for a short duration
 * and then disappear automatically. They are ideal for providing brief feedback
 * on user actions like "Saved successfully" or "File copied".
 * 
 * @param message - Message content
 * @param options - Optional configuration including type and duration
 * @returns Promise that resolves when the toast request is sent
 * @example
 * ```typescript
 * import { toast } from 'onin-sdk';
 * 
 * // Basic info toast
 * await toast.show('Hello World!');
 * 
 * // Success toast with custom duration
 * await toast.success('Operation complete!', { duration: 5000 });
 * ```
 * @since 0.1.0
 * @group API
 */
export async function showToast(message: string, options?: ToastOptions): Promise<void> {
  return invoke('plugin_toast', { 
    message, 
    kind: options?.kind || 'default',
    duration: options?.duration
  });
}

/**
 * Shows a success toast
 * @param message - Message content
 * @param options - Toast options (excluding kind)
 */
export function success(message: string, options?: Omit<ToastOptions, 'kind'>): Promise<void> {
  return showToast(message, { ...options, kind: 'success' });
}

/**
 * Shows an error toast
 * @param message - Message content
 * @param options - Toast options (excluding kind)
 */
export function error(message: string, options?: Omit<ToastOptions, 'kind'>): Promise<void> {
  return showToast(message, { ...options, kind: 'error' });
}

/**
 * Shows a warning toast
 * @param message - Message content
 * @param options - Toast options (excluding kind)
 */
export function warning(message: string, options?: Omit<ToastOptions, 'kind'>): Promise<void> {
  return showToast(message, { ...options, kind: 'warning' });
}

/**
 * Shows an info toast
 * @param message - Message content
 * @param options - Toast options (excluding kind)
 */
export function info(message: string, options?: Omit<ToastOptions, 'kind'>): Promise<void> {
  return showToast(message, { ...options, kind: 'info' });
}

/**
 * Toast API namespace - provides in-app toast notification functionality
 * 
 * Toasts appear within the current window and are used for transient feedback.
 * Unlike system notifications, they don't persist in the system notification center.
 * 
 * @namespace toast
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 */
export const toast = {
  show: showToast,
  success,
  error,
  warning,
  info,
};
