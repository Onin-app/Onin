import { invoke, listen } from '../core/ipc';
import { dispatch } from '../core/dispatch';

/**
 * Command handler function type - defines the signature for handling incoming commands
 * @param command - The command name to handle
 * @param args - Arguments passed with the command
 * @returns The result of the command execution (can be synchronous or asynchronous)
 * @since 0.1.0
 * @group Types
 */
export type CommandHandler = (command: string, args: any) => any | Promise<any>;

/**
 * Command keyword definition
 * @interface CommandKeyword
 * @since 0.2.0
 */
export interface CommandKeyword {
  /** Keyword name */
  name: string;
  /** Keyword type: "prefix" | "fuzzy" | "exact" */
  type?: string;
}

/**
 * Command match definition for content-based matching
 * @interface CommandMatchDefinition
 * @since 0.2.0
 */
export interface CommandMatchDefinition {
  /** Match type: "text" | "image" | "file" | "folder" */
  type: 'text' | 'image' | 'file' | 'folder';
  /** Match name */
  name: string;
  /** Match description */
  description?: string;
  /** Regular expression for text matching */
  regexp?: string;
  /** Minimum count */
  min?: number;
  /** Maximum count */
  max?: number;
  /** File extensions filter */
  extensions?: string[];
}

/**
 * Command definition for dynamic registration
 * @interface CommandDefinition
 * @since 0.2.0
 */
export interface CommandDefinition {
  /** Unique command code */
  code: string;
  /** Display name */
  name: string;
  /** Command description */
  description?: string;
  /** Trigger keywords */
  keywords?: CommandKeyword[];
  /** Content match rules */
  matches?: CommandMatchDefinition[];
}

let isHandlerRegistered = false;

/**
 * For testing purposes - reset the registration state
 * @internal
 */
export function _resetRegistrationState() {
  isHandlerRegistered = false;
}

/**
 * Registers a command handler to respond to command calls from the main application.
 * Each plugin instance should call this function only once.
 *
 * The registered handler will receive all commands directed to this plugin and should
 * implement appropriate routing and response logic. Commands are executed asynchronously
 * and results are automatically sent back to the caller.
 *
 * @param handler - A function to handle received commands. It receives `command` and `args` as parameters.
 * @returns Promise that resolves when the handler is successfully registered.
 * @throws {Error} When handler registration fails or when called multiple times
 * @example
 * ```typescript
 * import { command } from 'onin-plugin-sdk';
 *
 * // Define command handler with routing
 * await command.handle(async (code, args) => {
 *   console.log(`Received command: ${code}`, args);
 *
 *   switch (code) {
 *     case 'greet':
 *       return `Hello, ${args.name || 'World'}!`;
 *
 *     case 'calculate':
 *       const { operation, a, b } = args;
 *       switch (operation) {
 *         case 'add': return a + b;
 *         case 'multiply': return a * b;
 *         default: throw new Error(`Unknown operation: ${operation}`);
 *       }
 *
 *     default:
 *       throw new Error(`Unknown command: ${code}`);
 *   }
 * });
 * ```
 * @since 0.1.0
 * @group API
 */
export async function handleCommand(
  handler: CommandHandler,
): Promise<void> {
  if (isHandlerRegistered) {
    console.warn(
      'CommandHandler has already been registered. Ignoring subsequent calls.',
    );
    return;
  }

  await dispatch({
    webview: () => {
      const eventCallback = async (event: any) => {
        const { command, args, requestId } = event.payload as {
          command: string;
          args: any;
          requestId: string;
        };

        try {
          const result = await handler(command, args);
          try {
            await invoke('plugin_command_result', {
              requestId,
              success: true,
              result,
            });
          } catch (invokeError) {
            console.error('Failed to send command result:', invokeError);
          }
        } catch (error) {
          try {
            await invoke('plugin_command_result', {
              requestId,
              success: false,
              error: error instanceof Error ? error.message : String(error),
            });
          } catch (invokeError) {
            console.error('Failed to send command error:', invokeError);
          }
        }
      };
      return listen('plugin_command_execute', eventCallback);
    },
    headless: () => listen('plugin_command_execute', handler as any),
  });

  isHandlerRegistered = true;
}

