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

export interface RequestError extends Error {
  name: 'RequestError';
}

export interface TimeoutError extends Error {
  name: 'TimeoutError';
}

export interface HttpError extends Error {
  name: 'HttpError';
  response: Response;
}

export function createRequestError(message: string): RequestError {
  const error = new Error(message);
  (error as RequestError).name = 'RequestError';
  return error as RequestError;
}

export function createTimeoutError(message = 'Request timed out'): TimeoutError {
  const error = new Error(message);
  (error as TimeoutError).name = 'TimeoutError';
  return error as TimeoutError;
}

export function createHttpError(response: Response): HttpError {
  const error = new Error(`Request failed with status ${response.status}`);
  (error as HttpError).name = 'HttpError';
  (error as HttpError).response = response;
  return error as HttpError;
}

export async function request<T>(options: RequestOptions): Promise<Response<T>> {
  try {
    const response = await invoke<Response<T>>('plugin_request', { options });
    if (response.status >= 200 && response.status < 300) {
      return response;
    } else {
      throw createHttpError(response);
    }
  } catch (error: any) {
    if (error && typeof error === 'object' && 'response' in error) {
      throw error;
    }
    if (typeof error === 'string' && error.includes('timeout')) {
      throw createTimeoutError();
    }
    throw createRequestError(typeof error === 'string' ? error : error.message || 'An unknown error occurred');
  }
}