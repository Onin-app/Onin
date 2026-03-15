import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the modules using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn(),
}));

vi.mock('../../../src/core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn(),
  };
});

vi.mock('../../../src/utils/error-parser', () => ({
  parseDialogError: vi.fn(),
}));

vi.mock('../../../src/types/errors', () => ({
  createPluginError: vi.fn(),
  errorUtils: {
    isPluginError: vi.fn(),
  },
}));

// Import after mocking
import {
  showMessage,
  showConfirm,
  showOpen,
  selectFile,
  dialog,
  type DialogFilter,
} from '../../../src/api/dialog';
import { invoke } from '../../../src/core/ipc';
import {
  getEnvironment,
  RuntimeEnvironment,
} from '../../../src/core/environment';
import { parseDialogError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockGetEnvironment = vi.mocked(getEnvironment);
const mockParseDialogError = vi.mocked(parseDialogError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Dialog API Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockErrorUtils.isPluginError.mockReturnValue(false);
    mockCreatePluginError.mockImplementation((code, message) => {
      const error = new Error(message) as any;
      error.name = 'PluginError';
      error.code = code;
      return error;
    });
  });

  it('should work correctly in both webview and headless environments', async () => {
    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    mockInvoke.mockResolvedValue(undefined);

    await showMessage({ message: 'Test message' });
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
      message: 'Test message',
    });

    // Reset and test headless environment
    mockInvoke.mockClear();
    mockInvoke.mockResolvedValue(true);
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const result = await showConfirm({ message: 'Confirm?' });
    expect(result).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', {
      message: 'Confirm?',
    });
  });

  it('should properly handle error parsing in different environments', async () => {
    const originalError = new Error('Permission denied');
    const parsedError = mockCreatePluginError(
      'DIALOG_PERMISSION_DENIED',
      'Permission denied',
    );

    mockInvoke.mockRejectedValue(originalError);
    mockParseDialogError.mockReturnValue(parsedError);

    // Test in webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    await expect(showMessage({ message: 'test' })).rejects.toThrow(parsedError);
    expect(mockParseDialogError).toHaveBeenCalledWith(originalError, {
      method: 'plugin_dialog_message',
      args: { message: 'test' },
    });

    // Reset and test in headless environment
    mockParseDialogError.mockClear();
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    await expect(showConfirm({ message: 'test?' })).rejects.toThrow(
      parsedError,
    );
    expect(mockParseDialogError).toHaveBeenCalledWith(originalError, {
      method: 'plugin_dialog_confirm',
      args: { message: 'test?' },
    });
  });

  it('should handle complex dialog workflow', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Simulate a complete dialog workflow
    mockInvoke
      .mockResolvedValueOnce(undefined) // info message
      .mockResolvedValueOnce(true) // confirm dialog
      .mockResolvedValueOnce('/path/to/file.txt') // file selection
      .mockResolvedValueOnce('/path/to/save.txt'); // save dialog

    // Show info message
    await dialog.info('Starting process...');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
      message: 'Starting process...',
      title: undefined,
      kind: 'info',
    });

    // Confirm action
    const confirmed = await dialog.confirm('Continue with process?');
    expect(confirmed).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', {
      message: 'Continue with process?',
      title: undefined,
    });

    // Select input file
    const inputFile = await dialog.selectFile([
      { name: 'Text Files', extensions: ['txt'] },
    ]);
    expect(inputFile).toBe('/path/to/file.txt');

    // Select output location
    const outputFile = await dialog.saveFile('output.txt');
    expect(outputFile).toBe('/path/to/save.txt');

    expect(mockInvoke).toHaveBeenCalledTimes(4);
  });

  it('should handle file dialog return type variations', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Test single file selection (string return)
    mockInvoke.mockResolvedValueOnce('/single/file.txt');
    const singleFile = await showOpen({ multiple: false });
    expect(singleFile).toBe('/single/file.txt');

    // Test multiple file selection (array return)
    const multipleFiles = ['/file1.txt', '/file2.txt', '/file3.txt'];
    mockInvoke.mockResolvedValueOnce(multipleFiles);
    const selectedFiles = await showOpen({ multiple: true });
    expect(selectedFiles).toEqual(multipleFiles);
    expect(Array.isArray(selectedFiles)).toBe(true);

    // Test cancelled dialog (null return)
    mockInvoke.mockResolvedValueOnce(null);
    const cancelledResult = await showOpen();
    expect(cancelledResult).toBeNull();

    // Test undefined return (should convert to null)
    mockInvoke.mockResolvedValueOnce(undefined);
    const undefinedResult = await showOpen();
    expect(undefinedResult).toBeNull();

    // Test invalid return type (should convert to null)
    mockInvoke.mockResolvedValueOnce(42);
    const invalidResult = await showOpen();
    expect(invalidResult).toBeNull();
  });

  it('should handle different dialog types with proper options', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const filters: DialogFilter[] = [
      { name: 'Images', extensions: ['png', 'jpg', 'gif'] },
      { name: 'All Files', extensions: ['*'] },
    ];

    // Test file selection
    mockInvoke.mockResolvedValueOnce('/path/to/image.png');
    await selectFile(filters, '/home/user/pictures');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
      filters,
      defaultPath: '/home/user/pictures',
      multiple: false,
      directory: false,
    });

    // Test folder selection
    mockInvoke.mockResolvedValueOnce('/path/to/folder');
    await dialog.selectFolder('/home/user');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
      defaultPath: '/home/user',
      multiple: false,
      directory: true,
    });

    // Test multiple file selection
    const files = ['/file1.png', '/file2.jpg'];
    mockInvoke.mockResolvedValueOnce(files);
    await dialog.selectFiles(filters);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
      filters,
      defaultPath: undefined,
      multiple: true,
      directory: false,
    });
  });

  it('should handle message dialog variations', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    mockInvoke.mockResolvedValue(undefined);

    // Test different message types
    await dialog.info('Info message', 'Information');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
      message: 'Info message',
      title: 'Information',
      kind: 'info',
    });

    await dialog.warning('Warning message');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
      message: 'Warning message',
      title: undefined,
      kind: 'warning',
    });

    await dialog.error('Error message', 'Error');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
      message: 'Error message',
      title: 'Error',
      kind: 'error',
    });
  });

  it('should handle confirm dialog variations', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    // Test simple confirm
    mockInvoke.mockResolvedValueOnce(true);
    const result1 = await dialog.confirm('Simple confirm?');
    expect(result1).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', {
      message: 'Simple confirm?',
      title: undefined,
    });

    // Test detailed confirm
    mockInvoke.mockResolvedValueOnce(false);
    const result2 = await showConfirm({
      message: 'Delete all files?',
      title: 'Dangerous Action',
      kind: 'warning',
      okLabel: 'Delete All',
      cancelLabel: 'Keep Files',
    });
    expect(result2).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', {
      message: 'Delete all files?',
      title: 'Dangerous Action',
      kind: 'warning',
      okLabel: 'Delete All',
      cancelLabel: 'Keep Files',
    });
  });

  it('should handle concurrent dialog operations', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Test multiple dialogs running concurrently
    mockInvoke
      .mockResolvedValueOnce(undefined) // info
      .mockResolvedValueOnce(true) // confirm
      .mockResolvedValueOnce('/file.txt') // selectFile
      .mockResolvedValueOnce('/save.txt'); // saveFile

    const [, confirmResult, fileResult, saveResult] = await Promise.all([
      dialog.info('Processing...'),
      dialog.confirm('Continue?'),
      dialog.selectFile(),
      dialog.saveFile('output.txt'),
    ]);

    expect(confirmResult).toBe(true);
    expect(fileResult).toBe('/file.txt');
    expect(saveResult).toBe('/save.txt');
    expect(mockInvoke).toHaveBeenCalledTimes(4);
  });

  it('should handle error recovery scenarios', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    const error = new Error('Dialog system unavailable');
    const parsedError = mockCreatePluginError(
      'DIALOG_SYSTEM_ERROR',
      'Dialog system unavailable',
    );

    mockInvoke
      .mockRejectedValueOnce(error) // First call fails
      .mockResolvedValueOnce(true); // Second call succeeds

    mockParseDialogError.mockReturnValue(parsedError);

    // First dialog fails
    await expect(dialog.confirm('First attempt?')).rejects.toThrow(parsedError);

    // Second dialog succeeds (simulating retry or recovery)
    const result = await dialog.confirm('Second attempt?');
    expect(result).toBe(true);
  });
});
