import { describe, it, expect, vi, beforeEach } from 'vitest';

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
    copyFile,
    moveFile,
    getFileInfo,
    fs,
    type FileInfo
} from '../../../src/api/fs';
import { invoke } from '../../../src/core/ipc';
import { getEnvironment, RuntimeEnvironment } from '../../../src/core/environment';
import { parseFsError } from '../../../src/utils/error-parser';
import { createPluginError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockGetEnvironment = vi.mocked(getEnvironment);
const mockParseFsError = vi.mocked(parseFsError);
const mockCreatePluginError = vi.mocked(createPluginError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('FileSystem API Integration', () => {
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
        mockInvoke.mockResolvedValue('webview content');

        let result = await readFile('test.txt');
        expect(result).toBe('webview content');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_read_file', { path: 'test.txt' });

        // Reset and test headless environment
        mockInvoke.mockClear();
        mockInvoke.mockResolvedValue('headless content');
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

        result = await readFile('test.txt');
        expect(result).toBe('headless content');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_read_file', { path: 'test.txt' });
    });

    it('should properly handle error parsing in different environments', async () => {
        const originalError = new Error('Permission denied');
        const parsedError = mockCreatePluginError('FS_PERMISSION_DENIED', 'Permission denied');

        mockInvoke.mockRejectedValue(originalError);
        mockParseFsError.mockReturnValue(parsedError);

        // Test in webview environment
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

        await expect(writeFile('protected.txt', 'content')).rejects.toThrow(parsedError);
        expect(mockParseFsError).toHaveBeenCalledWith(originalError, {
            path: 'protected.txt',
            method: 'plugin_fs_write_file',
            args: { path: 'protected.txt', content: 'content' }
        });

        // Reset and test in headless environment
        mockParseFsError.mockClear();
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

        await expect(deleteFile('protected.txt')).rejects.toThrow(parsedError);
        expect(mockParseFsError).toHaveBeenCalledWith(originalError, {
            path: 'protected.txt',
            method: 'plugin_fs_delete_file',
            args: { path: 'protected.txt' }
        });
    });

    it('should handle complete file management workflow', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

        // Simulate a complete file management workflow
        const fileInfo: FileInfo = {
            name: 'document.txt',
            path: 'docs/document.txt',
            isFile: true,
            isDirectory: false,
            size: 1024,
            modifiedTime: Date.now(),
            createdTime: Date.now() - 3600000
        };

        mockInvoke
            .mockResolvedValueOnce(undefined) // createDir
            .mockResolvedValueOnce(undefined) // writeFile
            .mockResolvedValueOnce(true) // exists
            .mockResolvedValueOnce(fileInfo) // getFileInfo
            .mockResolvedValueOnce('document content') // readFile
            .mockResolvedValueOnce(undefined) // copyFile
            .mockResolvedValueOnce([fileInfo]) // listDir
            .mockResolvedValueOnce(undefined); // deleteFile

        // Create directory structure
        await fs.createDir('docs');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_create_dir', { path: 'docs', recursive: true });

        // Write initial file
        await fs.writeFile('docs/document.txt', 'document content');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', {
            path: 'docs/document.txt',
            content: 'document content'
        });

        // Check if file exists
        const fileExists = await fs.exists('docs/document.txt');
        expect(fileExists).toBe(true);

        // Get file information
        const info = await fs.getFileInfo('docs/document.txt');
        expect(info).toEqual(fileInfo);

        // Read file content
        const content = await fs.readFile('docs/document.txt');
        expect(content).toBe('document content');

        // Create backup copy
        await fs.copyFile('docs/document.txt', 'docs/document.backup.txt');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_copy_file', {
            sourcePath: 'docs/document.txt',
            destPath: 'docs/document.backup.txt'
        });

        // List directory contents
        const dirContents = await fs.listDir('docs');
        expect(dirContents).toEqual([fileInfo]);

        // Clean up
        await fs.deleteFile('docs/document.backup.txt');
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_delete_file', { path: 'docs/document.backup.txt' });

        expect(mockInvoke).toHaveBeenCalledTimes(8);
    });

    it('should handle directory operations workflow', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

        const dirInfo: FileInfo = {
            name: 'project',
            path: 'workspace/project',
            isFile: false,
            isDirectory: true,
            size: 0,
            modifiedTime: Date.now(),
            createdTime: Date.now() - 7200000
        };

        const subDirContents: FileInfo[] = [
            {
                name: 'src',
                path: 'workspace/project/src',
                isFile: false,
                isDirectory: true,
                size: 0,
                modifiedTime: Date.now(),
                createdTime: Date.now() - 3600000
            },
            {
                name: 'README.md',
                path: 'workspace/project/README.md',
                isFile: true,
                isDirectory: false,
                size: 512,
                modifiedTime: Date.now(),
                createdTime: Date.now() - 1800000
            }
        ];

        mockInvoke
            .mockResolvedValueOnce(undefined) // createDir workspace
            .mockResolvedValueOnce(undefined) // createDir project
            .mockResolvedValueOnce(undefined) // createDir src
            .mockResolvedValueOnce(undefined) // writeFile README
            .mockResolvedValueOnce(subDirContents) // listDir project
            .mockResolvedValueOnce(dirInfo) // getFileInfo project
            .mockResolvedValueOnce(true) // exists check
            .mockResolvedValueOnce(undefined); // deleteDir

        // Create nested directory structure
        await createDir('workspace', true);
        await createDir('workspace/project', true);
        await createDir('workspace/project/src', false);

        // Add a README file
        await writeFile('workspace/project/README.md', '# Project\n\nDescription here.');

        // List project contents
        const projectContents = await listDir('workspace/project');
        expect(projectContents).toEqual(subDirContents);

        // Get project directory info
        const projectInfo = await getFileInfo('workspace/project');
        expect(projectInfo).toEqual(dirInfo);

        // Verify directory exists
        const dirExists = await exists('workspace/project');
        expect(dirExists).toBe(true);

        // Clean up (recursive delete)
        await fs.deleteDir('workspace', true);
        expect(mockInvoke).toHaveBeenLastCalledWith('plugin_fs_delete_dir', {
            path: 'workspace',
            recursive: true
        });
    });

    it('should handle file operations with different content types', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

        // Test different content types
        const textContent = 'Plain text content';
        const jsonContent = JSON.stringify({ key: 'value', number: 42, array: [1, 2, 3] });
        const csvContent = 'Name,Age,City\nJohn,30,New York\nJane,25,Los Angeles';
        const binaryContent = 'Binary-like content with special chars: \x00\x01\x02\xFF';

        mockInvoke.mockResolvedValue(undefined);

        // Write different content types
        await Promise.all([
            writeFile('data/text.txt', textContent),
            writeFile('data/config.json', jsonContent),
            writeFile('data/export.csv', csvContent),
            writeFile('data/binary.dat', binaryContent)
        ]);

        expect(mockInvoke).toHaveBeenCalledTimes(4);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', { path: 'data/text.txt', content: textContent });
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', { path: 'data/config.json', content: jsonContent });
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', { path: 'data/export.csv', content: csvContent });
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_write_file', { path: 'data/binary.dat', content: binaryContent });
    });

    it('should handle file move and rename operations', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

        mockInvoke.mockResolvedValue(undefined);

        // Test various move/rename scenarios
        await moveFile('temp/draft.txt', 'documents/final.txt'); // Move and rename
        await moveFile('old-name.txt', 'new-name.txt'); // Simple rename
        await moveFile('project/v1/file.txt', 'project/v2/file.txt'); // Move between versions

        expect(mockInvoke).toHaveBeenCalledTimes(3);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
            sourcePath: 'temp/draft.txt',
            destPath: 'documents/final.txt'
        });
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
            sourcePath: 'old-name.txt',
            destPath: 'new-name.txt'
        });
        expect(mockInvoke).toHaveBeenCalledWith('plugin_fs_move_file', {
            sourcePath: 'project/v1/file.txt',
            destPath: 'project/v2/file.txt'
        });
    });

    it('should handle concurrent file operations safely', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

        // Simulate concurrent file operations that might happen in real usage
        mockInvoke
            .mockResolvedValueOnce('file1 content') // readFile
            .mockResolvedValueOnce('file2 content') // readFile
            .mockResolvedValueOnce(undefined) // writeFile
            .mockResolvedValueOnce(undefined) // writeFile
            .mockResolvedValueOnce(true) // exists
            .mockResolvedValueOnce(false) // exists
            .mockResolvedValueOnce(undefined) // copyFile
            .mockResolvedValueOnce(undefined); // deleteFile

        const operations = await Promise.all([
            readFile('input1.txt'),
            readFile('input2.txt'),
            writeFile('output1.txt', 'processed content 1'),
            writeFile('output2.txt', 'processed content 2'),
            exists('config.json'),
            exists('temp.txt'),
            copyFile('important.txt', 'backup/important.txt'),
            deleteFile('temp/old-file.txt')
        ]);

        expect(operations[0]).toBe('file1 content');
        expect(operations[1]).toBe('file2 content');
        expect(operations[4]).toBe(true); // config.json exists
        expect(operations[5]).toBe(false); // temp.txt doesn't exist
        expect(mockInvoke).toHaveBeenCalledTimes(8);
    });

    it('should handle error recovery and retry scenarios', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

        const tempError = new Error('Temporary file system error');
        const parsedError = mockCreatePluginError('FS_TEMP_ERROR', 'Temporary error');

        mockInvoke
            .mockRejectedValueOnce(tempError) // First attempt fails
            .mockResolvedValueOnce('success content'); // Second attempt succeeds

        mockParseFsError.mockReturnValue(parsedError);

        // First operation fails
        await expect(readFile('unstable.txt')).rejects.toThrow(parsedError);

        // Second operation succeeds (simulating retry or recovery)
        const result = await readFile('stable.txt');
        expect(result).toBe('success content');

        expect(mockInvoke).toHaveBeenCalledTimes(2);
    });

    it('should handle complex directory listing scenarios', async () => {
        mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

        // Mock complex directory structure
        const complexDirContents: FileInfo[] = [
            {
                name: '.hidden',
                path: 'complex/.hidden',
                isFile: true,
                isDirectory: false,
                size: 128,
                modifiedTime: Date.now(),
                createdTime: Date.now() - 86400000
            },
            {
                name: 'subfolder',
                path: 'complex/subfolder',
                isFile: false,
                isDirectory: true,
                size: 0,
                modifiedTime: Date.now(),
                createdTime: Date.now() - 43200000
            },
            {
                name: 'large-file.dat',
                path: 'complex/large-file.dat',
                isFile: true,
                isDirectory: false,
                size: 10485760, // 10MB
                modifiedTime: Date.now(),
                createdTime: Date.now() - 21600000
            },
            {
                name: 'file with spaces.txt',
                path: 'complex/file with spaces.txt',
                isFile: true,
                isDirectory: false,
                size: 256,
                modifiedTime: Date.now(),
                createdTime: Date.now() - 10800000
            }
        ];

        mockInvoke.mockResolvedValue(complexDirContents);

        const contents = await listDir('complex');

        expect(contents).toEqual(complexDirContents);
        expect(contents).toHaveLength(4);

        // Verify different file types are present
        const files = contents.filter(item => item.isFile);
        const dirs = contents.filter(item => item.isDirectory);

        expect(files).toHaveLength(3);
        expect(dirs).toHaveLength(1);

        // Verify special cases
        const hiddenFile = contents.find(item => item.name === '.hidden');
        const spaceFile = contents.find(item => item.name === 'file with spaces.txt');
        const largeFile = contents.find(item => item.name === 'large-file.dat');

        expect(hiddenFile).toBeDefined();
        expect(spaceFile).toBeDefined();
        expect(largeFile?.size).toBe(10485760);
    });
});