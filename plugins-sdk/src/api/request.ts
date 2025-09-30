import { invoke } from '../core/ipc';
import { createError, errorUtils } from '../types/errors';
import { parseHttpError } from '../utils/error-parser';

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

export async function request<T>(options: RequestOptions): Promise<Response<T>> {
  try {
    const response = await invoke<Response<T>>('plugin_request', options);

    // 处理 ArrayBuffer 响应类型
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
      // 使用改进的HTTP错误创建函数，自动选择精确的错误码
      throw createError.http.httpError(response.status, response.statusText, {
        url: options.url,
        method: options.method,
        response
      });
    }
  } catch (error: any) {
    // 如果已经是 PluginError，直接抛出
    if (errorUtils.isPluginError(error)) {
      throw error;
    }

    // 使用统一的错误解析器
    throw parseHttpError(error, {
      url: options.url,
      method: options.method,
      timeout: options.timeout,
      headers: options.headers
    });
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

// HTTP 客户端对象 - 提供核心功能和便捷方法
export const http = {
  request,
  get,
  post,
  put,
  patch,
  delete: del, // delete 是关键字，用 del
};