// 插件环境类型
export type PluginEnvironment = 'headless' | 'ui';

// API 响应类型
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// 通知选项类型
export interface NotificationOptions {
  title: string;
  body?: string;
  icon?: string;
}

// SDK 功能函数接口
export interface PluginSDKFunctions {
  getEnvironment: () => PluginEnvironment;
  showNotification: (options: NotificationOptions) => Promise<ApiResponse<void>>;
}

// 错误代码常量
export const ERROR_CODES = {
  UNSUPPORTED_ENVIRONMENT: 'UNSUPPORTED_ENVIRONMENT',
  VALIDATION_ERROR: 'VALIDATION_ERROR',
  INVOKE_ERROR: 'INVOKE_ERROR',
  UNKNOWN_ERROR: 'UNKNOWN_ERROR'
} as const;

export type ErrorCode = typeof ERROR_CODES[keyof typeof ERROR_CODES];

// 错误类型
export class PluginSDKError extends Error {
  constructor(message: string, public code?: ErrorCode) {
    super(message);
    this.name = 'PluginSDKError';
  }
}

// 验证错误类型
export class ValidationError extends PluginSDKError {
  constructor(message: string, public field?: string) {
    super(message, ERROR_CODES.VALIDATION_ERROR);
    this.name = 'ValidationError';
  }
}

// 调用错误类型
export class InvokeError extends PluginSDKError {
  constructor(message: string, public originalError?: unknown) {
    super(message, ERROR_CODES.INVOKE_ERROR);
    this.name = 'InvokeError';
  }
}