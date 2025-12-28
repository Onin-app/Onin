import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

vi.mock('../../../src/utils/error-parser', () => ({
  parseClipboardError: vi.fn()
}));

vi.mock('../../../src/types/errors', () => ({
  createPluginError: vi.fn(),
  errorUtils: {
    isPluginError: vi.fn()
  }
}));

// Import after mocking
import {
  readText,
  writeText,
  readImage,
  writeImage,
  clear,
  hasText,
  hasImage,
  copy,
  paste,
  clipboard
} from '../../../src/api/clipboard';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';
import { parseClipboardError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);
const mockParseClipboardError = vi.mocked(parseClipboardError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Clipboard API', () => {
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

  describe('readText', () => {
    it('should read text from clipboard successfully', async () => {
      const expectedText = 'Hello, World!';
      mockInvoke.mockResolvedValue(expectedText);

      const result = await readText();

      expect(result).toBe(expectedText);
      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_text', undefined);
    });

    it('should return null when clipboard is empty', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await readText();

      expect(result).toBeNull();
    });

    it('should handle errors and parse them', async () => {
      const originalError = new Error('Clipboard access denied');
      const parsedError = createPluginError('CLIPBOARD_ACCESS_DENIED', 'Access denied');
      
      mockInvoke.mockRejectedValue(originalError);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(readText()).rejects.toThrow(parsedError);
      expect(mockParseClipboardError).toHaveBeenCalledWith(originalError, {
        method: 'plugin_clipboard_read_text',
        args: undefined
      });
    });
  });

  describe('writeText', () => {
    it('should write text to clipboard successfully', async () => {
      const text = 'Test text';
      mockInvoke.mockResolvedValue(undefined);

      await writeText(text);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text });
    });

    it('should handle empty string', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await writeText('');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text: '' });
    });

    it('should handle special characters', async () => {
      const specialText = 'Text with émojis 🎉 and\nnewlines\ttabs';
      mockInvoke.mockResolvedValue(undefined);

      await writeText(specialText);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text: specialText });
    });

    it('should handle errors', async () => {
      const error = new Error('Write failed');
      const parsedError = createPluginError('CLIPBOARD_WRITE_FAILED', 'Write failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(writeText('test')).rejects.toThrow(parsedError);
    });
  });

  describe('readImage', () => {
    it('should read image from clipboard successfully', async () => {
      const base64Image = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==';
      mockInvoke.mockResolvedValue(base64Image);

      const result = await readImage();

      expect(result).toBe(base64Image);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_image', undefined);
    });

    it('should return null when no image in clipboard', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await readImage();

      expect(result).toBeNull();
    });

    it('should handle errors', async () => {
      const error = new Error('Image read failed');
      const parsedError = createPluginError('CLIPBOARD_READ_FAILED', 'Read failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(readImage()).rejects.toThrow(parsedError);
    });
  });

  describe('writeImage', () => {
    it('should write base64 image to clipboard', async () => {
      const base64Image = 'data:image/png;base64,test';
      mockInvoke.mockResolvedValue(undefined);

      await writeImage(base64Image);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_image', { 
        imageData: base64Image 
      });
    });

    it('should write Uint8Array image to clipboard', async () => {
      const imageArray = new Uint8Array([1, 2, 3, 4]);
      mockInvoke.mockResolvedValue(undefined);

      await writeImage(imageArray);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_image', { 
        imageData: [1, 2, 3, 4] 
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Image write failed');
      const parsedError = createPluginError('CLIPBOARD_WRITE_FAILED', 'Write failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(writeImage('test')).rejects.toThrow(parsedError);
    });
  });

  describe('clear', () => {
    it('should clear clipboard successfully', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await clear();

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_clear', undefined);
    });

    it('should handle errors', async () => {
      const error = new Error('Clear failed');
      const parsedError = createPluginError('CLIPBOARD_CLEAR_FAILED', 'Clear failed');
      
      mockInvoke.mockRejectedValue(error);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(clear()).rejects.toThrow(parsedError);
    });
  });

  describe('hasText', () => {
    it('should return true when clipboard has text', async () => {
      mockInvoke.mockResolvedValue('Some text');

      const result = await hasText();

      expect(result).toBe(true);
    });

    it('should return false when clipboard is empty', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await hasText();

      expect(result).toBe(false);
    });

    it('should return false when clipboard has empty string', async () => {
      mockInvoke.mockResolvedValue('');

      const result = await hasText();

      expect(result).toBe(false);
    });

    it('should return true for whitespace-only text', async () => {
      mockInvoke.mockResolvedValue('   ');

      const result = await hasText();

      expect(result).toBe(true);
    });
  });

  describe('hasImage', () => {
    it('should return true when clipboard has image', async () => {
      mockInvoke.mockResolvedValue('data:image/png;base64,test');

      const result = await hasImage();

      expect(result).toBe(true);
    });

    it('should return false when clipboard has no image', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await hasImage();

      expect(result).toBe(false);
    });
  });

  describe('convenience methods', () => {
    describe('copy', () => {
      it('should be an alias for writeText', async () => {
        const text = 'Copy this';
        mockInvoke.mockResolvedValue(undefined);

        await copy(text);

        expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text });
      });
    });

    describe('paste', () => {
      it('should be an alias for readText', async () => {
        const text = 'Pasted text';
        mockInvoke.mockResolvedValue(text);

        const result = await paste();

        expect(result).toBe(text);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_text', undefined);
      });
    });
  });

  describe('clipboard namespace', () => {
    it('should have all expected methods', () => {
      expect(clipboard.readText).toBe(readText);
      expect(clipboard.writeText).toBe(writeText);
      expect(clipboard.readImage).toBe(readImage);
      expect(clipboard.writeImage).toBe(writeImage);
      expect(clipboard.clear).toBe(clear);
      expect(clipboard.hasText).toBe(hasText);
      expect(clipboard.hasImage).toBe(hasImage);
      expect(clipboard.copy).toBe(copy);
      expect(clipboard.paste).toBe(paste);
      expect(clipboard.parseClipboardError).toBe(parseClipboardError);
    });

    it('should work through namespace methods', async () => {
      const text = 'Namespace test';
      mockInvoke.mockResolvedValue(undefined);

      await clipboard.writeText(text);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text });
    });
  });

  describe('error handling', () => {
    it('should not re-parse PluginError instances', async () => {
      const pluginError = mockCreatePluginError('CLIPBOARD_ACCESS_DENIED', 'Already parsed');
      mockInvoke.mockRejectedValue(pluginError);
      // Mock isPluginError to return true for this specific error
      mockErrorUtils.isPluginError.mockReturnValue(true);

      await expect(readText()).rejects.toThrow(pluginError);
      expect(mockParseClipboardError).not.toHaveBeenCalled();
    });

    it('should pass context to error parser', async () => {
      const error = new Error('Test error');
      const parsedError = createPluginError('CLIPBOARD_ERROR', 'Parsed error');
      
      mockInvoke.mockRejectedValue(error);
      mockParseClipboardError.mockReturnValue(parsedError);

      await expect(writeText('test')).rejects.toThrow(parsedError);
      expect(mockParseClipboardError).toHaveBeenCalledWith(error, {
        method: 'plugin_clipboard_write_text',
        args: { text: 'test' }
      });
    });
  });

  describe('concurrent operations', () => {
    it('should handle multiple simultaneous reads', async () => {
      mockInvoke.mockResolvedValue('concurrent text');

      const promises = [readText(), readText(), readText()];
      const results = await Promise.all(promises);

      expect(results).toEqual(['concurrent text', 'concurrent text', 'concurrent text']);
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed operations', async () => {
      mockInvoke
        .mockResolvedValueOnce('read text')
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce('image data');

      const [readResult, , imageResult] = await Promise.all([
        readText(),
        writeText('write text'),
        readImage()
      ]);

      expect(readResult).toBe('read text');
      expect(imageResult).toBe('image data');
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });
});