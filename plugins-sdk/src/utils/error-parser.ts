import { errorCode, createError, PluginError } from '../types/errors';

/**
 * 错误解析器 - 将底层错误转换为结构化的插件错误
 */

export interface ErrorPattern {
  patterns: string[];
  createError: (message: string, context?: Record<string, any>) => PluginError;
}

/**
 * 错误匹配结果
 */
interface ErrorMatchResult {
  matched: boolean;
  error?: PluginError;
  pattern?: ErrorPattern;
}

/**
 * HTTP 错误解析规则
 * 注意：模式按优先级排序，更具体的错误应该放在前面
 */
export const httpErrorPatterns: ErrorPattern[] = [
  // 具体的网络错误（优先级高）
  {
    patterns: ['timed out', 'timeout', 'TIMEOUT'],
    createError: (message, context) => createError.http.timeout(
      context?.url || 'unknown', 
      context?.timeout || 30000, 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['Network error', 'network error', 'NETWORK_ERROR', 'connection refused', 'connection failed'],
    createError: (message, context) => createError.http.networkError(message, { originalError: message, ...context })
  },

  // 通用权限错误（优先级低，放在最后）
  {
    patterns: ['Permission denied', 'permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      `URL: ${context?.url || 'unknown'}`, 
      { originalError: message, ...context }
    )
  }
];

/**
 * 文件系统错误解析规则
 * 注意：模式按优先级排序，更具体的错误应该放在前面
 */
export const fsErrorPatterns: ErrorPattern[] = [
  // 具体的文件系统错误（优先级高）
  {
    patterns: ['not found', 'No such file', 'FILE_NOT_FOUND', 'does not exist'],
    createError: (message, context) => createError.fs.fileNotFound(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['Access denied', 'access denied', 'FILE_ACCESS_DENIED'],
    createError: (message, context) => createError.fs.fileAccessDenied(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['No space left', 'disk full', 'DISK_FULL', 'insufficient space'],
    createError: (message, context) => createError.fs.diskFull(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['already exists', 'file exists', 'FILE_ALREADY_EXISTS'],
    createError: (message, context) => createError.fs.fileAlreadyExists(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['invalid path', 'INVALID_PATH', 'malformed path'],
    createError: (message, context) => createError.fs.invalidPath(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  },
  // 通用权限错误（优先级低，放在最后）
  {
    patterns: ['Permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      `file system operation on ${context?.path || 'unknown'}`, 
      { originalError: message, ...context }
    )
  }
];

/**
 * 剪贴板错误解析规则
 * 注意：模式按优先级排序，更具体的错误应该放在前面
 */
export const clipboardErrorPatterns: ErrorPattern[] = [
  // 具体的剪贴板错误（优先级高）
  {
    patterns: ['format not supported', 'unsupported format', 'FORMAT_UNSUPPORTED'],
    createError: (message, context) => createError.clipboard.formatUnsupported(
      context?.format, 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['empty', 'no content', 'CLIPBOARD_EMPTY'],
    createError: (message, context) => createError.clipboard.empty({ originalError: message, ...context })
  },
  {
    patterns: ['clipboard access denied', 'clipboard ACCESS_DENIED'],
    createError: (message, context) => createError.clipboard.accessDenied({ originalError: message, ...context })
  },
  {
    patterns: ['unavailable', 'not available', 'CLIPBOARD_UNAVAILABLE', 'clipboard not accessible'],
    createError: (message, context) => createError.clipboard.unavailable({ originalError: message, ...context })
  },
  // 通用权限错误（优先级低，放在最后）
  {
    patterns: ['Permission denied', 'permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      'clipboard access', 
      { originalError: message, ...context }
    )
  }
];

/**
 * 对话框错误解析规则
 */
export const dialogErrorPatterns: ErrorPattern[] = [
  {
    patterns: ['cancelled', 'canceled', 'DIALOG_CANCELLED', 'user cancelled'],
    createError: (message, context) => createError.dialog.cancelled({ originalError: message, ...context })
  },
  {
    patterns: ['unavailable', 'not available', 'DIALOG_UNAVAILABLE', 'dialog not supported'],
    createError: (message, context) => createError.dialog.unavailable({ originalError: message, ...context })
  },
  {
    patterns: ['invalid options', 'INVALID_OPTIONS', 'malformed options'],
    createError: (message, context) => createError.dialog.invalidOptions(
      context?.reason, 
      { originalError: message, ...context }
    )
  },
  {
    patterns: ['Permission denied', 'permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      'dialog access', 
      { originalError: message, ...context }
    )
  }
];

/**
 * 安全的错误消息提取
 */
function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  
  if (error && typeof error === 'object') {
    const errorObj = error as Record<string, any>;
    if (typeof errorObj.message === 'string') {
      return errorObj.message;
    }
    if (typeof errorObj.error === 'string') {
      return errorObj.error;
    }
  }
  
  return 'Unknown error';
}

/**
 * 匹配错误模式
 */
function matchErrorPattern(message: string, patterns: ErrorPattern[]): ErrorMatchResult {
  const lowerMessage = message.toLowerCase();
  
  // 按优先级匹配错误模式
  for (const pattern of patterns) {
    for (const patternStr of pattern.patterns) {
      if (lowerMessage.includes(patternStr.toLowerCase())) {
        return {
          matched: true,
          pattern,
        };
      }
    }
  }
  
  return { matched: false };
}

/**
 * 通用错误解析器
 */
export function parseError(
  error: unknown, 
  patterns: ErrorPattern[], 
  context?: Record<string, any>
): PluginError {
  const message = extractErrorMessage(error);
  const matchResult = matchErrorPattern(message, patterns);
  
  if (matchResult.matched && matchResult.pattern) {
    try {
      return matchResult.pattern.createError(message, context);
    } catch (createError) {
      // 如果创建特定错误失败，降级到通用错误
      console.warn('Failed to create specific error, falling back to generic error:', createError);
    }
  }
  
  // 如果没有匹配到特定模式，返回通用错误
  return createError.common.unknown(message, { 
    originalError: message, 
    parseContext: 'No pattern matched',
    ...context 
  });
}

/**
 * HTTP 错误解析器
 */
export function parseHttpError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, httpErrorPatterns, { 
    ...context, 
    errorType: 'http' 
  });
}

/**
 * 文件系统错误解析器
 */
export function parseFsError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, fsErrorPatterns, { 
    ...context, 
    errorType: 'filesystem' 
  });
}

/**
 * 剪贴板错误解析器
 */
export function parseClipboardError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, clipboardErrorPatterns, { 
    ...context, 
    errorType: 'clipboard' 
  });
}

/**
 * 对话框错误解析器
 */
export function parseDialogError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, dialogErrorPatterns, { 
    ...context, 
    errorType: 'dialog' 
  });
}