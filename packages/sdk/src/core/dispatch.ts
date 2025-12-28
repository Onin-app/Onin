import { getEnvironment, RuntimeEnvironment } from './environment';

/**
 * Defines handler function signatures for different environments
 * @interface Handlers
 * @typeParam T - The return type of the handlers
 * @since 0.1.0
 * @group Core
 */
export interface Handlers<T> {
  webview: () => T;
  headless: () => T;
}

/**
 * Automatically selects and executes the appropriate handler function based on the current runtime environment.
 * 
 * This function provides environment-aware execution, allowing the same code to work seamlessly
 * in both webview (UI) and headless (background) environments. It automatically detects the
 * current runtime and calls the appropriate handler.
 * 
 * @typeParam T - The return type of the handler functions
 * @param handlers - An object containing implementations for both webview and headless environments
 * @returns The execution result of the selected handler function
 * @throws {Error} If the environment is neither webview nor headless
 * @example
 * ```typescript
 * // Different behavior for different environments
 * const result = dispatch({
 *   webview: () => {
 *     // Code that runs in UI environment
 *     return window.someWebAPICall();
 *   },
 *   headless: () => {
 *     // Code that runs in background environment
 *     return Deno.env.get('SOME_VALUE');
 *   }
 * });
 * 
 * // API calls that work in both environments
 * await dispatch({
 *   webview: () => invoke('some_command', args),
 *   headless: () => invoke('some_command', args)
 * });
 * ```
 * @since 0.1.0
 * @group Core
 */
export function dispatch<T>(handlers: Handlers<T>): T {
  const environment = getEnvironment();

  if (environment === RuntimeEnvironment.Webview) {
    return handlers.webview();
  }

  if (environment === RuntimeEnvironment.Headless) {
    return handlers.headless();
  }

  throw new Error(`Unsupported environment: ${environment}`);
}