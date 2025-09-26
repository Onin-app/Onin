/**
 * 错误码定义 - 按命名空间组织
 */
export const errorCode = {
  // 通用错误
  common: {
    UNKNOWN: 'COMMON_UNKNOWN',
    PERMISSION_DENIED: 'COMMON_PERMISSION_DENIED',
    INVALID_ARGUMENT: 'COMMON_INVALID_ARGUMENT',
  },

  // HTTP 相关错误
  http: {
    NETWORK_ERROR: 'HTTP_NETWORK_ERROR',
    TIMEOUT: 'HTTP_TIMEOUT',
    HTTP_ERROR: 'HTTP_HTTP_ERROR', // 通用HTTP错误，包含状态码信息
  },

  // 文件系统错误
  fs: {
    FILE_NOT_FOUND: 'FS_FILE_NOT_FOUND',
    FILE_ACCESS_DENIED: 'FS_FILE_ACCESS_DENIED',
    DIRECTORY_NOT_FOUND: 'FS_DIRECTORY_NOT_FOUND',
    DISK_FULL: 'FS_DISK_FULL',
    FILE_ALREADY_EXISTS: 'FS_FILE_ALREADY_EXISTS',
    INVALID_PATH: 'FS_INVALID_PATH',
    READ_ONLY_FILESYSTEM: 'FS_READ_ONLY_FILESYSTEM',
  },

  // 剪贴板错误
  clipboard: {
    UNAVAILABLE: 'CLIPBOARD_UNAVAILABLE',
    FORMAT_UNSUPPORTED: 'CLIPBOARD_FORMAT_UNSUPPORTED',
    EMPTY: 'CLIPBOARD_EMPTY',
    ACCESS_DENIED: 'CLIPBOARD_ACCESS_DENIED',
  },

  // 对话框错误
  dialog: {
    CANCELLED: 'DIALOG_CANCELLED',
    UNAVAILABLE: 'DIALOG_UNAVAILABLE',
    INVALID_OPTIONS: 'DIALOG_INVALID_OPTIONS',
  },

  // 存储错误
  storage: {
    QUOTA_EXCEEDED: 'STORAGE_QUOTA_EXCEEDED',
    UNAVAILABLE: 'STORAGE_UNAVAILABLE',
  },
} as const;

// 扁平化的错误码类型
export type ErrorCode =
  | typeof errorCode.common[keyof typeof errorCode.common]
  | typeof errorCode.http[keyof typeof errorCode.http]
  | typeof errorCode.fs[keyof typeof errorCode.fs]
  | typeof errorCode.clipboard[keyof typeof errorCode.clipboard]
  | typeof errorCode.dialog[keyof typeof errorCode.dialog]
  | typeof errorCode.storage[keyof typeof errorCode.storage];

/**
 * 插件错误接口 - 使用普通对象而非类
 */
export interface PluginError extends Error {
  readonly name: 'PluginError';
  readonly code: ErrorCode;
  readonly context?: Record<string, any>;
}

/**
 * 创建插件错误的工厂函数
 */
function createPluginError(
  code: ErrorCode,
  message: string,
  context?: Record<string, any>
): PluginError {
  const error = new Error(message) as PluginError;
  Object.defineProperty(error, 'name', { value: 'PluginError', writable: false });
  (error as any).code = code;
  (error as any).context = context;
  return error;
}

/**
 * 错误工厂函数 - 按命名空间组织
 */
export const createError = {
  // 通用错误
  common: {
    unknown: (message: string, context?: Record<string, any>) =>
      createPluginError(errorCode.common.UNKNOWN, message, context),

    permissionDenied: (resource: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.common.PERMISSION_DENIED,
        `Permission denied for ${resource}`,
        context
      ),

    invalidArgument: (argument: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.common.INVALID_ARGUMENT,
        `Invalid argument: ${argument}`,
        context
      ),
  },

  // HTTP 错误
  http: {
    networkError: (message: string, context?: Record<string, any>) =>
      createPluginError(errorCode.http.NETWORK_ERROR, message, context),

    timeout: (url: string, timeout: number, context?: Record<string, any>) =>
      createPluginError(
        errorCode.http.TIMEOUT,
        `Request to ${url} timed out after ${timeout}ms`,
        { url, timeout, ...context }
      ),

    httpError: (status: number, statusText: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.http.HTTP_ERROR,
        `HTTP ${status}: ${statusText}`,
        { status, statusText, ...context }
      ),
  },

  // 文件系统错误
  fs: {
    fileNotFound: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_NOT_FOUND,
        `File not found: ${path}`,
        { path, ...context }
      ),

    fileAccessDenied: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_ACCESS_DENIED,
        `Access denied: ${path}`,
        { path, ...context }
      ),

    diskFull: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.DISK_FULL,
        `Disk full while accessing: ${path}`,
        { path, ...context }
      ),

    fileAlreadyExists: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.FILE_ALREADY_EXISTS,
        `File already exists: ${path}`,
        { path, ...context }
      ),

    invalidPath: (path: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.fs.INVALID_PATH,
        `Invalid path: ${path}`,
        { path, ...context }
      ),
  },

  // 剪贴板错误
  clipboard: {
    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.UNAVAILABLE,
        'Clipboard is not available',
        context
      ),

    formatUnsupported: (format?: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.FORMAT_UNSUPPORTED,
        `Clipboard format not supported${format ? `: ${format}` : ''}`,
        { format, ...context }
      ),

    empty: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.EMPTY,
        'Clipboard is empty',
        context
      ),

    accessDenied: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.clipboard.ACCESS_DENIED,
        'Clipboard access denied',
        context
      ),
  },

  // 对话框错误
  dialog: {
    cancelled: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.CANCELLED,
        'Dialog was cancelled by user',
        context
      ),

    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.UNAVAILABLE,
        'Dialog is not available',
        context
      ),

    invalidOptions: (reason?: string, context?: Record<string, any>) =>
      createPluginError(
        errorCode.dialog.INVALID_OPTIONS,
        `Invalid dialog options${reason ? `: ${reason}` : ''}`,
        { reason, ...context }
      ),
  },

  // 存储错误
  storage: {
    quotaExceeded: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.storage.QUOTA_EXCEEDED,
        'Storage quota exceeded',
        context
      ),

    unavailable: (context?: Record<string, any>) =>
      createPluginError(
        errorCode.storage.UNAVAILABLE,
        'Storage is not available',
        context
      ),
  },
};

/**
 * 错误检查工具函数
 */
export const errorUtils = {
  /**
   * 检查是否为插件错误
   */
  isPluginError: (error: any): error is PluginError => {
    return error && error.name === 'PluginError' && typeof error.code === 'string';
  },

  /**
   * 检查错误是否为指定类型
   */
  isErrorCode: (error: any, code: ErrorCode): boolean => {
    return errorUtils.isPluginError(error) && error.code === code;
  },

  /**
   * 检查错误是否为指定类型之一
   */
  isOneOfErrorCodes: (error: any, codes: ErrorCode[]): boolean => {
    return errorUtils.isPluginError(error) && codes.includes(error.code);
  },

  /**
   * 获取错误的详细信息
   */
  getErrorInfo: (error: any) => {
    if (errorUtils.isPluginError(error)) {
      return {
        name: error.name,
        code: error.code,
        message: error.message,
        context: error.context,
      };
    }
    return null;
  },
};