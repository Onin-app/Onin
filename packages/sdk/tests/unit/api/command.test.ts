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
  handleCommand,
  registerCommand,
  removeCommand,
  handle,
  register,
  remove,
  command,
  _resetRegistrationState,
  type CommandHandler,
  type CommandDefinition
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
    mockListen.mockResolvedValue(() => { });
    _resetRegistrationState();
  });

  describe('handleCommand', () => {
    it('should register command handler successfully', async () => {
      const handler: CommandHandler = vi.fn();
      mockDispatch.mockImplementation(({ webview }) => webview());

      await handleCommand(handler);

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
        return Promise.resolve(() => { });
      });

      await handleCommand(handler);

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

      await handleCommand(handler);

      expect(mockListen).toHaveBeenCalledWith('plugin_command_execute', handler);
    });

    it('should handle command execution errors', async () => {
      const handler: CommandHandler = vi.fn().mockRejectedValue(new Error('Command failed'));
      let capturedCallback: any;

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => { });
      });

      await handleCommand(handler);

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
        return Promise.resolve(() => { });
      });

      await handleCommand(handler);

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

    it('should handle async command handlers', async () => {
      const handler: CommandHandler = vi.fn().mockImplementation(async (command, args) => {
        await new Promise(resolve => setTimeout(resolve, 10));
        return { command, processed: true, args };
      });

      let capturedCallback: any;
      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => { });
      });

      await handleCommand(handler);

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

    it('should prevent multiple handler registrations', async () => {
      const handler1: CommandHandler = vi.fn();
      const handler2: CommandHandler = vi.fn();
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });

      mockDispatch.mockImplementation(({ webview }) => webview());

      // First registration should succeed
      await handleCommand(handler1);
      expect(mockDispatch).toHaveBeenCalledTimes(1);

      // Second registration should be ignored
      await handleCommand(handler2);
      expect(mockDispatch).toHaveBeenCalledTimes(1); // Still only called once
      expect(consoleSpy).toHaveBeenCalledWith(
        "CommandHandler has already been registered. Ignoring subsequent calls."
      );

      consoleSpy.mockRestore();
    });
  });

  describe('registerCommand', () => {
    it('should register a dynamic command', async () => {
      const definition: CommandDefinition = {
        code: 'test-command',
        name: 'Test Command',
        description: 'A test command',
        keywords: [{ name: 'test' }]
      };

      await registerCommand(definition);

      expect(mockInvoke).toHaveBeenCalledWith('register_dynamic_command', {
        command: definition
      });
    });

    it('should register a command with matches', async () => {
      const definition: CommandDefinition = {
        code: 'url-handler',
        name: 'URL Handler',
        keywords: [{ name: 'url' }],
        matches: [
          { type: 'text', name: 'URL', regexp: '^https?://' }
        ]
      };

      await registerCommand(definition);

      expect(mockInvoke).toHaveBeenCalledWith('register_dynamic_command', {
        command: definition
      });
    });

    it('should throw error if code is missing', async () => {
      const definition = {
        name: 'No Code',
      } as CommandDefinition;

      await expect(registerCommand(definition)).rejects.toThrow('Command code is required');
    });

    it('should throw error if name is missing', async () => {
      const definition = {
        code: 'no-name',
      } as CommandDefinition;

      await expect(registerCommand(definition)).rejects.toThrow('Command name is required');
    });
  });

  describe('removeCommand', () => {
    it('should remove a dynamic command', async () => {
      await removeCommand('test-command');

      expect(mockInvoke).toHaveBeenCalledWith('remove_dynamic_command', {
        commandCode: 'test-command'
      });
    });

    it('should throw error if code is missing', async () => {
      await expect(removeCommand('')).rejects.toThrow('Command code is required');
    });
  });

  describe('aliases', () => {
    it('handle should be the same function as handleCommand', () => {
      expect(handle).toBe(handleCommand);
    });

    it('register should be the same function as registerCommand', () => {
      expect(register).toBe(registerCommand);
    });

    it('remove should be the same function as removeCommand', () => {
      expect(remove).toBe(removeCommand);
    });
  });

  describe('command namespace', () => {
    it('should have handle method', () => {
      expect(command.handle).toBe(handleCommand);
    });

    it('should have register method', () => {
      expect(command.register).toBe(registerCommand);
    });

    it('should have remove method', () => {
      expect(command.remove).toBe(removeCommand);
    });
  });

  describe('error scenarios', () => {
    it('should handle listen function errors', async () => {
      const handler: CommandHandler = vi.fn();
      const listenError = new Error('Listen failed');

      mockListen.mockRejectedValue(listenError);
      mockDispatch.mockImplementation(({ webview }) => webview());

      await expect(handleCommand(handler)).rejects.toThrow('Listen failed');
    });

    it('should handle invoke errors during result reporting', async () => {
      const handler: CommandHandler = vi.fn().mockResolvedValue('result');
      const invokeError = new Error('Invoke failed');
      let capturedCallback: any;
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      mockDispatch.mockImplementation(({ webview }) => webview());
      mockListen.mockImplementation((event, callback) => {
        capturedCallback = callback;
        return Promise.resolve(() => { });
      });
      mockInvoke.mockRejectedValue(invokeError);

      await handleCommand(handler);

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
        return Promise.resolve(() => { });
      });

      await handleCommand(handler);

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
  });
});