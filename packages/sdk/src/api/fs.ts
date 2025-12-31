import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';
import { errorUtils } from '../types/errors';
import { parseFsError } from '../utils/error-parser';

/**
 * File or directory metadata
 * @interface FileInfo
 * @since 0.1.0
 * @group Types
 */
export interface FileInfo {
  /** File or directory name */
  name: string;
  /** Complete path of the file or directory */
  path: string;
  /** True if it's a file */
  isFile: boolean;
  /** True if it's a directory */
  isDirectory: boolean;
  /** File size in bytes */
  size: number;
  /** Last modified time (Unix timestamp) */
  modifiedTime: number;
  /** Creation time (Unix timestamp) */
  createdTime: number;
}

/**
 * Generic file system API call helper function
 * @typeParam T - The expected return type
 * @param method - The file system method to call
 * @param args - Optional arguments for the method
 * @returns Promise resolving to the method result
 * @internal
 * @group Core
 */
async function callFsApi<T = any>(method: string, args?: any): Promise<T> {
  try {
    return await dispatch({
      webview: () => invoke<T>(method, args),
      headless: () => invoke<T>(method, args),
    });
  } catch (error: any) {
    if (errorUtils.isPluginError(error)) {
      throw error;
    }

    // Use unified error parser
    throw parseFsError(error, {
      path: args?.path || args?.sourcePath,
      method,
      args,
    });
  }
}

/**
 * Reads the text file content at the specified path with UTF-8 encoding.
 * @param path - File path relative to the plugin data directory.
 * @returns Promise that resolves to the file's text content.
 * @throws {PluginError} With code `FS_FILE_NOT_FOUND` when the file doesn't exist
 * @throws {PluginError} With code `FS_FILE_ACCESS_DENIED` when permission is denied
 * @throws {PluginError} With code `FS_INVALID_PATH` when the path is malformed
 * @throws {PluginError} With code `PERMISSION_DENIED` for general file system permission issues
 * @example
 * ```typescript
 * // Read configuration file
 * try {
 *   const content = await fs.readFile('config.json');
 *   const config = JSON.parse(content);
 *   console.log('Configuration loaded:', config);
 * } catch (error) {
 *   if (errorUtils.isErrorCode(error, 'FS_FILE_NOT_FOUND')) {
 *     console.log('Config file not found, using defaults');
 *   } else {
 *     console.error('Failed to read config:', error.message);
 *   }
 * }
 *
 * // Read log file
 * const logContent = await fs.readFile('logs/app.log');
 * console.log('Recent logs:', logContent.split('\n').slice(-10));
 * ```
 * @since 0.1.0
 * @group API
 */
export function readFile(path: string): Promise<string> {
  return callFsApi<string>('plugin_fs_read_file', { path });
}

/**
 * Writes text content to a file at the specified path. Creates the file if it doesn't exist, or overwrites it if it does.
 * @param path - File path relative to the plugin data directory.
 * @param content - Text content to write to the file.
 * @returns Promise that resolves when the file write operation is complete.
 * @example
 * await fs.writeFile('log.txt', 'This is a log entry.');
 */
export function writeFile(path: string, content: string): Promise<void> {
  return callFsApi('plugin_fs_write_file', { path, content });
}

/**
 * Checks if a file or directory exists at the specified path.
 * @param path - Path relative to the plugin data directory.
 * @returns Promise that resolves to true if the path exists, false otherwise.
 * @example
 * if (await fs.exists('config.json')) {
 *   console.log('Config file exists.');
 * }
 */
export function exists(path: string): Promise<boolean> {
  return callFsApi<boolean>('plugin_fs_exists', { path });
}

/**
 * Creates a new directory.
 * @param path - Directory path relative to the plugin data directory.
 * @param recursive - Whether to recursively create all non-existing parent directories.
 * @returns Promise that resolves when the directory creation operation is complete.
 * @example
 * await fs.createDir('data/logs', true);
 */
export function createDir(
  path: string,
  recursive: boolean = true,
): Promise<void> {
  return callFsApi('plugin_fs_create_dir', { path, recursive });
}

/**
 * Lists all files and subdirectories in the specified directory.
 * @param path - Directory path relative to the plugin data directory.
 * @returns Promise that resolves to an array of FileInfo objects containing the directory contents.
 * @example
 * const items = await fs.listDir('.');
 * for (const item of items) {
 *   console.log(item.name, item.isDirectory ? '(dir)' : '(file)');
 * }
 */
