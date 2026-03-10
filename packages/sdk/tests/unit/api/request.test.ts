import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/utils/error-parser', () => ({
  parseHttpError: vi.fn()
}));

vi.mock('../../../src/types/errors', () => ({
  createError: {
    http: {
      httpError: vi.fn(),
      networkError: vi.fn(),
      timeout: vi.fn()
    },
    common: {
      unknown: vi.fn(),
      permissionDenied: vi.fn(),
      invalidArgument: vi.fn()
    }
  },
  errorUtils: {
    isPluginError: vi.fn()
  }
}));

// Import after mocking
import {
  request,
  get,
  post,
  put,
  patch,
  del,
  http,
  type RequestOptions,
  type Response,
  type HttpMethod,
  type ResponseType
} from '../../../src/api/request';
import { invoke } from '../../../src/core/ipc';
import { parseHttpError } from '../../../src/utils/error-parser';
import { createError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockParseHttpError = vi.mocked(parseHttpError);
const mockCreateError = vi.mocked(createError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Request API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock errorUtils.isPluginError to return false by default
    mockErrorUtils.isPluginError.mockReturnValue(false);
    // Mock createError functions to return proper error objects
    mockCreateError.http.httpError.mockImplementation((status, statusText, context) => {
      const error = new Error(`HTTP ${status}: ${statusText}`) as any;
      error.name = 'PluginError';
      error.code = 'HTTP_HTTP_ERROR';
      error.context = context;
      return error;
    });
    mockCreateError.http.networkError.mockImplementation((message, context) => {
      const error = new Error(message) as any;
      error.name = 'PluginError';
      error.code = 'HTTP_NETWORK_ERROR';
      error.context = context;
      return error;
    });
    mockCreateError.http.timeout.mockImplementation((url, timeout, context) => {
      const error = new Error(`Request to ${url} timed out after ${timeout}ms`) as any;
      error.name = 'PluginError';
      error.code = 'HTTP_TIMEOUT';
      error.context = context;
      return error;
    });
  });

  describe('request', () => {
    it('should make successful GET request', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: { 'content-type': 'application/json' },
        body: { message: 'success' }
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/data',
        method: 'GET'
      };

      const result = await request(options);

      expect(result).toEqual(mockResponse);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_request', options);
    });

    it('should make successful POST request with JSON body', async () => {
      const mockResponse: Response = {
        status: 201,
        statusText: 'Created',
        headers: { 'content-type': 'application/json' },
        body: { id: 123, created: true }
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/users',
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: { name: 'John Doe', email: 'john@example.com' }
      };

      const result = await request(options);

      expect(result).toEqual(mockResponse);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_request', options);
    });

    it('should handle ArrayBuffer response type', async () => {
      const base64Data = btoa('binary data content');
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: { 'content-type': 'application/octet-stream' },
        body: base64Data
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/binary',
        method: 'GET',
        responseType: 'arraybuffer'
      };

      const result = await request(options);

      expect(result.status).toBe(200);
      expect(result.body).toBeInstanceOf(ArrayBuffer);
      
      // Verify the ArrayBuffer content
      const uint8Array = new Uint8Array(result.body as ArrayBuffer);
      const decodedString = String.fromCharCode(...uint8Array);
      expect(decodedString).toBe('binary data content');
    });

    it('should handle different HTTP methods', async () => {
      const methods: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];
      
      for (const method of methods) {
        const mockResponse: Response = {
          status: 200,
          statusText: 'OK',
          headers: {},
          body: `${method} response`
        };
        mockInvoke.mockResolvedValue(mockResponse);

        const options: RequestOptions = {
          url: `https://api.example.com/${method.toLowerCase()}`,
          method
        };

        const result = await request(options);
        expect(result.body).toBe(`${method} response`);
      }
    });

    it('should handle different response types', async () => {
      const responseTypes: ResponseType[] = ['json', 'text', 'arraybuffer'];
      
      for (const responseType of responseTypes) {
        const mockResponse: Response = {
          status: 200,
          statusText: 'OK',
          headers: {},
          body: responseType === 'json' ? { type: responseType } : `${responseType} content`
        };
        mockInvoke.mockResolvedValue(mockResponse);

        const options: RequestOptions = {
          url: 'https://api.example.com/data',
          responseType
        };

        await request(options);
        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', options);
      }
    });

    it('should handle custom headers', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: { 'x-custom-header': 'custom-value' },
        body: 'success'
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/data',
        headers: {
          'Authorization': 'Bearer token123',
          'X-API-Key': 'api-key-456',
          'Content-Type': 'application/json'
        }
      };

      const result = await request(options);

      expect(result).toEqual(mockResponse);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_request', options);
    });

    it('should handle timeout option', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'success'
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/slow',
        timeout: 5000
      };

      await request(options);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_request', options);
    });

    it('should throw HTTP error for 4xx status codes', async () => {
      const mockResponse: Response = {
        status: 404,
        statusText: 'Not Found',
        headers: {},
        body: { error: 'Resource not found' }
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/missing',
        method: 'GET'
      };

      await expect(request(options)).rejects.toThrow('HTTP 404: Not Found');
      expect(mockCreateError.http.httpError).toHaveBeenCalledWith(404, 'Not Found', {
        url: 'https://api.example.com/missing',
        method: 'GET',
        response: mockResponse
      });
    });

    it('should throw HTTP error for 5xx status codes', async () => {
      const mockResponse: Response = {
        status: 500,
        statusText: 'Internal Server Error',
        headers: {},
        body: { error: 'Server error' }
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/error',
        method: 'POST',
        body: { data: 'test' }
      };

      await expect(request(options)).rejects.toThrow('HTTP 500: Internal Server Error');
      expect(mockCreateError.http.httpError).toHaveBeenCalledWith(500, 'Internal Server Error', {
        url: 'https://api.example.com/error',
        method: 'POST',
        response: mockResponse
      });
    });

    it('should handle network errors', async () => {
      const networkError = new Error('Network connection failed');
      const parsedError = mockCreateError.http.networkError('Network connection failed', {
        url: 'https://api.example.com/data',
        method: 'GET'
      });
      
      mockInvoke.mockRejectedValue(networkError);
      mockParseHttpError.mockReturnValue(parsedError);

      const options: RequestOptions = {
        url: 'https://api.example.com/data',
        method: 'GET'
      };

      await expect(request(options)).rejects.toThrow(parsedError);
      expect(mockParseHttpError).toHaveBeenCalledWith(networkError, {
        url: 'https://api.example.com/data',
        method: 'GET',
        options
      });
    });

    it('should not re-parse PluginError instances', async () => {
      const pluginError = mockCreateError.http.timeout('https://api.example.com/slow', 5000, {});
      mockInvoke.mockRejectedValue(pluginError);
      // Mock isPluginError to return true for this specific error
      mockErrorUtils.isPluginError.mockReturnValue(true);

      const options: RequestOptions = {
        url: 'https://api.example.com/slow',
        timeout: 5000
      };

      await expect(request(options)).rejects.toThrow(pluginError);
      expect(mockParseHttpError).not.toHaveBeenCalled();
    });

    it('should handle different body types', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'success'
      };
      mockInvoke.mockResolvedValue(mockResponse);

      // Test string body
      await request({
        url: 'https://api.example.com/string',
        method: 'POST',
        body: 'string data'
      });

      // Test object body
      await request({
        url: 'https://api.example.com/json',
        method: 'POST',
        body: { key: 'value' }
      });

      // Test ArrayBuffer body
      const arrayBuffer = new ArrayBuffer(8);
      await request({
        url: 'https://api.example.com/binary',
        method: 'POST',
        body: arrayBuffer
      });

      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });

  describe('convenience methods', () => {
    beforeEach(() => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'success'
      };
      mockInvoke.mockResolvedValue(mockResponse);
    });

    describe('get', () => {
      it('should make GET request', async () => {
        await get('https://api.example.com/data');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/data',
          method: 'GET'
        });
      });

      it('should make GET request with options', async () => {
        await get('https://api.example.com/data', {
          headers: { 'Authorization': 'Bearer token' },
          timeout: 3000
        });

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/data',
          method: 'GET',
          headers: { 'Authorization': 'Bearer token' },
          timeout: 3000
        });
      });
    });

    describe('post', () => {
      it('should make POST request without body', async () => {
        await post('https://api.example.com/data');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/data',
          method: 'POST',
          body: undefined
        });
      });

      it('should make POST request with body', async () => {
        const body = { name: 'John', age: 30 };
        await post('https://api.example.com/users', body);

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/users',
          method: 'POST',
          body
        });
      });

      it('should make POST request with body and options', async () => {
        const body = { data: 'test' };
        await post('https://api.example.com/data', body, {
          headers: { 'Content-Type': 'application/json' },
          timeout: 5000
        });

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/data',
          method: 'POST',
          body,
          headers: { 'Content-Type': 'application/json' },
          timeout: 5000
        });
      });
    });

    describe('put', () => {
      it('should make PUT request', async () => {
        const body = { id: 1, name: 'Updated' };
        await put('https://api.example.com/users/1', body);

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/users/1',
          method: 'PUT',
          body
        });
      });
    });

    describe('patch', () => {
      it('should make PATCH request', async () => {
        const body = { name: 'Patched' };
        await patch('https://api.example.com/users/1', body);

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/users/1',
          method: 'PATCH',
          body
        });
      });
    });

    describe('del', () => {
      it('should make DELETE request', async () => {
        await del('https://api.example.com/users/1');

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/users/1',
          method: 'DELETE'
        });
      });

      it('should make DELETE request with options', async () => {
        await del('https://api.example.com/users/1', {
          headers: { 'Authorization': 'Bearer token' }
        });

        expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
          url: 'https://api.example.com/users/1',
          method: 'DELETE',
          headers: { 'Authorization': 'Bearer token' }
        });
      });
    });
  });

  describe('http namespace', () => {
    it('should have all expected methods', () => {
      expect(http.request).toBe(request);
      expect(http.get).toBe(get);
      expect(http.post).toBe(post);
      expect(http.put).toBe(put);
      expect(http.patch).toBe(patch);
      expect(http.delete).toBe(del);
    });

    it('should work through namespace methods', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'namespace test'
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const result = await http.get('https://api.example.com/namespace');

      expect(result.body).toBe('namespace test');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_request', {
        url: 'https://api.example.com/namespace',
        method: 'GET'
      });
    });
  });

  describe('error handling edge cases', () => {
    it('should handle malformed ArrayBuffer response', async () => {
      const mockResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'invalid-base64-!@#$%'
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const options: RequestOptions = {
        url: 'https://api.example.com/binary',
        responseType: 'arraybuffer'
      };

      // Should not throw, but handle gracefully
      await expect(request(options)).rejects.toThrow();
    });

    it('should handle empty response body', async () => {
      const mockResponse: Response = {
        status: 204,
        statusText: 'No Content',
        headers: {},
        body: null
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const result = await request({
        url: 'https://api.example.com/empty'
      });

      expect(result.status).toBe(204);
      expect(result.body).toBeNull();
    });

    it('should handle boundary status codes', async () => {
      // Test 200 (success boundary)
      const successResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'success'
      };
      mockInvoke.mockResolvedValue(successResponse);

      let result = await request({ url: 'https://api.example.com/200' });
      expect(result.status).toBe(200);

      // Test 299 (success boundary)
      const success299Response: Response = {
        status: 299,
        statusText: 'Success',
        headers: {},
        body: 'success'
      };
      mockInvoke.mockResolvedValue(success299Response);

      result = await request({ url: 'https://api.example.com/299' });
      expect(result.status).toBe(299);

      // Test 300 (error boundary)
      const error300Response: Response = {
        status: 300,
        statusText: 'Multiple Choices',
        headers: {},
        body: 'error'
      };
      mockInvoke.mockResolvedValue(error300Response);

      await expect(request({ url: 'https://api.example.com/300' })).rejects.toThrow();
    });
  });

  describe('concurrent requests', () => {
    it('should handle multiple simultaneous requests', async () => {
      const responses = [
        { status: 200, statusText: 'OK', headers: {}, body: 'response1' },
        { status: 200, statusText: 'OK', headers: {}, body: 'response2' },
        { status: 200, statusText: 'OK', headers: {}, body: 'response3' }
      ];

      mockInvoke
        .mockResolvedValueOnce(responses[0])
        .mockResolvedValueOnce(responses[1])
        .mockResolvedValueOnce(responses[2]);

      const [result1, result2, result3] = await Promise.all([
        get('https://api.example.com/1'),
        post('https://api.example.com/2', { data: 'test' }),
        put('https://api.example.com/3', { data: 'update' })
      ]);

      expect(result1.body).toBe('response1');
      expect(result2.body).toBe('response2');
      expect(result3.body).toBe('response3');
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const successResponse: Response = {
        status: 200,
        statusText: 'OK',
        headers: {},
        body: 'success'
      };
      const errorResponse: Response = {
        status: 500,
        statusText: 'Internal Server Error',
        headers: {},
        body: 'error'
      };

      mockInvoke
        .mockResolvedValueOnce(successResponse)
        .mockResolvedValueOnce(errorResponse);

      const results = await Promise.allSettled([
        get('https://api.example.com/success'),
        get('https://api.example.com/error')
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect((results[0] as PromiseFulfilledResult<Response>).value.body).toBe('success');
      expect(results[1].status).toBe('rejected');
    });
  });
});