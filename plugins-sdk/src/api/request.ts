import { invoke } from '../core/ipc';

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';

export type ResponseType = 'json' | 'text' | 'arraybuffer';

export interface RequestOptions {
  url: string;
  method?: HttpMethod;
  headers?: Record<string, string>;
  body?: string | ArrayBuffer | Record<string, any>;
  timeout?: number;
  responseType?: ResponseType;
}

export interface Response<T = any> {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: T;
}

// 错误类型定义
export interface BaizeRequestError extends Error {
  name: 'BaizeRequestError';
}

export interface PermissionDeniedError extends Error {
  name: 'PermissionDeniedError';
  url: string;
}

export interface TimeoutError extends Error {
  name: 'TimeoutError';
  url: string;
  timeout: number;
}

export interface NetworkError extends Error {
  name: 'NetworkError';
}

export interface HttpError extends Error {
  name: 'HttpError';
  response: Response;
}

// 错误工厂函数
export function createBaizeRequestError(message: string): BaizeRequestError {
  const error = new Error(message) as BaizeRequestError;
  error.name = 'BaizeRequestError';
  return error;
}

export function createPermissionDeniedError(url: string, message?: string): PermissionDeniedError {
  const error = new Error(
    message || `Permission denied for URL: ${url}. Please add it to the 'permissions.http.allowUrls' array in your manifest.json.`
  ) as PermissionDeniedError;
  error.name = 'PermissionDeniedError';
  error.url = url;
  return error;
}

export function createTimeoutError(url: string, timeout: number, message?: string): TimeoutError {
  const error = new Error(
    message || `Request to ${url} timed out after ${timeout}ms`
  ) as TimeoutError;
  error.name = 'TimeoutError';
  error.url = url;
  error.timeout = timeout;
  return error;
}

export function createNetworkError(message: string): NetworkError {
  const error = new Error(message) as NetworkError;
  error.name = 'NetworkError';
  return error;
}

export function createHttpError(response: Response): HttpError {
  const error = new Error(
    `Request failed with status ${response.status} ${response.statusText}`
  ) as HttpError;
  error.name = 'HttpError';
  error.response = response;
  return error;
}

// 类型检查辅助函数
export function isBaizeRequestError(error: any): error is BaizeRequestError {
  return error && error.name === 'BaizeRequestError';
}

export function isPermissionDeniedError(error: any): error is PermissionDeniedError {
  return error && error.name === 'PermissionDeniedError';
}

export function isTimeoutError(error: any): error is TimeoutError {
  return error && error.name === 'TimeoutError';
}

export function isNetworkError(error: any): error is NetworkError {
  return error && error.name === 'NetworkError';
}

export function isHttpError(error: any): error is HttpError {
  return error && error.name === 'HttpError';
}

export async function request<T>(options: RequestOptions): Promise<Response<T>> {
  try {
    // 直接传递 options，就像 showNotification 一样
    const response = await invoke<Response<T>>('plugin_request', options);

    // 处理 ArrayBuffer 响应类型
    if (options.responseType === 'arraybuffer' && typeof response.body === 'string') {
      // 从 base64 解码为 ArrayBuffer
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
      throw createHttpError(response);
    }
  } catch (error: any) {
    // 简化错误处理
    const message = typeof error === 'string' ? error : error.message || 'An unknown error occurred';

    if (message.includes('timed out')) {
      throw createTimeoutError(options.url, options.timeout || 30000, message);
    } else if (message.includes('Network error')) {
      throw createNetworkError(message);
    } else if (message.includes('Permission denied')) {
      throw createPermissionDeniedError(options.url, message);
    } else {
      throw createBaizeRequestError(message);
    }
  }
}

// 便捷方法
export async function get<T>(url: string, options?: Omit<RequestOptions, 'url' | 'method'>): Promise<Response<T>> {
  return request({ ...options, url, method: 'GET' });
}

export async function post<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request({ ...options, url, method: 'POST', body });
}

export async function put<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request({ ...options, url, method: 'PUT', body });
}

export async function patch<T>(url: string, body?: any, options?: Omit<RequestOptions, 'url' | 'method' | 'body'>): Promise<Response<T>> {
  return request({ ...options, url, method: 'PATCH', body });
}

export async function del<T>(url: string, options?: Omit<RequestOptions, 'url' | 'method'>): Promise<Response<T>> {
  return request({ ...options, url, method: 'DELETE' });
}

// 错误处理工具集合
export const errors = {
  createBaizeRequestError,
  createPermissionDeniedError,
  createTimeoutError,
  createNetworkError,
  createHttpError,
  isBaizeRequestError,
  isPermissionDeniedError,
  isTimeoutError,
  isNetworkError,
  isHttpError,
};

// HTTP 客户端对象 - 提供核心功能和便捷方法
export const http = {
  request,
  get,
  post,
  put,
  patch,
  delete: del, // delete 是关键字，用 del
  errors,
};