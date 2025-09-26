import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

// Import after mocking
import {
  registerCommandHandler,
  register,
  command,
  _resetRegistrationState,
  type CommandHandler
} from '../../../src/api/command';
import { invoke, listen } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);
const mockDispatch = vi.mocked(dispatch);

describe('Command API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
    mockListen.mockResolvedValue(() => {});
    _resetRegistrationState();
  });

  describe('registerCommandHandler', () => {
    it('should register command handler successfully', async () => {
      const handler: CommandHandler = vi.fn();
      mockDispatch.mockImplementation(({ webview }) => webview());

      await registerCommandHandler(handler);

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
      expect(mockListen).toHaveBeenCalledWith('plugin_command_execute', expect.any(Function));
    });

    it('should handle webview environment correctly', async () => {
      const handler: CommandHandler = vi.fn().mockResolvedValue('test result');
      let capturedCallback: any;

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      // Simulate command execution
      const mockEvent = {
        payload: {
          command: 'test-command',
          args: { param: 'value' },
          requestId: 'req-123'
        }
      };

      await capturedCallback(mockEvent);

      expect(handler).toHaveBeenCalledWith('test-command', { param: 'value' });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-123',
        success: true,
        result: 'test result'
      });
    });

    it('should handle headless environment correctly', async () => {
      const handler: CommandHandler = vi.fn();
      mockDispatch.mockImplementation(({ headless }) => headless());

      await registerCommandHandler(handler);

      expect(mockListen).toHaveBeenCalledWith('plugin_command_execute', handler);
    });

    it('should handle command execution errors', async () => {
      const handler: CommandHandler = vi.fn().mockRejectedValue(new Error('Command failed'));
      let capturedCallback: any;

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      const mockEvent = {
        payload: {
          command: 'failing-command',
          args: {},
          requestId: 'req-456'
        }
      };

      await capturedCallback(mockEvent);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-456',
        success: false,
        error: 'Command failed'
      });
    });

    it('should handle non-Error exceptions', async () => {
      const handler: CommandHandler = vi.fn().mockRejectedValue('String error');
      let capturedCallback: any;

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      const mockEvent = {
        payload: {
          command: 'string-error-command',
          args: {},
          requestId: 'req-789'
        }
      };

      await capturedCallback(mockEvent);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-789',
        success: false,
        error: 'String error'
      });
    });

    it('should handle different command types', async () => {
      const handler: CommandHandler = vi.fn((command, args) => {
        switch (command) {
          case 'get-info':
            return { info: 'Plugin information' };
          case 'process-data':
            return { processed: args.data.toUpperCase() };
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

      // Test get-info command
      await capturedCallback({
        payload: { command: 'get-info', args: {}, requestId: 'req-1' }
      });

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-1',
        success: true,
        result: { info: 'Plugin information' }
      });

      // Test process-data command
      await capturedCallback({
        payload: { command: 'process-data', args: { data: 'hello' }, requestId: 'req-2' }
      });

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-2',
        success: true,
        result: { processed: 'HELLO' }
      });
    });

    it('should handle async command handlers', async () => {
      const handler: CommandHandler = vi.fn().mockImplementation(async (command, args) => {
        await new Promise(resolve => setTimeout(resolve, 10));
        return { command, processed: true, args };
      });

      let capturedCallback: any;
      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      await capturedCallback({
        payload: {
          command: 'async-command',
          args: { data: 'test' },
          requestId: 'req-async'
        }
      });

      expect(handler).toHaveBeenCalledWith('async-command', { data: 'test' });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-async',
        success: true,
        result: {
          command: 'async-command',
          processed: true,
          args: { data: 'test' }
        }
      });
    });

    it('should handle command handler that returns undefined', async () => {
      const handler: CommandHandler = vi.fn().mockReturnValue(undefined);
      let capturedCallback: any;

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      await capturedCallback({
        payload: {
          command: 'void-command',
          args: {},
          requestId: 'req-void'
        }
      });

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-void',
        success: true,
        result: undefined
      });
    });

    it('should prevent multiple handler registrations', async () => {
      const handler1: CommandHandler = vi.fn();
      const handler2: CommandHandler = vi.fn();
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      mockDispatch.mockImplementation(({ webview }) => webview());

      // First registration should succeed
      await registerCommandHandler(handler1);
      expect(mockDispatch).toHaveBeenCalledTimes(1);

      // Second registration should be ignored
      await registerCommandHandler(handler2);
      expect(mockDispatch).toHaveBeenCalledTimes(1); // Still only called once
      expect(consoleSpy).toHaveBeenCalledWith(
        "CommandHandler has already been registered. Ignoring subsequent calls."
      );

      consoleSpy.mockRestore();
    });
  });

  describe('register alias', () => {
    it('should be the same function as registerCommandHandler', () => {
      expect(register).toBe(registerCommandHandler);
    });
  });

  describe('command namespace', () => {
    it('should have register method', () => {
      expect(command.register).toBe(registerCommandHandler);
    });
  });

  describe('error scenarios', () => {
    it('should handle listen function errors', async () => {
      const handler: CommandHandler = vi.fn();
      const listenError = new Error('Listen failed');
      
      mockListen.mockRejectedValue(listenError);
      mockDispatch.mockImplementation(({ webview }) => webview());

      await expect(registerCommandHandler(handler)).rejects.toThrow('Listen failed');
    });

    it('should handle invoke errors during result reporting', async () => {
      const handler: CommandHandler = vi.fn().mockResolvedValue('result');
      const invokeError = new Error('Invoke failed');
      let capturedCallback: any;
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });
      mockInvoke.mockRejectedValue(invokeError);

      await registerCommandHandler(handler);

      // This should not throw, but the invoke error should be handled internally
      await capturedCallback({
        payload: {
          command: 'test-command',
          args: {},
          requestId: 'req-invoke-error'
        }
      });

      expect(handler).toHaveBeenCalled();
      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-invoke-error',
        success: true,
        result: 'result'
      });
      expect(consoleSpy).toHaveBeenCalledWith('Failed to send command result:', invokeError);
      
      consoleSpy.mockRestore();
    });
  });

  describe('concurrent command handling', () => {
    it('should handle multiple simultaneous commands', async () => {
      const handler: CommandHandler = vi.fn().mockImplementation(async (command, args) => {
        const delay = command === 'slow-command' ? 50 : 10;
        await new Promise(resolve => setTimeout(resolve, delay));
        return { command, processed: true };
      });

      let capturedCallback: any;
      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      const commands = [
        { command: 'fast-command', args: {}, requestId: 'req-1' },
        { command: 'slow-command', args: {}, requestId: 'req-2' },
        { command: 'fast-command', args: {}, requestId: 'req-3' }
      ];

      await Promise.all(
        commands.map(payload => capturedCallback({ payload }))
      );

      expect(handler).toHaveBeenCalledTimes(3);
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const handler: CommandHandler = vi.fn().mockImplementation((command, args) => {
        if (command === 'failing-command') {
          throw new Error('Command failed');
        }
        return { command, success: true };
      });

      let capturedCallback: any;
      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      const commands = [
        { command: 'success-command', args: {}, requestId: 'req-success' },
        { command: 'failing-command', args: {}, requestId: 'req-failure' }
      ];

      await Promise.all(
        commands.map(payload => capturedCallback({ payload }))
      );

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-success',
        success: true,
        result: { command: 'success-command', success: true }
      });

      expect(mockInvoke).toHaveBeenCalledWith('plugin_command_result', {
        requestId: 'req-failure',
        success: false,
        error: 'Command failed'
      });
    });
  });

  describe('command handler patterns', () => {
    it('should support command routing pattern', async () => {
      const commands = {
        'user.create': vi.fn().mockResolvedValue({ id: 1, created: true }),
        'user.update': vi.fn().mockResolvedValue({ id: 1, updated: true }),
        'user.delete': vi.fn().mockResolvedValue({ id: 1, deleted: true }),
        'data.process': vi.fn().mockResolvedValue({ processed: true })
      };

      const handler: CommandHandler = vi.fn().mockImplementation((command, args) => {
        const commandHandler = commands[command as keyof typeof commands];
        if (!commandHandler) {
          throw new Error(`Unknown command: ${command}`);
        }
        return commandHandler(args);
      });

      let capturedCallback: any;
      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => {});
      });

      await registerCommandHandler(handler);

      // Test each command
      await capturedCallback({
        payload: { command: 'user.create', args: { name: 'John' }, requestId: 'req-1' }
      });
      await capturedCallback({
        payload: { command: 'user.update', args: { id: 1, name: 'Jane' }, requestId: 'req-2' }
      });
      await capturedCallback({
        payload: { command: 'data.process', args: { data: 'test' }, requestId: 'req-3' }
      });

      expect(commands['user.create']).toHaveBeenCalledWith({ name: 'John' });
      expect(commands['user.update']).toHaveBeenCalledWith({ id: 1, name: 'Jane' });
      expect(commands['data.process']).toHaveBeenCalledWith({ data: 'test' });
    });
  });
});