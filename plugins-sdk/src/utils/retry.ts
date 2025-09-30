import { errorCode, errorUtils } from '../types/errors';

/**
 * 重试配置选项
 */
export interface RetryOptions {
  /** 最大重试次数 */
  maxRetries?: number;
  /** 基础延迟时间（毫秒） */
  baseDelay?: number;
  /** 是否使用指数退避 */
  exponentialBackoff?: boolean;
  /** 最大延迟时间（毫秒） */
  maxDelay?: number;
  /** 自定义重试条件判断函数 */
  shouldRetry?: (error: unknown) => boolean;
  /** 自定义延迟计算函数 */
  getDelay?: (error: unknown, attempt: number) => number;
  /** 重试前的回调函数 */
  onRetry?: (error: unknown, attempt: number, delay: number) => void;
}

/**
 * 检查错误是否可以重试
 */
export function isRetryableError(error: unknown): boolean {
  if (errorUtils.isPluginError(error)) {
    switch (error.code) {
      case errorCode.http.NETWORK_ERROR:
      case errorCode.http.TIMEOUT:
        return true;

      case errorCode.http.HTTP_ERROR:
        // 基于标准 HTTP 状态码判断是否可重试
        const status = error.context?.status;
        return status === 429 || // Too Many Requests
          status === 500 || // Internal Server Error
          status === 502 || // Bad Gateway
          status === 503 || // Service Unavailable
          status === 504;   // Gateway Timeout

      case errorCode.clipboard.UNAVAILABLE:
        return true;

      default:
        return false;
    }
  }
  return false;
}

/**
 * 获取建议的重试延迟时间（毫秒）
 */
export function getRetryDelay(error: unknown, attempt: number = 1): number {
  if (errorUtils.isPluginError(error)) {
    switch (error.code) {
      case errorCode.http.HTTP_ERROR:
        const status = error.context?.status;
        switch (status) {
          case 429: // Too Many Requests
            return (error.context?.retryAfter || 60) * 1000;
          case 500: // Internal Server Error
            return Math.min(10000 * attempt, 60000); // 10秒 * 尝试次数，最大60秒
          case 502: // Bad Gateway
          case 503: // Service Unavailable
          case 504: // Gateway Timeout
            return Math.min(5000 * attempt, 30000); // 5秒 * 尝试次数，最大30秒
          default:
            return Math.min(3000 * attempt, 15000);
        }

      case errorCode.http.TIMEOUT:
        return Math.min(5000 * attempt, 30000); // 5秒 * 尝试次数，最大30秒

      default:
        return Math.min(3000 * attempt, 15000); // 3秒 * 尝试次数，最大15秒
    }
  }
  return 3000 * attempt;
}

/**
 * 计算指数退避延迟
 */
export function calculateExponentialBackoff(
  baseDelay: number,
  attempt: number,
  maxDelay: number = 30000
): number {
  const delay = baseDelay * Math.pow(2, attempt - 1);
  return Math.min(delay, maxDelay);
}

/**
 * 带重试的操作执行器
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    exponentialBackoff = false,
    maxDelay = 30000,
    shouldRetry = isRetryableError,
    getDelay = (error, attempt) => exponentialBackoff
      ? calculateExponentialBackoff(baseDelay, attempt, maxDelay)
      : getRetryDelay(error, attempt),
    onRetry = (error, attempt, delay) => {
      console.log(`操作失败，${delay}ms 后重试 (${attempt}/${maxRetries})`);
      if (errorUtils.isPluginError(error)) {
        console.log(`错误类型: ${error.code}, 消息: ${error.message}`);
      }
    }
  } = options;

  let lastError: unknown;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;

      // 如果是最后一次尝试或错误不可重试，直接抛出
      if (attempt === maxRetries || !shouldRetry(error)) {
        throw error;
      }

      // 计算延迟时间
      const delay = getDelay(error, attempt);

      // 调用重试回调
      onRetry(error, attempt, delay);

      // 等待延迟时间
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError;
}

/**
 * 创建带重试的函数包装器
 */
export function createRetryWrapper<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options: RetryOptions = {}
): T {
  return ((...args: Parameters<T>) => {
    return withRetry(() => fn(...args), options);
  }) as T;
}

/**
 * 重试工具集合
 */
export const retry = {
  withRetry,
  createRetryWrapper,
  isRetryableError,
  getRetryDelay,
  calculateExponentialBackoff,
};