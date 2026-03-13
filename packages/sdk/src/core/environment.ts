/**
 * Runtime environment detection module
 * @fileoverview Used to identify whether the current SDK is running in a Headless (Deno) environment or Webview (HTML) environment
 */

/**
 * Supported runtime environment enumeration
 * @enum {string}
 */
export enum RuntimeEnvironment {
  Headless = 'headless' /** Headless Deno environment */,
  Webview = 'webview' /** Webview environment with UI */,
  Unknown = 'unknown',
}

/**
 * Gets the current runtime environment.
 *
 * Determines the current code's runtime environment by checking global objects and runtime characteristics.
 * - If `window.__TAURI_INTERNALS__` object exists, it's considered a Webview environment
 * - If `Deno.core` exists, it's considered a Headless environment (including plugin runtime)
 * - Otherwise, it's considered an unknown environment.
 *
 * @returns The current runtime environment
 */
export function getEnvironment(): RuntimeEnvironment {
  // @ts-ignore
  if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
    return RuntimeEnvironment.Webview;
  }

  // @ts-ignore
  if (typeof Deno !== 'undefined' && Deno.core) {
    return RuntimeEnvironment.Headless;
  }

  return RuntimeEnvironment.Unknown;
}
