import { errorCode, errorUtils } from '../types/errors';

/**
 * Retry configuration options
 * @interface RetryOptions
 */
export interface RetryOptions {
  /** Maximum number of retry attempts */
  maxRetries?: number;
  /** Base delay time in milliseconds */
  baseDelay?: number;
  /** Whether to use exponential backoff */
  exponentialBackoff?: boolean;
  /** Maximum delay time in milliseconds */
  maxDelay?: number;
  /** Custom retry condition function */
  shouldRetry?: (error: unknown) => boolean;
  /** Custom delay calculation function */
  getDelay?: (error: unknown, attempt: number) => number;
  /** Callback function called before retry */
  onRetry?: (error: unknown, attempt: number, delay: number) => void;
}

/**
 * Checks if an error is retryable
 * @param error - The error to check
 * @returns True if the error can be retried
 */
export function isRetryableError(error: unknown): boolean {
  if (errorUtils.isPluginError(error)) {
    switch (error.code) {
      case errorCode.http.NETWORK_ERROR:
      case errorCode.http.TIMEOUT:
        return true;

      case errorCode.http.HTTP_ERROR:
        // Based on standard HTTP status codes to determine if retryable
        const status = error.context?.status;
        return (
          status === 429 || // Too Many Requests
          status === 500 || // Internal Server Error
          status === 502 || // Bad Gateway
          status === 503 || // Service Unavailable
          status === 504
        ); // Gateway Timeout

      case errorCode.clipboard.UNAVAILABLE:
        return true;

      default:
        return false;
    }
  }
  return false;
}

/**
 * Gets the suggested retry delay time in milliseconds
 * @param error - The error that occurred
 * @param attempt - The current attempt number (1-based)
 * @returns Delay time in milliseconds
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
            return Math.min(10000 * attempt, 60000); // 10 seconds * attempt number, max 60 seconds
          case 502: // Bad Gateway
          case 503: // Service Unavailable
          case 504: // Gateway Timeout
            return Math.min(5000 * attempt, 30000); // 5 seconds * attempt number, max 30 seconds
          default:
            return Math.min(3000 * attempt, 15000);
        }

      case errorCode.http.TIMEOUT:
        return Math.min(5000 * attempt, 30000); // 5 seconds * attempt number, max 30 seconds

      default:
        return Math.min(3000 * attempt, 15000); // 3 seconds * attempt number, max 15 seconds
    }
  }
  return 3000 * attempt;
}

/**
 * Calculates exponential backoff delay
 * @param baseDelay - Base delay in milliseconds
 * @param attempt - Current attempt number (1-based)
 * @param maxDelay - Maximum delay in milliseconds
 * @returns Calculated delay in milliseconds
 */
export function calculateExponentialBackoff(
  baseDelay: number,
  attempt: number,
  maxDelay: number = 30000,
): number {
  const delay = baseDelay * Math.pow(2, attempt - 1);
  return Math.min(delay, maxDelay);
}

/**
 * Operation executor with retry capability
 * @typeParam T - The return type of the operation
 * @param operation - The async operation to execute
 * @param options - Retry configuration options
 * @returns Promise that resolves to the operation result
 * @throws The last error if all retry attempts fail
 * @since 0.1.0
 * @group Utilities
 */
export async function withRetry<T>(
  operation: () => Promise<T>,
  options: RetryOptions = {},
): Promise<T> {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    exponentialBackoff = false,
    maxDelay = 30000,
    shouldRetry = isRetryableError,
    getDelay = (error, attempt) =>
      exponentialBackoff
        ? calculateExponentialBackoff(baseDelay, attempt, maxDelay)
        : getRetryDelay(error, attempt),
    onRetry = (error, attempt, delay) => {
      console.log(
        `Operation failed, retrying in ${delay}ms (${attempt}/${maxRetries})`,
      );
      if (errorUtils.isPluginError(error)) {
        console.log(`Error type: ${error.code}, message: ${error.message}`);
      }
    },
  } = options;

  let lastError: unknown;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;

      // If this is the last attempt or error is not retryable, throw immediately
      if (attempt === maxRetries || !shouldRetry(error)) {
        throw error;
      }

      // Calculate delay time
      const delay = getDelay(error, attempt);

      // Call retry callback
      onRetry(error, attempt, delay);

      // Wait for delay time
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }

  throw lastError;
}

/**
 * Creates a function wrapper with retry capability
 * @typeParam T - The function type to wrap
 * @param fn - The function to wrap with retry logic
 * @param options - Retry configuration options
 * @returns A new function with retry capability
 * @since 0.1.0
 * @group Utilities
 */
export function createRetryWrapper<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options: RetryOptions = {},
): T {
  return ((...args: Parameters<T>) => {
    return withRetry(() => fn(...args), options);
  }) as T;
}

/**
 * Retry utilities collection - provides retry mechanisms for error-prone operations
 *
 * Offers configurable retry logic with exponential backoff, custom error filtering,
 * and callback hooks for monitoring retry attempts. Particularly useful for network
 * operations and other potentially unreliable operations.
 *
 * **Features:**
 * - Configurable retry count and delay strategies
 * - Exponential backoff with jitter support
 * - Custom error filtering to determine retry eligibility
 * - Progress callbacks for monitoring retry attempts
 * - Function wrapper utilities for easy integration
 *
 * @namespace retry
 * @version 0.1.0
 * @since 0.1.0
 * @group Utilities
 * @example
 * ```typescript
 * import { retry, http } from 'onin-sdk';
 *
 * // Basic retry with default options
 * const data = await retry.withRetry(async () => {
 *   return await http.get('https://api.example.com/data');
 * });
 *
 * // Advanced retry with custom options
 * const result = await retry.withRetry(
 *   async () => await someUnreliableOperation(),
 *   {
 *     maxRetries: 5,
 *     exponentialBackoff: true,
 *     baseDelay: 2000,
 *     maxDelay: 30000,
 *     shouldRetry: (error) => {
 *       // Only retry on network errors
 *       return errorUtils.isErrorCode(error, 'HTTP_NETWORK_ERROR');
 *     },
 *     onRetry: (error, attempt, delay) => {
 *       console.log(`Retry attempt ${attempt}, waiting ${delay}ms`);
 *     }
 *   }
 * );
 *
 * // Create reusable retry wrapper
 * const reliableHttpGet = retry.createRetryWrapper(
 *   (url: string) => http.get(url),
 *   { maxRetries: 3, exponentialBackoff: true }
 * );
 *
 * const response = await reliableHttpGet('https://api.example.com/data');
 * ```
 */
export const retry = {
  withRetry,
  createRetryWrapper,
  isRetryableError,
  getRetryDelay,
  calculateExponentialBackoff,
};
