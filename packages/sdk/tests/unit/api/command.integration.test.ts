import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the modules using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn(),
}));

vi.mock('../../../src/core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn(),
  };
});

// Import after mocking
import {
  registerCommandHandler,
  command,
  _resetRegistrationState,
  type CommandHandler,
} from '../../../src/api/command';
import { invoke, listen } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';
import {
  getEnvironment,
  RuntimeEnvironment,
} from '../../../src/core/environment';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);
const mockDispatch = vi.mocked(dispatch);
const mockGetEnvironment = vi.mocked(getEnvironment);

describe('Command API Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(() => {});
    _resetRegistrationState();
  });

  it('should work correctly in both webview and headless environments', async () => {
    const handler: CommandHandler = vi.fn().mockResolvedValue('test result');

    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    mockDispatch.mockImplementation(({ webview }) => webview());

    await registerCommandHandler(handler);
    expect(mockListen).toHaveBeenCalledWith(
      'plugin_command_execute',
      expect.any(Function),
    );

    // Reset for headless test
    vi.clearAllMocks();
    _resetRegistrationState(); // Reset the registration state
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);
    mockDispatch.mockImplementation(({ headless }) => headless());
    mockListen.mockResolvedValue(() => {});

    const headlessHandler: CommandHandler = vi.fn();
    await registerCommandHandler(headlessHandler);
    expect(mockListen).toHaveBeenCalledWith(
      'plugin_command_execute',
      headlessHandler,
    );
  });

  it('should handle complete plugin command workflow', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Simulate a complete plugin with state management
    const pluginState = {
      users: [] as Array<{ id: number; name: string; email: string }>,
      settings: { theme: 'dark', version: '1.0.0' },
    };

    const handler: CommandHandler = vi
      .fn()
      .mockImplementation(async (command, args) => {
        switch (command) {
          case 'plugin.getInfo':
            return {
              name: 'Test Plugin',
              version: pluginState.settings.version,
              userCount: pluginState.users.length,
            };

          case 'user.create':
            const newUser = {
              id: pluginState.users.length + 1,
              name: args.name,
              email: args.email,
            };
            pluginState.users.push(newUser);
            return { success: true, user: newUser };

          case 'user.list':
            return {
              users: pluginState.users,
              total: pluginState.users.length,
            };

          case 'settings.update':
            pluginState.settings = {
              ...pluginState.settings,
              ...args.settings,
            };
            return { success: true, settings: pluginState.settings };

          default:
            throw new Error(`Unknown command: ${command}`);
        }
      });

    let capturedCallback: any;
    mockDispatch.mockImplementation(({ webview }) => webview());
    mockListen.mockImplementation((event, callback) => {
      capturedCallback = callback;
      return Promise.resolve(() => {});
    });

    await registerCommandHandler(handler);

    // Test plugin info
    await capturedCallback({
      payload: { command: 'plugin.getInfo', args: {}, requestId: 'req-1' },
    });

    expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
      requestId: 'req-1',
      success: true,
      result: expect.objectContaining({
        name: 'Test Plugin',
        version: '1.0.0',
        userCount: 0,
      }),
    });

    // Create user
    await capturedCallback({
      payload: {
        command: 'user.create',
        args: { name: 'John Doe', email: 'john@example.com' },
        requestId: 'req-2',
      },
    });

    expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
      requestId: 'req-2',
      success: true,
      result: {
        success: true,
        user: { id: 1, name: 'John Doe', email: 'john@example.com' },
      },
    });

    // List users
    await capturedCallback({
      payload: { command: 'user.list', args: {}, requestId: 'req-3' },
    });

    expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
      requestId: 'req-3',
      success: true,
      result: {
        users: [{ id: 1, name: 'John Doe', email: 'john@example.com' }],
        total: 1,
      },
    });

    expect(mockInvoke).toHaveBeenCalledTimes(3);
  });

  it('should handle error recovery scenarios', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);
    mockDispatch.mockImplementation(({ headless }) => headless());

    let failureCount = 0;
    const handler: CommandHandler = vi
      .fn()
      .mockImplementation((command, args) => {
        if (command === 'unstable-command') {
          failureCount++;
          if (failureCount <= 2) {
            throw new Error(`Temporary failure ${failureCount}`);
          }
          return {
            success: true,
            message: 'Operation completed after retries',
          };
        }
        return { command, success: true };
      });

    await registerCommandHandler(handler);

    // Test that the handler was registered (in headless mode, it's passed directly)
    expect(mockListen).toHaveBeenCalledWith('plugin_command_execute', handler);

    // Simulate multiple attempts
    try {
      await handler('unstable-command', {});
      expect(true).toBe(false); // Should not reach here
    } catch (error) {
      expect((error as Error).message).toBe('Temporary failure 1');
    }

    try {
      await handler('unstable-command', {});
      expect(true).toBe(false); // Should not reach here
    } catch (error) {
      expect((error as Error).message).toBe('Temporary failure 2');
    }

    // Third attempt should succeed
    const result = await handler('unstable-command', {});
    expect(result.success).toBe(true);
    expect(result.message).toBe('Operation completed after retries');
  });

  it('should handle complex command arguments and responses', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    const handler: CommandHandler = vi
      .fn()
      .mockImplementation((command, args) => {
        return {
          command,
          receivedArgs: args,
          timestamp: Date.now(),
          metadata: {
            processed: true,
            environment: 'webview',
          },
        };
      });

    let capturedCallback: any;
    mockDispatch.mockImplementation(({ webview }) => webview());
    mockListen.mockImplementation((event, callback) => {
      capturedCallback = callback;
      return Promise.resolve(() => {});
    });

    await registerCommandHandler(handler);

    const complexArgs = {
      stringParam: 'test string',
      numberParam: 42,
      booleanParam: true,
      arrayParam: [1, 2, 3, 'four'],
      objectParam: {
        nested: {
          value: 'deep nested',
          array: [{ id: 1 }, { id: 2 }],
        },
      },
    };

    await capturedCallback({
      payload: {
        command: 'complex-command',
        args: complexArgs,
        requestId: 'req-complex',
      },
    });

    expect(handler).toHaveBeenCalledWith('complex-command', complexArgs);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
      requestId: 'req-complex',
      success: true,
      result: expect.objectContaining({
        command: 'complex-command',
        receivedArgs: complexArgs,
        metadata: expect.objectContaining({
          processed: true,
          environment: 'webview',
        }),
      }),
    });
  });
});