/**
 * Dynamically register a command for the current plugin.
 * 
 * This allows plugins to create commands at runtime based on user data,
 * external APIs, or other dynamic sources. Registered commands are persisted
 * and will appear in the command list.
 *
 * @param definition - The command definition including code, name, keywords, and matches
 * @returns Promise that resolves when the command is successfully registered
 * @throws {Error} When command registration fails
 * @example
 * ```typescript
 * import { command } from 'onin-plugin-sdk';
 *
 * // Register a dynamic command
 * await command.register({
 *   code: 'open-bookmark-1',
 *   name: 'My Favorite Site',
 *   keywords: [{ name: 'bookmark' }, { name: 'favorite' }],
 * });
 *
 * // Handle all commands
 * await command.handle((code, args) => {
 *   if (code.startsWith('open-bookmark-')) {
 *     // Open the bookmark
 *   }
 * });
 * ```
 * @since 0.2.0
 * @group API
 */
export async function registerCommand(
  definition: CommandDefinition,
): Promise<void> {
  if (!definition.code) {
    throw new Error('Command code is required');
  }
  if (!definition.name) {
    throw new Error('Command name is required');
  }

  await invoke('register_dynamic_command', {
    command: definition,
  });
}

/**
 * Remove a dynamically registered command.
 *
 * @param code - The unique command code to remove
 * @returns Promise that resolves when the command is successfully removed
 * @throws {Error} When command removal fails or command doesn't exist
 * @example
 * ```typescript
 * import { command } from 'onin-plugin-sdk';
 *
 * // Remove a previously registered command
 * await command.remove('open-bookmark-1');
 * ```
 * @since 0.2.0
 * @group API
 */
export async function removeCommand(code: string): Promise<void> {
  if (!code) {
    throw new Error('Command code is required');
  }

  await invoke('remove_dynamic_command', {
    commandCode: code,
  });
}

/**
 * Simplified method name alias for handleCommand
 * @see {@link handleCommand} - For detailed documentation
 * @since 0.2.0
 * @group API
 */
export const handle = handleCommand;
export const registerCommandHandler = handleCommand;

/**
 * Simplified method name alias for registerCommand
 * @see {@link registerCommand} - For detailed documentation
 * @since 0.2.0
 * @group API
 */
export const register = registerCommand;

/**
 * Simplified method name alias for removeCommand
 * @see {@link removeCommand} - For detailed documentation
 * @since 0.2.0
 * @group API
 */
export const remove = removeCommand;

/**
 * Command API namespace - provides command handling functionality for plugins
 *
 * Allows plugins to:
 * - **register**: Dynamically register commands at runtime
 * - **handle**: Register a handler to process command executions
 * - **remove**: Remove dynamically registered commands
 *
 * @namespace command
 * @version 0.2.0
 * @since 0.1.0
 * @group API
 * @example
 * ```typescript
 * import { command } from 'onin-plugin-sdk';
 *
 * // Register dynamic commands
 * await command.register({
 *   code: 'search-google',
 *   name: 'Search Google',
 *   keywords: [{ name: 'google' }],
 *   matches: [{ type: 'text', name: 'Search text', min: 1 }]
 * });
 *
 * // Handle command execution
 * await command.handle(async (code, args) => {
 *   if (code === 'search-google') {
 *     window.open(`https://google.com/search?q=${encodeURIComponent(args.text)}`);
 *   }
 * });
 *
 * // Remove a command when no longer needed
 * await command.remove('search-google');
 * ```
 */
export const command = {
  register: registerCommand,
  handle: handleCommand,
  remove: removeCommand,
};
