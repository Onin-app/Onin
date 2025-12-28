import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

vi.mock('../../../src/utils/error-parser', () => ({
  parseFsError: vi.fn()
}));

vi.mock('../../../src/types/errors', () => ({
  createPluginError: vi.fn(),
  errorUtils: {
    isPluginError: vi.fn()
  }
}));

// Import after mocking
import {
  readFile,
  writeFile,
  exists,
  createDir,
  listDir,
  deleteFile,
  deleteDir,
  getFileInfo,
  copyFile,
  moveFile,
  fs,
  type FileInfo
} from '../../../src/api/fs';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';
import { parseFsError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);
const mockParseFsError = vi.mocked(parseFsError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('FileSystem API', () => {
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

  describe('readFile', () => {
    it('should read file content successfully', async () => {
      const content = 'Hello, World!';
      mockInvoke.mockResolvedValue(content);

      const result = await readFile('test.txt');

      expect(result).toBe(content);
      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_read_file', { path: 'test.txt' });
    });

    it('should handle empty file', async () => {
      mockInvoke.mockResolvedValue('');

      const result = await readFile('empty.txt');

      expect(result).toBe('');
    });

    it('should handle UTF-8 content', async () => {
      const utf8Content = 'Hello 世界! 🌍 émojis and spëcial chars';
      mockInvoke.mockResolvedValue(utf8Content);

      const result = await readFile('utf8.txt');

      expect(result).toBe(utf8Content);
    });

    it('should handle errors', async () => {
      const error = new Error('File not found');
      const parsedError = mockCreatePluginError('FS_FILE_NOT_FOUND', 'File not found');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(readFile('nonexistent.txt')).rejects.toThrow(parsedError);
      expect(mockParseFsError).toHaveBeenCalledWith(error, {
        path: 'nonexistent.txt',
        method: 'plugin_fs_read_file',
        args: { path: 'nonexistent.txt' }
      });
    });
  });

  describe('writeFile', () => {
    it('should write file content successfully', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await writeFile('output.txt', 'Hello, World!');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', {
        path: 'output.txt',
        content: 'Hello, World!'
      });
    });

    it('should handle empty content', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await writeFile('empty.txt', '');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', {
        path: 'empty.txt',
        content: ''
      });
    });

    it('should handle large content', async () => {
      const largeContent = 'x'.repeat(10000);
      mockInvoke.mockResolvedValue(undefined);

      await writeFile('large.txt', largeContent);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', {
        path: 'large.txt',
        content: largeContent
      });
    });

    it('should handle special characters', async () => {
      const specialContent = 'Line 1\nLine 2\tTabbed\r\nWindows line ending';
      mockInvoke.mockResolvedValue(undefined);

      await writeFile('special.txt', specialContent);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', {
        path: 'special.txt',
        content: specialContent
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Permission denied');
      const parsedError = mockCreatePluginError('FS_PERMISSION_DENIED', 'Permission denied');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(writeFile('readonly.txt', 'content')).rejects.toThrow(parsedError);
    });
  });

  describe('exists', () => {
    it('should return true for existing file', async () => {
      mockInvoke.mockResolvedValue(true);

      const result = await exists('existing.txt');

      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_exists', { path: 'existing.txt' });
    });

    it('should return false for non-existing file', async () => {
      mockInvoke.mockResolvedValue(false);

      const result = await exists('nonexistent.txt');

      expect(result).toBe(false);
    });

    it('should work with directories', async () => {
      mockInvoke.mockResolvedValue(true);

      const result = await exists('some/directory');

      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_exists', { path: 'some/directory' });
    });

    it('should handle errors', async () => {
      const error = new Error('Access denied');
      const parsedError = mockCreatePluginError('FS_ACCESS_DENIED', 'Access denied');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(exists('protected/file.txt')).rejects.toThrow(parsedError);
    });
  });

  describe('createDir', () => {
    it('should create directory with default recursive option', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await createDir('new/directory');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_create_dir', {
        path: 'new/directory',
        recursive: true
      });
    });

    it('should create directory with explicit recursive option', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await createDir('another/directory', false);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_create_dir', {
        path: 'another/directory',
        recursive: false
      });
    });

    it('should handle single directory creation', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await createDir('single');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_create_dir', {
        path: 'single',
        recursive: true
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Directory already exists');
      const parsedError = mockCreatePluginError('FS_DIR_EXISTS', 'Directory already exists');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(createDir('existing')).rejects.toThrow(parsedError);
    });
  });

  describe('listDir', () => {
    it('should list directory contents', async () => {
      const fileInfos: FileInfo[] = [
        {
          name: 'file1.txt',
          path: 'dir/file1.txt',
          isFile: true,
          isDirectory: false,
          size: 1024,
          modifiedTime: 1640995200000,
          createdTime: 1640995100000
        },
        {
          name: 'subdir',
          path: 'dir/subdir',
          isFile: false,
          isDirectory: true,
          size: 0,
          modifiedTime: 1640995300000,
          createdTime: 1640995200000
        }
      ];
      mockInvoke.mockResolvedValue(fileInfos);

      const result = await listDir('dir');

      expect(result).toEqual(fileInfos);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_list_dir', { path: 'dir' });
    });

    it('should handle empty directory', async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await listDir('empty');

      expect(result).toEqual([]);
    });

    it('should handle root directory', async () => {
      const rootContents: FileInfo[] = [
        {
          name: 'config.json',
          path: 'config.json',
          isFile: true,
          isDirectory: false,
          size: 512,
          modifiedTime: 1640995400000,
          createdTime: 1640995300000
        }
      ];
      mockInvoke.mockResolvedValue(rootContents);

      const result = await listDir('.');

      expect(result).toEqual(rootContents);
    });

    it('should handle errors', async () => {
      const error = new Error('Directory not found');
      const parsedError = mockCreatePluginError('FS_DIR_NOT_FOUND', 'Directory not found');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(listDir('nonexistent')).rejects.toThrow(parsedError);
    });
  });

  describe('deleteFile', () => {
    it('should delete file successfully', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await deleteFile('unwanted.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_delete_file', { path: 'unwanted.txt' });
    });

    it('should handle nested file deletion', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await deleteFile('deep/nested/file.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_delete_file', { path: 'deep/nested/file.txt' });
    });

    it('should handle errors', async () => {
      const error = new Error('File in use');
      const parsedError = mockCreatePluginError('FS_FILE_IN_USE', 'File in use');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(deleteFile('locked.txt')).rejects.toThrow(parsedError);
    });
  });

  describe('deleteDir', () => {
    it('should delete directory with default non-recursive option', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await deleteDir('empty-dir');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_delete_dir', {
        path: 'empty-dir',
        recursive: false
      });
    });

    it('should delete directory recursively', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await deleteDir('full-dir', true);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_delete_dir', {
        path: 'full-dir',
        recursive: true
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Directory not empty');
      const parsedError = mockCreatePluginError('FS_DIR_NOT_EMPTY', 'Directory not empty');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(deleteDir('non-empty-dir')).rejects.toThrow(parsedError);
    });
  });

  describe('getFileInfo', () => {
    it('should get file information', async () => {
      const fileInfo: FileInfo = {
        name: 'document.pdf',
        path: 'docs/document.pdf',
        isFile: true,
        isDirectory: false,
        size: 2048576,
        modifiedTime: 1640995500000,
        createdTime: 1640995400000
      };
      mockInvoke.mockResolvedValue(fileInfo);

      const result = await getFileInfo('docs/document.pdf');

      expect(result).toEqual(fileInfo);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_get_file_info', { path: 'docs/document.pdf' });
    });

    it('should get directory information', async () => {
      const dirInfo: FileInfo = {
        name: 'images',
        path: 'assets/images',
        isFile: false,
        isDirectory: true,
        size: 0,
        modifiedTime: 1640995600000,
        createdTime: 1640995500000
      };
      mockInvoke.mockResolvedValue(dirInfo);

      const result = await getFileInfo('assets/images');

      expect(result).toEqual(dirInfo);
    });

    it('should handle errors', async () => {
      const error = new Error('Path not found');
      const parsedError = mockCreatePluginError('FS_PATH_NOT_FOUND', 'Path not found');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(getFileInfo('missing/path')).rejects.toThrow(parsedError);
    });
  });

  describe('copyFile', () => {
    it('should copy file successfully', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await copyFile('source.txt', 'destination.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_copy_file', {
        sourcePath: 'source.txt',
        destPath: 'destination.txt'
      });
    });

    it('should copy file to different directory', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await copyFile('docs/original.pdf', 'backup/copy.pdf');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_copy_file', {
        sourcePath: 'docs/original.pdf',
        destPath: 'backup/copy.pdf'
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Source file not found');
      const parsedError = mockCreatePluginError('FS_SOURCE_NOT_FOUND', 'Source file not found');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(copyFile('missing.txt', 'copy.txt')).rejects.toThrow(parsedError);
      expect(mockParseFsError).toHaveBeenCalledWith(error, {
        path: 'missing.txt', // sourcePath takes precedence
        method: 'plugin_fs_copy_file',
        args: { sourcePath: 'missing.txt', destPath: 'copy.txt' }
      });
    });
  });

  describe('moveFile', () => {
    it('should move file successfully', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await moveFile('old-location.txt', 'new-location.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
        sourcePath: 'old-location.txt',
        destPath: 'new-location.txt'
      });
    });

    it('should rename file', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await moveFile('oldname.txt', 'newname.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
        sourcePath: 'oldname.txt',
        destPath: 'newname.txt'
      });
    });

    it('should move file between directories', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await moveFile('temp/file.txt', 'permanent/file.txt');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
        sourcePath: 'temp/file.txt',
        destPath: 'permanent/file.txt'
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Destination already exists');
      const parsedError = mockCreatePluginError('FS_DEST_EXISTS', 'Destination already exists');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(moveFile('source.txt', 'existing.txt')).rejects.toThrow(parsedError);
    });
  });

  describe('fs namespace', () => {
    it('should have all expected methods', () => {
      expect(fs.readFile).toBe(readFile);
      expect(fs.writeFile).toBe(writeFile);
      expect(fs.exists).toBe(exists);
      expect(fs.createDir).toBe(createDir);
      expect(fs.listDir).toBe(listDir);
      expect(fs.deleteFile).toBe(deleteFile);
      expect(fs.deleteDir).toBe(deleteDir);
      expect(fs.getFileInfo).toBe(getFileInfo);
      expect(fs.copyFile).toBe(copyFile);
      expect(fs.moveFile).toBe(moveFile);
    });

    it('should work through namespace methods', async () => {
      mockInvoke.mockResolvedValue('namespace content');

      const result = await fs.readFile('namespace-test.txt');

      expect(result).toBe('namespace content');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_read_file', { path: 'namespace-test.txt' });
    });
  });

  describe('error handling', () => {
    it('should not re-parse PluginError instances', async () => {
      const pluginError = mockCreatePluginError('FS_ACCESS_DENIED', 'Already parsed');
      mockInvoke.mockRejectedValue(pluginError);
      // Mock isPluginError to return true for this specific error
      mockErrorUtils.isPluginError.mockReturnValue(true);

      await expect(readFile('test.txt')).rejects.toThrow(pluginError);
      expect(mockParseFsError).not.toHaveBeenCalled();
    });

    it('should pass correct context to error parser for single path operations', async () => {
      const error = new Error('Test error');
      const parsedError = mockCreatePluginError('FS_ERROR', 'Parsed error');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(readFile('test.txt')).rejects.toThrow(parsedError);
      expect(mockParseFsError).toHaveBeenCalledWith(error, {
        path: 'test.txt',
        method: 'plugin_fs_read_file',
        args: { path: 'test.txt' }
      });
    });

    it('should pass correct context to error parser for dual path operations', async () => {
      const error = new Error('Copy error');
      const parsedError = mockCreatePluginError('FS_COPY_ERROR', 'Copy error');
      
      mockInvoke.mockRejectedValue(error);
      mockParseFsError.mockReturnValue(parsedError);

      await expect(copyFile('source.txt', 'dest.txt')).rejects.toThrow(parsedError);
      expect(mockParseFsError).toHaveBeenCalledWith(error, {
        path: 'source.txt', // sourcePath takes precedence
        method: 'plugin_fs_copy_file',
        args: { sourcePath: 'source.txt', destPath: 'dest.txt' }
      });
    });
  });

  describe('concurrent operations', () => {
    it('should handle multiple simultaneous file operations', async () => {
      mockInvoke
        .mockResolvedValueOnce('file1 content') // readFile
        .mockResolvedValueOnce(undefined) // writeFile
        .mockResolvedValueOnce(true) // exists
        .mockResolvedValueOnce(undefined); // deleteFile

      const [content, , fileExists, ] = await Promise.all([
        readFile('file1.txt'),
        writeFile('file2.txt', 'content'),
        exists('file3.txt'),
        deleteFile('file4.txt')
      ]);

      expect(content).toBe('file1 content');
      expect(fileExists).toBe(true);
      expect(mockInvoke).toHaveBeenCalledTimes(4);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const error = new Error('Operation failed');
      const parsedError = mockCreatePluginError('FS_ERROR', 'Operation failed');
      
      mockInvoke
        .mockResolvedValueOnce('success content') // success
        .mockRejectedValueOnce(error); // failure
      
      mockParseFsError.mockReturnValue(parsedError);

      const results = await Promise.allSettled([
        readFile('success.txt'),
        readFile('failure.txt')
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect((results[0] as PromiseFulfilledResult<string>).value).toBe('success content');
      expect(results[1].status).toBe('rejected');
      expect((results[1] as PromiseRejectedResult).reason).toBe(parsedError);
    });
  });

  describe('path handling', () => {
    it('should handle various path formats', async () => {
      mockInvoke.mockClear();
      mockInvoke.mockResolvedValue(true);

      // Test different path formats
      await exists('simple.txt');
      await exists('./relative.txt');
      await exists('deep/nested/path/file.txt');
      await exists('../parent.txt');

      expect(mockInvoke).toHaveBeenCalledTimes(4);
      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'plugin_fs_exists', { path: 'simple.txt' });
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'plugin_fs_exists', { path: './relative.txt' });
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'plugin_fs_exists', { path: 'deep/nested/path/file.txt' });
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'plugin_fs_exists', { path: '../parent.txt' });
    });

    it('should handle special characters in paths', async () => {
      mockInvoke.mockClear();
      mockInvoke.mockResolvedValue(true);

      await exists('file with spaces.txt');
      await exists('file-with-dashes.txt');
      await exists('file_with_underscores.txt');
      await exists('file.with.dots.txt');

      expect(mockInvoke).toHaveBeenCalledTimes(4);
    });
  });
});