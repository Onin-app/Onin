import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';
import { errorUtils, createError } from '../types/errors';
import { parseHttpError } from '../utils/error-parser';

/**
 * HTTP method types
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';

/**
 * Response type options
 */
export type ResponseType = 'json' | 'text' | 'arraybuffer';

/**
 * HTTP request options
 * @interface RequestOptions
 */
export interface RequestOptions {
  /** Target URL for the request */
  url: string;
  /** HTTP request method */
  method?: HttpMethod;
  /** HTTP headers to attach to the request */
  headers?: Record<string, string>;
  /** Request body. For JSON, provide a serializable object */
  body?: string | ArrayBuffer | Record<string, any>;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Expected response body type */
  responseType?: ResponseType;
}

/**
 * HTTP response interface
 * @interface Response
 * @typeParam T - The type of the response body
 * @since 0.1.0
 * @group Types
 */
export interface Response<T = any> {
  /** HTTP status code of the response */
  status: number;
  /** HTTP status text of the response */
  statusText: string;
  /** HTTP headers of the response */
  headers: Record<string, string>;
  /** Response body, type depends on responseType in RequestOptions */
  body: T;
}

/**
 * Makes an HTTP request.
 * This is a generic request function that supports various HTTP methods and options.
 *
 * @typeParam T - The expected response body type
 * @param options - HTTP request configuration options
 * @returns Promise that resolves to a Response object containing the response data
 * @throws {PluginError} With code `HTTP_NETWORK_ERROR` for network connectivity issues
 * @throws {PluginError} With code `HTTP_TIMEOUT` when request exceeds timeout limit
 * @throws {PluginError} With code `HTTP_HTTP_ERROR` for HTTP status errors (4xx, 5xx)
 * @throws {PluginError} With code `PERMISSION_DENIED` when network permission is denied
 * @see {@link ../../HTTP_ERROR_DESIGN_DECISION.md} - Design decisions about HTTP error handling
 * @see {@link ../../ERROR_HANDLING.md} - General guidelines for plugin error handling
 * @example
 * ```typescript
 * // Make a GET request
 * async function fetchData() {
 *   try {
 *     const response = await http.request({
 *       url: 'https://api.example.com/data',
 *       method: 'GET',
 *       responseType: 'json'
 *     });
 *     console.log(response.body);
 *   } catch (error) {
 *     if (errorUtils.isErrorCode(error, 'HTTP_NETWORK_ERROR')) {
 *       console.error('Network connection failed');
 *     } else if (errorUtils.isErrorCode(error, 'HTTP_HTTP_ERROR')) {
 *       console.error('HTTP error:', error.context?.status);
 *     }
 *   }
 * }
 *
 * // Make a POST request
 * async function postData() {
 *   try {
 *     const response = await http.request({
 *       url: 'https://api.example.com/users',
 *       method: 'POST',
 *       body: { name: 'John Doe', email: 'john.doe@example.com' }
 *     });
 *     console.log('User created:', response.body);
 *   } catch (error) {
 *     console.error('Failed to create user:', error.message);
 *   }
 * }
 * ```
 * @since 0.1.0
 * @group API
 */
export async function request<T = any>(options: RequestOptions): Promise<Response<T>> {
  try {
    const response = await invoke<Response<T>>('plugin_request', options);

    // Handle ArrayBuffer response type
    if (options.responseType === 'arraybuffer' && typeof response.body === 'string') {
      const binaryString = atob(response.body);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      (response as any).body = bytes.buffer;
    }

    if (response.status >= 200 && response.status < 300) {
      return response;
    } else {
      // Use improved HTTP error creation function, automatically selects precise error code
      throw createError.http.httpError(response.status, response.statusText, {
        url: options.url,
        method: options.method || 'GET',
        response: response
      });
    }
  } catch (error: any) {
    // If already a PluginError, throw directly
    if (errorUtils.isPluginError(error)) {
      throw error;
    }

    // Use unified error parser
    throw parseHttpError(error, {
      url: options.url,
      method: options.method || 'GET',
      options
    });
  }
}

/**
 * Makes an HTTP GET request.
 * @typeParam T - The expected response body type
 * @param url - Target URL for the request
 * @param options - Other request options
 * @returns Promise that resolves to the response
 * @throws {PluginError} Same error conditions as {@link request}
 * @see {@link request} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export async function get<T>(url: string, options?: Omit<RequestOptions, 'url' | 'method'>): Promise<Response<T>> {
  return request<T>({ ...options, url, method: 'GET' });
}

/**
 * Makes an HTTP POST request.
 * @typeParam T - The expected response body type
 * @param url - Target URL for the request
 * @param body - Request body
 * @param options - Other request options
 * @returns Promise that resolves to the response
 * @throws {PluginError} Same error conditions as {@link request}
 * @see {@link request} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export async function post<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request<T>({ ...options, url, method: 'POST', body });
}

/**
 * Makes an HTTP PUT request.
 * @typeParam T - The expected response body type
 * @param url - Target URL for the request
 * @param body - Request body
 * @param options - Other request options
 * @returns Promise that resolves to the response
 * @throws {PluginError} Same error conditions as {@link request}
 * @see {@link request} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export async function put<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request<T>({ ...options, url, method: 'PUT', body });
}

/**
 * Makes an HTTP PATCH request.
 * @typeParam T - The expected response body type
 * @param url - Target URL for the request
 * @param body - Request body
 * @param options - Other request options
 * @returns Promise that resolves to the response
 * @throws {PluginError} Same error conditions as {@link request}
 * @see {@link request} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export async function patch<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request<T>({ ...options, url, method: 'PATCH', body });
}

/**
 * Makes an HTTP DELETE request.
 * @typeParam T - The expected response body type
 * @param url - Target URL for the request
 * @param options - Other request options
 * @returns Promise that resolves to the response
 * @throws {PluginError} Same error conditions as {@link request}
 * @see {@link request} - For detailed error information
 * @since 0.1.0
 * @group API
 */
export async function del<T>(url: string, options?: Omit<RequestOptions, 'url' | 'method'>): Promise<Response<T>> {
  return request<T>({ ...options, url, method: 'DELETE' });
}

/**
 * HTTP API namespace - provides functions for making HTTP requests
 * 
 * Supports all standard HTTP methods (GET, POST, PUT, PATCH, DELETE) with comprehensive
 * error handling and response type management. All requests return structured Response objects
 * and throw detailed PluginError instances on failure.
 * 
 * @namespace http
 * @version 0.1.0
 * @since 0.1.0
 * @group API
 * @see {@link parseHttpError} - For HTTP error parsing utilities
 * @see {@link createError.http} - For HTTP error creation functions
 * @example
 * ```typescript
 * import { http } from 'baize-plugin-sdk';
 *
 * // Simple GET request
 * const { body } = await http.get('https://api.example.com/posts/1');
 * console.log(body.title);
 * 
 * // POST with error handling
 * try {
 *   const response = await http.post('https://api.example.com/users', {
 *     name: 'John',
 *     email: 'john@example.com'
 *   });
 *   console.log('Created user:', response.body);
 * } catch (error) {
 *   if (errorUtils.isErrorCode(error, 'HTTP_HTTP_ERROR')) {
 *     console.error('HTTP Error:', error.context?.status);
 *   }
 * }
 * ```
 */
export const http = {
  request,
  get,
  post,
  put,
  patch,
  delete: del, // delete is a keyword, use del
};