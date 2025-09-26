import { describe, it, expect, vi } from 'vitest';

// Mock the modules using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn()
  };
});

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
import { readText, writeText, clipboard } from '../../../src/api/clipboard';
import { invoke } from '../../../src/core/ipc';
import { getEnvironment, RuntimeEnvironment } from '../../../src/core/environment';
import { parseClipboardError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockGetEnvironment = vi.mocked(getEnvironment);
const mockParseClipboardError = vi.mocked(parseClipboardError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Clipboard API Integration', () => {
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
    mockInvoke.mockResolvedValue('test text');

    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    
    let result = await readText();
    expect(result).toBe('test text');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_text', undefined);

    // Reset and test headless environment
    mockInvoke.mockClear();
    mockInvoke.mockResolvedValue('headless text');
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    result = await readText();
    expect(result).toBe('headless text');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_text', undefined);
  });

  it('should properly handle error parsing in different environments', async () => {
    const originalError = new Error('Permission denied');
    const parsedError = createPluginError('CLIPBOARD_PERMISSION_DENIED', 'Permission denied');
    
    mockInvoke.mockRejectedValue(originalError);
    mockParseClipboardError.mockReturnValue(parsedError);

    // Test in webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    
    await expect(writeText('test')).rejects.toThrow(parsedError);
    expect(mockParseClipboardError).toHaveBeenCalledWith(originalError, {
      method: 'plugin_clipboard_write_text',
      args: { text: 'test' }
    });

    // Reset and test in headless environment
    mockParseClipboardError.mockClear();
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);
    
    await expect(writeText('test2')).rejects.toThrow(parsedError);
    expect(mockParseClipboardError).toHaveBeenCalledWith(originalError, {
      method: 'plugin_clipboard_write_text',
      args: { text: 'test2' }
    });
  });

  it('should handle complex clipboard operations workflow', async () => {
    // Simulate a complete clipboard workflow
    mockInvoke
      .mockResolvedValueOnce(undefined) // clear
      .mockResolvedValueOnce(undefined) // writeText
      .mockResolvedValueOnce('copied text') // readText
      .mockResolvedValueOnce('copied text'); // hasText internal readText call

    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Clear clipboard
    await clipboard.clear();
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_clear', undefined);

    // Write text
    await clipboard.writeText('copied text');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_text', { text: 'copied text' });

    // Read text back
    const text = await clipboard.readText();
    expect(text).toBe('copied text');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_read_text', undefined);

    // Check if has text
    const hasText = await clipboard.hasText();
    expect(hasText).toBe(true);
  });

  it('should handle image operations in different environments', async () => {
    const imageData = 'data:image/png;base64,test';
    const uint8Array = new Uint8Array([1, 2, 3, 4]);

    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    mockInvoke
      .mockResolvedValueOnce(undefined) // writeImage
      .mockResolvedValueOnce(imageData); // readImage

    await clipboard.writeImage(uint8Array);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_image', { 
      imageData: [1, 2, 3, 4] 
    });

    const result = await clipboard.readImage();
    expect(result).toBe(imageData);

    // Test headless environment
    mockInvoke.mockClear();
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);
    mockInvoke.mockResolvedValueOnce(undefined).mockResolvedValueOnce(null);

    await clipboard.writeImage('base64string');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_clipboard_write_image', { 
      imageData: 'base64string' 
    });

    const hasImage = await clipboard.hasImage();
    expect(hasImage).toBe(false);
  });
});