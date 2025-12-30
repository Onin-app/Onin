import { errorCode, createError, PluginError } from '../types/errors';

/**
 * Error parser - converts low-level errors to structured plugin errors
 * @fileoverview Provides error parsing utilities for different API categories
 */

export interface ErrorPattern {
  patterns: string[];
  createError: (message: string, context?: Record<string, any>) => PluginError;
}

/**
 * Error matching result
 * @interface ErrorMatchResult
 */
interface ErrorMatchResult {
  matched: boolean;
  error?: PluginError;
  pattern?: ErrorPattern;
}

/**
 * HTTP error parsing rules
 * Note: Patterns are sorted by priority, more specific errors should come first
 */
export const httpErrorPatterns: ErrorPattern[] = [
  // Specific network errors (high priority)
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

  // Generic permission errors (low priority, placed last)
  {
    patterns: ['Permission denied', 'permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      `URL: ${context?.url || 'unknown'}`, 
      { originalError: message, ...context }
    )
  }
];

/**
 * File system error parsing rules
 * Note: Patterns are sorted by priority, more specific errors should come first
 */
export const fsErrorPatterns: ErrorPattern[] = [
  // Specific file system errors (high priority)
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
  // Generic permission errors (low priority, placed last)
  {
    patterns: ['Permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      `file system operation on ${context?.path || 'unknown'}`, 
      { originalError: message, ...context }
    )
  }
];

/**
 * Clipboard error parsing rules
 * Note: Patterns are sorted by priority, more specific errors should come first
 */
export const clipboardErrorPatterns: ErrorPattern[] = [
  // Specific clipboard errors (high priority)
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
  // Generic permission errors (low priority, placed last)
  {
    patterns: ['Permission denied', 'permission denied', 'PERMISSION_DENIED'],
    createError: (message, context) => createError.common.permissionDenied(
      'clipboard access', 
      { originalError: message, ...context }
    )
  }
];

/**
 * Dialog error parsing rules
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
 * Safe error message extraction
 * @param error - The error to extract message from
 * @returns The error message string
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
 * Matches error patterns against a message
 * @param message - The error message to match
 * @param patterns - Array of error patterns to match against
 * @returns Error match result
 */
function matchErrorPattern(message: string, patterns: ErrorPattern[]): ErrorMatchResult {
  const lowerMessage = message.toLowerCase();
  
  // Match error patterns by priority
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
 * Generic error parser
 * @param error - The error to parse
 * @param patterns - Array of error patterns to match against
 * @param context - Additional context information
 * @returns Parsed plugin error
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
      // If creating specific error fails, fallback to generic error
      console.warn('Failed to create specific error, falling back to generic error:', createError);
    }
  }
  
  // If no specific pattern matched, return generic error
  return createError.common.unknown(message, { 
    originalError: message, 
    parseContext: 'No pattern matched',
    ...context 
  });
}

/**
 * HTTP error parser
 * @param error - The error to parse
 * @param context - Additional context information
 * @returns Parsed plugin error
 */
export function parseHttpError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, httpErrorPatterns, { 
    ...context, 
    errorType: 'http' 
  });
}

/**
 * File system error parser
 * @param error - The error to parse
 * @param context - Additional context information
 * @returns Parsed plugin error
 */
export function parseFsError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, fsErrorPatterns, { 
    ...context, 
    errorType: 'filesystem' 
  });
}

/**
 * Clipboard error parser
 * @param error - The error to parse
 * @param context - Additional context information
 * @returns Parsed plugin error
 */
export function parseClipboardError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, clipboardErrorPatterns, { 
    ...context, 
    errorType: 'clipboard' 
  });
}

/**
 * Dialog error parser
 * @param error - The error to parse
 * @param context - Additional context information
 * @returns Parsed plugin error
 */
export function parseDialogError(error: unknown, context?: Record<string, any>): PluginError {
  return parseError(error, dialogErrorPatterns, { 
    ...context, 
    errorType: 'dialog' 
  });
}