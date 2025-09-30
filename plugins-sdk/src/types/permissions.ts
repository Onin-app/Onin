/**
 * 插件权限配置类型定义
 */

export interface HttpPermission {
  /** 是否启用 HTTP 权限 */
  enable: boolean;
  /** 允许访问的 URL 列表，支持通配符 */
  allowUrls: string[];
  /** 请求超时时间（毫秒），可选 */
  timeout?: number;
  /** 最大重试次数，可选 */
  maxRetries?: number;
}

export interface StoragePermission {
  /** 是否启用存储权限 */
  enable: boolean;
  /** 是否允许本地存储 */
  local: boolean;
  /** 是否允许会话存储 */
  session: boolean;
  /** 最大存储大小，可选 */
  maxSize?: string;
}

export interface NotificationPermission {
  /** 是否启用通知权限 */
  enable: boolean;
  /** 是否允许声音通知 */
  sound: boolean;
  /** 是否允许徽章通知 */
  badge: boolean;
}

export interface CommandPermission {
  /** 是否启用命令权限 */
  enable: boolean;
  /** 允许的命令列表，支持通配符 */
  allowCommands: string[];
  /** 最大执行时间（毫秒），可选 */
  maxExecutionTime?: number;
}

interface FileSystemPermission {
  /** 是否启用文件系统权限 */
  enable: boolean;
  /** 是否允许读取文件 */
  read: boolean;
  /** 是否允许写入文件 */
  write: boolean;
  /** 是否允许删除文件 */
  delete: boolean;
  /** 最大文件大小限制（字节），可选 */
  maxFileSize?: number;
}

interface DialogPermission {
  /** 是否启用对话框权限 */
  enable: boolean;
  /** 是否允许消息对话框 */
  message: boolean;
  /** 是否允许确认对话框 */
  confirm: boolean;
  /** 是否允许文件选择对话框 */
  fileDialog: boolean;
}

interface ClipboardPermission {
  /** 是否启用剪贴板权限 */
  enable: boolean;
  /** 是否允许读取文本 */
  readText: boolean;
  /** 是否允许写入文本 */
  writeText: boolean;
  /** 是否允许读取图像 */
  readImage: boolean;
  /** 是否允许写入图像 */
  writeImage: boolean;
  /** 是否允许清空剪贴板 */
  clear: boolean;
}

export interface PluginPermissions {
  /** HTTP 权限配置 */
  http?: HttpPermission;
  /** 存储权限配置 */
  storage?: StoragePermission;
  /** 通知权限配置 */
  notification?: NotificationPermission;
  /** 命令权限配置 */
  command?: CommandPermission;
  /** 文件系统权限配置 */
  fs?: FileSystemPermission;
  /** 对话框权限配置 */
  dialog?: DialogPermission;
  /** 剪贴板权限配置 */
  clipboard?: ClipboardPermission;
}

export interface PluginManifest {
  /** 插件唯一标识符 */
  id: string;
  /** 插件名称 */
  name: string;
  /** 插件版本 */
  version: string;
  /** 插件描述 */
  description: string;
  /** 入口文件路径 */
  entry: string;
  /** 权限配置 */
  permissions?: PluginPermissions;
  /** 命令定义 */
  commands?: Array<{
    id: string;
    name: string;
    description: string;
  }>;
}