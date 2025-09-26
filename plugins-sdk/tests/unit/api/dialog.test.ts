import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

vi.mock('../../../src/utils/error-parser', () => ({
  parseDialogError: vi.fn()
}));

vi.mock('../../../src/types/errors', () => ({
  createPluginError: vi.fn(),
  errorUtils: {
    isPluginError: vi.fn()
  }
}));

// Import after mocking
import {
  showMessage,
  showConfirm,
  showOpen,
  showSave,
  info,
  warning,
  error,
  confirm,
  selectFile,
  selectFiles,
  selectFolder,
  saveFile,
  dialog,
  type MessageDialogOptions,
  type ConfirmDialogOptions,
  type OpenDialogOptions,
  type SaveDialogOptions,
  type DialogFilter
} from '../../../src/api/dialog';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';
import { parseDialogError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);
const mockParseDialogError = vi.mocked(parseDialogError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Dialog API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock dispatch to call the webview handler by default
    mockDispatch.mockImplementation(({ webview }) => webview());
    // Mock errorUtils.isPluginError to return false by default
    mockErrorUtils.isPluginError.mockReturnValue(false);
    // Mock createPluginError to return a proper error object
    mockCreatePluginError.mockImplementation((code, message) => {
      const error = new Error(message) as any;
      error.name = 'PluginError';
      error.code = code;
      return error;
    });
  });

  describe('showMessage', () => {
    it('should show message dialog with basic options', async () => {
      const options: MessageDialogOptions = {
        message: 'Test message',
        title: 'Test Title'
      };
      mockInvoke.mockResolvedValue(undefined);

      await showMessage(options);

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', options);
    });

    it('should show message dialog with all options', async () => {
      const options: MessageDialogOptions = {
        message: 'Test message',
        title: 'Test Title',
        kind: 'warning',
        okLabel: 'Got it'
      };
      mockInvoke.mockResolvedValue(undefined);

      await showMessage(options);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', options);
    });

    it('should handle errors', async () => {
      const options: MessageDialogOptions = { message: 'Test' };
      const error = new Error('Dialog failed');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Dialog failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseDialogError.mockReturnValue(parsedError);

      await expect(showMessage(options)).rejects.toThrow(parsedError);
      expect(mockParseDialogError).toHaveBeenCalledWith(error, {
        method: 'plugin_dialog_message',
        args: options
      });
    });
  });

  describe('showConfirm', () => {
    it('should show confirm dialog and return true', async () => {
      const options: ConfirmDialogOptions = {
        message: 'Are you sure?',
        title: 'Confirmation'
      };
      mockInvoke.mockResolvedValue(true);

      const result = await showConfirm(options);

      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', options);
    });

    it('should show confirm dialog and return false', async () => {
      const options: ConfirmDialogOptions = {
        message: 'Delete file?',
        kind: 'warning',
        okLabel: 'Delete',
        cancelLabel: 'Keep'
      };
      mockInvoke.mockResolvedValue(false);

      const result = await showConfirm(options);

      expect(result).toBe(false);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', options);
    });

    it('should handle errors', async () => {
      const options: ConfirmDialogOptions = { message: 'Test?' };
      const error = new Error('Confirm failed');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Confirm failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseDialogError.mockReturnValue(parsedError);

      await expect(showConfirm(options)).rejects.toThrow(parsedError);
    });
  });

  describe('showOpen', () => {
    it('should show open dialog with default options', async () => {
      mockInvoke.mockResolvedValue('/path/to/file.txt');

      const result = await showOpen();

      expect(result).toBe('/path/to/file.txt');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {});
    });

    it('should show open dialog with custom options', async () => {
      const options: OpenDialogOptions = {
        title: 'Select File',
        defaultPath: '/home/user',
        filters: [{ name: 'Text Files', extensions: ['txt', 'md'] }],
        multiple: false,
        directory: false
      };
      mockInvoke.mockResolvedValue('/path/to/selected.txt');

      const result = await showOpen(options);

      expect(result).toBe('/path/to/selected.txt');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', options);
    });

    it('should handle multiple file selection', async () => {
      const options: OpenDialogOptions = { multiple: true };
      const files = ['/path/to/file1.txt', '/path/to/file2.txt'];
      mockInvoke.mockResolvedValue(files);

      const result = await showOpen(options);

      expect(result).toEqual(files);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should return null when cancelled', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await showOpen();

      expect(result).toBeNull();
    });

    it('should return null for undefined result', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const result = await showOpen();

      expect(result).toBeNull();
    });

    it('should return null for invalid result types', async () => {
      mockInvoke.mockResolvedValue(123); // Invalid type

      const result = await showOpen();

      expect(result).toBeNull();
    });

    it('should handle errors', async () => {
      const error = new Error('Open dialog failed');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Open dialog failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseDialogError.mockReturnValue(parsedError);

      await expect(showOpen()).rejects.toThrow(parsedError);
    });
  });

  describe('showSave', () => {
    it('should show save dialog with default options', async () => {
      mockInvoke.mockResolvedValue('/path/to/save.txt');

      const result = await showSave();

      expect(result).toBe('/path/to/save.txt');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_save', {});
    });

    it('should show save dialog with custom options', async () => {
      const options: SaveDialogOptions = {
        title: 'Save As',
        defaultPath: '/home/user/document.txt',
        filters: [{ name: 'Text Files', extensions: ['txt'] }]
      };
      mockInvoke.mockResolvedValue('/path/to/saved.txt');

      const result = await showSave(options);

      expect(result).toBe('/path/to/saved.txt');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_save', options);
    });

    it('should return null when cancelled', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await showSave();

      expect(result).toBeNull();
    });

    it('should handle errors', async () => {
      const error = new Error('Save dialog failed');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Save dialog failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseDialogError.mockReturnValue(parsedError);

      await expect(showSave()).rejects.toThrow(parsedError);
    });
  });

  describe('convenience methods', () => {
    describe('info', () => {
      it('should show info message with title', async () => {
        mockInvoke.mockResolvedValue(undefined);

        await info('Information message', 'Info');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
          message: 'Information message',
          title: 'Info',
          kind: 'info'
        });
      });

      it('should show info message without title', async () => {
        mockInvoke.mockResolvedValue(undefined);

        await info('Information message');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
          message: 'Information message',
          title: undefined,
          kind: 'info'
        });
      });
    });

    describe('warning', () => {
      it('should show warning message', async () => {
        mockInvoke.mockResolvedValue(undefined);

        await warning('Warning message', 'Warning');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
          message: 'Warning message',
          title: 'Warning',
          kind: 'warning'
        });
      });
    });

    describe('error', () => {
      it('should show error message', async () => {
        mockInvoke.mockResolvedValue(undefined);

        await error('Error message', 'Error');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
          message: 'Error message',
          title: 'Error',
          kind: 'error'
        });
      });
    });

    describe('confirm', () => {
      it('should show simple confirm dialog', async () => {
        mockInvoke.mockResolvedValue(true);

        const result = await confirm('Are you sure?', 'Confirm');

        expect(result).toBe(true);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_confirm', {
          message: 'Are you sure?',
          title: 'Confirm'
        });
      });
    });

    describe('selectFile', () => {
      it('should select single file', async () => {
        const filters: DialogFilter[] = [{ name: 'Images', extensions: ['png', 'jpg'] }];
        mockInvoke.mockResolvedValue('/path/to/image.png');

        const result = await selectFile(filters, '/home/user');

        expect(result).toBe('/path/to/image.png');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
          filters,
          defaultPath: '/home/user',
          multiple: false,
          directory: false
        });
      });

      it('should handle no filters', async () => {
        mockInvoke.mockResolvedValue('/path/to/file.txt');

        await selectFile();

        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
          filters: undefined,
          defaultPath: undefined,
          multiple: false,
          directory: false
        });
      });
    });

    describe('selectFiles', () => {
      it('should select multiple files', async () => {
        const filters: DialogFilter[] = [{ name: 'Documents', extensions: ['pdf', 'doc'] }];
        const files = ['/path/to/doc1.pdf', '/path/to/doc2.pdf'];
        mockInvoke.mockResolvedValue(files);

        const result = await selectFiles(filters, '/documents');

        expect(result).toEqual(files);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
          filters,
          defaultPath: '/documents',
          multiple: true,
          directory: false
        });
      });
    });

    describe('selectFolder', () => {
      it('should select folder', async () => {
        mockInvoke.mockResolvedValue('/path/to/folder');

        const result = await selectFolder('/home');

        expect(result).toBe('/path/to/folder');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
          defaultPath: '/home',
          multiple: false,
          directory: true
        });
      });
    });

    describe('saveFile', () => {
      it('should save file with default name and filters', async () => {
        const filters: DialogFilter[] = [{ name: 'JSON', extensions: ['json'] }];
        mockInvoke.mockResolvedValue('/path/to/data.json');

        const result = await saveFile('data.json', filters);

        expect(result).toBe('/path/to/data.json');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_save', {
          defaultPath: 'data.json',
          filters
        });
      });
    });
  });

  describe('dialog namespace', () => {
    it('should have all expected methods', () => {
      expect(dialog.showMessage).toBe(showMessage);
      expect(dialog.showConfirm).toBe(showConfirm);
      expect(dialog.showOpen).toBe(showOpen);
      expect(dialog.showSave).toBe(showSave);
      expect(dialog.info).toBe(info);
      expect(dialog.warning).toBe(warning);
      expect(dialog.error).toBe(error);
      expect(dialog.confirm).toBe(confirm);
      expect(dialog.selectFile).toBe(selectFile);
      expect(dialog.selectFiles).toBe(selectFiles);
      expect(dialog.selectFolder).toBe(selectFolder);
      expect(dialog.saveFile).toBe(saveFile);
      expect(dialog.parseDialogError).toBe(parseDialogError);
    });

    it('should work through namespace methods', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await dialog.info('Namespace test');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_message', {
        message: 'Namespace test',
        title: undefined,
        kind: 'info'
      });
    });
  });

  describe('error handling', () => {
    it('should not re-parse PluginError instances', async () => {
      const pluginError = mockCreatePluginError('DIALOG_ACCESS_DENIED', 'Already parsed');
      mockInvoke.mockRejectedValue(pluginError);
      // Mock isPluginError to return true for this specific error
      mockErrorUtils.isPluginError.mockReturnValue(true);

      await expect(showMessage({ message: 'test' })).rejects.toThrow(pluginError);
      expect(mockParseDialogError).not.toHaveBeenCalled();
    });

    it('should pass context to error parser', async () => {
      const error = new Error('Test error');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Parsed error');
      const options = { message: 'test', title: 'Test' };
      
      mockInvoke.mockRejectedValue(error);
      mockParseDialogError.mockReturnValue(parsedError);

      await expect(showMessage(options)).rejects.toThrow(parsedError);
      expect(mockParseDialogError).toHaveBeenCalledWith(error, {
        method: 'plugin_dialog_message',
        args: options
      });
    });
  });

  describe('concurrent operations', () => {
    it('should handle multiple simultaneous dialogs', async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined) // info
        .mockResolvedValueOnce(true) // confirm
        .mockResolvedValueOnce('/path/file.txt'); // selectFile

      const [, confirmResult, fileResult] = await Promise.all([
        info('Information'),
        confirm('Are you sure?'),
        selectFile()
      ]);

      expect(confirmResult).toBe(true);
      expect(fileResult).toBe('/path/file.txt');
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const error = new Error('Dialog failed');
      const parsedError = mockCreatePluginError('DIALOG_ERROR', 'Dialog failed');
      
      mockInvoke
        .mockResolvedValueOnce(undefined) // success
        .mockRejectedValueOnce(error); // failure
      
      mockParseDialogError.mockReturnValue(parsedError);

      const results = await Promise.allSettled([
        info('Success'),
        warning('Failure')
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect(results[1].status).toBe('rejected');
      expect((results[1] as PromiseRejectedResult).reason).toBe(parsedError);
    });
  });

  describe('filter validation', () => {
    it('should handle complex filter configurations', async () => {
      const complexFilters: DialogFilter[] = [
        { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp'] },
        { name: 'Documents', extensions: ['pdf', 'doc', 'docx', 'txt', 'rtf'] },
        { name: 'All Files', extensions: ['*'] }
      ];
      
      mockInvoke.mockResolvedValue('/path/to/image.png');

      await selectFile(complexFilters);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
        filters: complexFilters,
        defaultPath: undefined,
        multiple: false,
        directory: false
      });
    });

    it('should handle empty filter extensions', async () => {
      const emptyFilters: DialogFilter[] = [
        { name: 'Empty', extensions: [] }
      ];
      
      mockInvoke.mockResolvedValue('/path/to/file');

      await selectFile(emptyFilters);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_dialog_open', {
        filters: emptyFilters,
        defaultPath: undefined,
        multiple: false,
        directory: false
      });
    });
  });
});