export function listDir(path: string): Promise<FileInfo[]> {
  return callFsApi<FileInfo[]>('plugin_fs_list_dir', { path });
}

/**
 * Deletes the file at the specified path.
 * @param path - File path relative to the plugin data directory.
 * @returns Promise that resolves when the file deletion operation is complete.
 * @example
 * await fs.deleteFile('temp.txt');
 */
export function deleteFile(path: string): Promise<void> {
  return callFsApi('plugin_fs_delete_file', { path });
}

/**
 * Deletes the directory at the specified path.
 * @param path - Directory path relative to the plugin data directory.
 * @param recursive - Whether to recursively delete all contents in the directory. If false and the directory is not empty, the operation will fail.
 * @returns Promise that resolves when the directory deletion operation is complete.
 * @example
 * await fs.deleteDir('old_data', true);
 */
export function deleteDir(
  path: string,
  recursive: boolean = false,
): Promise<void> {
  return callFsApi('plugin_fs_delete_dir', { path, recursive });
}

/**
 * Gets metadata for the file or directory at the specified path.
 * @param path - File path relative to the plugin data directory.
 * @returns Promise that resolves to a FileInfo object for the file or directory.
 * @example
 * const info = await fs.getFileInfo('data.txt');
 * console.log(`Size: ${info.size} bytes`);
 */
export function getFileInfo(path: string): Promise<FileInfo> {
  return callFsApi<FileInfo>('plugin_fs_get_file_info', { path });
}

/**
 * Copies a file from source path to destination path.
 * @param sourcePath - Source file path relative to the plugin data directory.
 * @param destPath - Destination file path relative to the plugin data directory.
 * @returns Promise that resolves when the file copy operation is complete.
 * @example
 * await fs.copyFile('source.txt', 'destination.txt');
 */
export function copyFile(sourcePath: string, destPath: string): Promise<void> {
  return callFsApi('plugin_fs_copy_file', { sourcePath, destPath });
}

/**
 * Moves or renames a file/directory.
 * @param sourcePath - Source path relative to the plugin data directory.
 * @param destPath - Destination path relative to the plugin data directory.
 * @returns Promise that resolves when the move/rename operation is complete.
 * @example
 * await fs.moveFile('old-name.txt', 'new-name.txt');
 */
export function moveFile(sourcePath: string, destPath: string): Promise<void> {
  return callFsApi('plugin_fs_move_file', { sourcePath, destPath });
}

/**
 * File system API namespace - provides sandboxed file operations for plugins
 *
 * All file operations are restricted to the plugin's data directory for security.
 * Paths are relative to the plugin's isolated storage space, ensuring data
 * separation between plugins and preventing access to system files.
 *
 * **Security**: All file operations are sandboxed within the plugin's data directory.
 * Attempts to access files outside this directory will result in errors.
 *
 * **Path Handling**: All paths are relative to the plugin data directory.
 * Use forward slashes (/) for cross-platform compatibility.
 *
 * **Encoding**: Text files are read/written using UTF-8 encoding by default.
 *
 * **Performance**: For better performance with multiple operations, consider
 * using batch operations when available.
 *
 * @namespace fs
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @see {@link parseFsError} - For file system error handling utilities
 * @example
 * ```typescript
 * import { fs } from 'onin-plugin-sdk';
 *
 * // Basic file operations
 * await fs.writeFile('data.txt', 'Hello World');
 * const content = await fs.readFile('data.txt');
 * console.log(content); // 'Hello World'
 *
 * // Directory operations
 * await fs.createDir('backups', true);
 * const files = await fs.listDir('.');
 * console.log('Files in plugin directory:', files);
 *
 * // File management
 * await fs.copyFile('data.txt', 'backups/data-backup.txt');
 * await fs.moveFile('old-name.txt', 'new-name.txt');
 *
 * // Check file existence
 * if (await fs.exists('config.json')) {
 *   const config = JSON.parse(await fs.readFile('config.json'));
 * } else {
 *   console.log('Config file not found, creating default...');
 *   await fs.writeFile('config.json', JSON.stringify({ version: 1 }));
 * }
 * ```
 */
export const fs = {
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
};
