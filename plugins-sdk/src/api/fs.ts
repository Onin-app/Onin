import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';
import { errorUtils } from '../types/errors';
import { parseFsError } from '../utils/error-parser';

// 文件信息接口
export interface FileInfo {
  name: string;
  path: string;
  isFile: boolean;
  isDirectory: boolean;
  size: number;
  modifiedTime: number;
  createdTime: number;
}

// 通用的文件系统调用辅助函数
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

    // 使用统一的错误解析器
    throw parseFsError(error, {
      path: args?.path || args?.sourcePath,
      method,
      args
    });
  }
}

/**
 * 读取文件内容
 * @param path 相对于插件目录的文件路径
 * @returns 文件内容字符串
 */
export function readFile(path: string): Promise<string> {
  return callFsApi<string>("plugin_fs_read_file", { path });
}

/**
 * 写入文件内容
 * @param path 相对于插件目录的文件路径
 * @param content 要写入的内容
 */
export function writeFile(path: string, content: string): Promise<void> {
  return callFsApi("plugin_fs_write_file", { path, content });
}

/**
 * 检查文件/目录是否存在
 * @param path 相对于插件目录的路径
 * @returns 是否存在
 */
export function exists(path: string): Promise<boolean> {
  return callFsApi<boolean>("plugin_fs_exists", { path });
}

/**
 * 创建目录
 * @param path 相对于插件目录的目录路径
 * @param recursive 是否递归创建父目录
 */
export function createDir(path: string, recursive: boolean = true): Promise<void> {
  return callFsApi("plugin_fs_create_dir", { path, recursive });
}

/**
 * 列出目录内容
 * @param path 相对于插件目录的目录路径
 * @returns 目录中的文件和子目录信息
 */
export function listDir(path: string): Promise<FileInfo[]> {
  return callFsApi<FileInfo[]>("plugin_fs_list_dir", { path });
}

/**
 * 删除文件
 * @param path 相对于插件目录的文件路径
 */
export function deleteFile(path: string): Promise<void> {
  return callFsApi("plugin_fs_delete_file", { path });
}

/**
 * 删除目录
 * @param path 相对于插件目录的目录路径
 * @param recursive 是否递归删除子目录和文件
 */
export function deleteDir(path: string, recursive: boolean = false): Promise<void> {
  return callFsApi("plugin_fs_delete_dir", { path, recursive });
}

/**
 * 获取文件信息
 * @param path 相对于插件目录的文件路径
 * @returns 文件详细信息
 */
export function getFileInfo(path: string): Promise<FileInfo> {
  return callFsApi<FileInfo>("plugin_fs_get_file_info", { path });
}

/**
 * 复制文件
 * @param sourcePath 源文件路径（相对于插件目录）
 * @param destPath 目标文件路径（相对于插件目录）
 */
export function copyFile(sourcePath: string, destPath: string): Promise<void> {
  return callFsApi("plugin_fs_copy_file", { sourcePath, destPath });
}

/**
 * 移动/重命名文件
 * @param sourcePath 源文件路径（相对于插件目录）
 * @param destPath 目标文件路径（相对于插件目录）
 */
export function moveFile(sourcePath: string, destPath: string): Promise<void> {
  return callFsApi("plugin_fs_move_file", { sourcePath, destPath });
}

// 创建文件系统 API 命名空间
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