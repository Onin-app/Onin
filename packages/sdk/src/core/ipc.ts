import { getEnvironment, RuntimeEnvironment } from './environment';
import {
  type EventName,
  type EventCallback,
  type UnlistenFn,
} from '@tauri-apps/api/event';

/** --- invoke --- */

/** Cache for the imported invoke function */
let invokeFn: ((cmd: string, args?: any) => Promise<any>) | null = null;

/** Message ID counter for postMessage calls */
let messageIdCounter = 0;

/** Pending postMessage calls waiting for response */
const pendingCalls = new Map<string, { resolve: (value: any) => void; reject: (error: Error) => void }>();

/**
 * Setup postMessage response listener (only once)
 */
let postMessageListenerSetup = false;
function setupPostMessageListener() {
  if (postMessageListenerSetup) return;
  postMessageListenerSetup = true;

  window.addEventListener('message', (event) => {
    const data = event.data;
    if (!data || typeof data !== 'object') return;
    if (!data.messageId || !pendingCalls.has(data.messageId)) return;

    const pending = pendingCalls.get(data.messageId)!;
    pendingCalls.delete(data.messageId);

    if (data.error) {
      pending.reject(new Error(data.error));
    } else {
      pending.resolve(data.result);
    }
  });
}

/**
 * Asynchronously loads and caches the invoke function
 * @returns The invoke function for the current environment
 * @internal
 * @group Core
 */
async function loadInvoke() {
  if (invokeFn) {
    return invokeFn;
  }

  const environment = getEnvironment();
  if (environment === RuntimeEnvironment.Webview) {
    const { invoke } = await import('@tauri-apps/api/core');
    invokeFn = invoke;
  } else if (environment === RuntimeEnvironment.Iframe) {
    // Iframe environment: use postMessage to communicate with parent window
    setupPostMessageListener();

    invokeFn = async <T>(method: string, args: any): Promise<T> => {
      return new Promise((resolve, reject) => {
        const messageId = `invoke_${++messageIdCounter}_${Date.now()}`;

        pendingCalls.set(messageId, { resolve, reject });

        // Send invoke request to parent window
        window.parent.postMessage({
          type: 'plugin-tauri-call',
          messageId,
          command: 'invoke',
          args: [method, args],
        }, '*');

        // Timeout after 30 seconds
        setTimeout(() => {
          if (pendingCalls.has(messageId)) {
            pendingCalls.delete(messageId);
            reject(new Error(`Invoke timeout for method: ${method}`));
          }
        }, 30000);
      });
    };
  } else if (environment === RuntimeEnvironment.Headless) {
    // Headless environment invoke implementation defined directly here
    invokeFn = async <T>(method: string, arg: any): Promise<T> => {
      // @ts-ignore: Deno.core is injected by the Deno runtime in Rust.
      const result = await Deno.core.ops.op_invoke(method, arg);

      // Handle InvokeResult enum
      if (result && typeof result === 'object') {
        if (result.type === 'error') {
          throw new Error(result.error);
        } else if (result.type === 'ok') {
          return result.value as T;
        }
      }

      // Compatibility with old format
      if (result && typeof result === 'object' && 'error' in result) {
        throw new Error(result.error);
      }

      return result as T;
    };
  } else {
    throw new Error('Unsupported runtime environment for invoke');
  }
  return invokeFn;
}

/**
 * Cross-environment invoke function for calling Tauri commands
 *
 * Provides a unified interface for invoking Tauri commands that works seamlessly
 * in both webview and headless environments. Automatically handles environment
 * detection and uses the appropriate underlying implementation.
 *
 * @typeParam T - The expected return type
 * @param method - The method name to invoke
 * @param arg - Arguments to pass to the method
 * @returns Promise resolving to the method result
 * @throws {Error} When the method call fails or returns an error
 * @example
 * ```typescript
 * // Simple method call
 * const result = await invoke('get_app_version');
 *
 * // Method call with arguments
 * const response = await invoke<UserData>('get_user_data', { userId: 123 });
 *
 * // Error handling
 * try {
 *   const data = await invoke('risky_operation', { param: 'value' });
 *   console.log('Success:', data);
 * } catch (error) {
 *   console.error('Operation failed:', error.message);
 * }
 * ```
 * @since 0.1.0
 * @group Core
 */
