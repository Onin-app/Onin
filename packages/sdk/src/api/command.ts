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
 * Command handler function type
 * @typeParam T - The return type of the command handler
 * @since 0.1.0
 * @group Types
 */

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
 * async function commandHandler(command: string, args: any) {
 *   console.log(`Received command: ${command}`, args);
 *
 *   switch (command) {
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
 *     case 'async-task':
 *       // Simulate async work
 *       await new Promise(resolve => setTimeout(resolve, 1000));
 *       return { status: 'completed', data: args.input };
 *
 *     default:
 *       throw new Error(`Unknown command: ${command}`);
 *   }
 * }
 *
 * // Register the handler
 * await command.register(commandHandler);
 * console.log('Command handler registered successfully');
 * ```
 * @since 0.1.0
 * @group API
 */
export async function registerCommandHandler(
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
 * Simplified method name alias for registerCommandHandler
 * @see {@link registerCommandHandler} - For detailed documentation
 * @since 0.1.0
 * @group API
 */
export const register = registerCommandHandler;

/**
 * Command API namespace - provides command handling functionality for plugins
 *
 * Allows plugins to register handlers for commands sent from the main application
 * or other plugins. Commands provide a way for external code to invoke plugin
 * functionality with arguments and receive results.
 *
 * **Single Handler**: Each plugin can only register one command handler. The handler
 * should implement internal routing to handle different command types.
 *
 * **Async Support**: Command handlers can be synchronous or asynchronous. Results
 * are automatically serialized and sent back to the caller.
 *
 * **Error Handling**: Errors thrown by command handlers are automatically caught
 * and sent back to the caller as error responses.
 *
 * @namespace command
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @example
 * ```typescript
 * import { command } from 'onin-plugin-sdk';
 *
 * // Simple command handler
 * await command.register(async (cmd, args) => {
 *   if (cmd === 'ping') {
 *     return 'pong';
 *   }
 *   if (cmd === 'echo') {
 *     return args.message;
 *   }
 *   throw new Error(`Unknown command: ${cmd}`);
 * });
 *
 * // Complex command handler with validation
 * await command.register(async (cmd, args) => {
 *   // Input validation
 *   if (!cmd || typeof cmd !== 'string') {
 *     throw new Error('Invalid command format');
 *   }
 *
 *   // Command routing
 *   switch (cmd) {
 *     case 'user.create':
 *       if (!args.name || !args.email) {
 *         throw new Error('Missing required fields: name, email');
 *       }
 *       return await createUser(args.name, args.email);
 *
 *     case 'user.list':
 *       return await listUsers(args.limit || 10);
 *
 *     default:
 *       throw new Error(`Command not supported: ${cmd}`);
 *   }
 * });
 * ```
 */
export const command = {
  register: registerCommandHandler,
};