export async function invoke<T>(method: string, arg: any): Promise<T> {
  const fn = await loadInvoke();
  if (!fn) {
    throw new Error('Invoke function not loaded');
  }
  return fn(method, arg);
}

/** --- listen --- */

/** Cache for the imported listen function */
let listenFn:
  | ((event: EventName, handler: EventCallback<any>) => Promise<UnlistenFn>)
  | null = null;

/**
 * Asynchronously loads and caches the listen function
 * @returns The listen function for the current environment
 * @internal
 * @group Core
 */
async function loadListen() {
  if (listenFn) {
    return listenFn;
  }

  const environment = getEnvironment();
  if (environment === RuntimeEnvironment.Webview) {
    const { listen } = await import('@tauri-apps/api/event');
    listenFn = listen;
  } else if (environment === RuntimeEnvironment.Iframe) {
    // Iframe environment: use postMessage to listen for events from parent window
    // Store event handlers
    const eventHandlers = new Map<string, Set<EventCallback<any>>>();

    // Setup listener for events from parent
    window.addEventListener('message', (event) => {
      const data = event.data;
      if (!data || typeof data !== 'object') return;

      // Handle stream events (for AI streaming)
      if (data.type === 'plugin-event' && data.eventName) {
        const handlers = eventHandlers.get(data.eventName);
        if (handlers) {
          handlers.forEach(handler => {
            handler({ payload: data.payload } as any);
          });
        }
      }
    });

    listenFn = (
      event: EventName,
      handler: EventCallback<any>,
    ): Promise<UnlistenFn> => {
      const eventName = event.toString();

      if (!eventHandlers.has(eventName)) {
        eventHandlers.set(eventName, new Set());
      }
      eventHandlers.get(eventName)!.add(handler);

      // Return unlisten function
      return Promise.resolve(() => {
        const handlers = eventHandlers.get(eventName);
        if (handlers) {
          handlers.delete(handler);
        }
      });
    };
  } else if (environment === RuntimeEnvironment.Headless) {
    // Headless environment simulates "listening" to specific events by mounting global variables
    listenFn = (
      event: EventName,
      handler: EventCallback<any>,
    ): Promise<UnlistenFn> => {
      if (event === 'plugin_command_execute') {
        // This is special handling logic for registerCommandHandler
        (globalThis as any).__ONIN_COMMAND_HANDLER__ = handler;
        // Headless mode has no concept of unlisten, return an empty function
        return Promise.resolve(() => { });
      }

      console.warn(
        `Event listening for '${event.toString()}' is not supported in headless mode.`,
      );
      return Promise.resolve(() => { }); // Return an empty unlisten function
    };
  } else {
    throw new Error('Unsupported runtime environment for listen');
  }
  return listenFn;
}

/**
 * Cross-environment event listener for Tauri events
 *
 * Provides a unified interface for listening to Tauri events that works in both
 * webview and headless environments. In webview mode, it uses the standard Tauri
 * event system. In headless mode, it provides limited event support for specific
 * plugin-related events.
 *
 * @typeParam T - The event payload type
 * @param event - The event name to listen for
 * @param handler - The event handler function
 * @returns Promise resolving to an unlisten function
 * @throws {Error} When event listening setup fails
 * @example
 * ```typescript
 * // Listen for custom events
 * const unlisten = await listen<string>('my-event', (event) => {
 *   console.log('Received event:', event.payload);
 * });
 *
 * // Listen for plugin command execution (special case)
 * await listen('plugin_command_execute', async (event) => {
 *   const { command, args } = event.payload;
 *   console.log(`Executing command: ${command}`, args);
 * });
 *
 * // Clean up listener when done
 * unlisten();
 * ```
 * @since 0.1.0
 * @group Core
 */
export async function listen<T>(
  event: EventName,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  const fn = await loadListen();
  if (!fn) {
    throw new Error('Listen function not loaded');
  }
  return fn(event, handler);
}
