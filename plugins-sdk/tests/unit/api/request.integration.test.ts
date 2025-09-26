import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the modules using factory functions
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
      permissionDenied: vi.fn()
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
  type Response,
  type RequestOptions 
} from '../../../src/api/request';
import { invoke } from '../../../src/core/ipc';
import { parseHttpError } from '../../../src/utils/error-parser';
import { createError, errorUtils } from '../../../src/types/errors';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockParseHttpError = vi.mocked(parseHttpError);
const mockCreateError = vi.mocked(createError);
const mockErrorUtils = vi.mocked(errorUtils);

describe('Request API Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockErrorUtils.isPluginError.mockReturnValue(false);
    
    // Mock createError functions
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

  it('should handle complete REST API workflow', async () => {
    // Simulate a complete CRUD workflow
    const userCreateResponse: Response = {
      status: 201,
      statusText: 'Created',
      headers: { 'content-type': 'application/json' },
      body: { id: 123, name: 'John Doe', email: 'john@example.com' }
    };

    const userGetResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: { id: 123, name: 'John Doe', email: 'john@example.com', created: '2023-01-01' }
    };

    const userUpdateResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: { id: 123, name: 'John Smith', email: 'john.smith@example.com', updated: '2023-01-02' }
    };

    const userDeleteResponse: Response = {
      status: 204,
      statusText: 'No Content',
      headers: {},
      body: null
    };

    mockInvoke
      .mockResolvedValueOnce(userCreateResponse) // POST create user
      .mockResolvedValueOnce(userGetResponse) // GET user details
      .mockResolvedValueOnce(userUpdateResponse) // PUT update user
      .mockResolvedValueOnce(userDeleteResponse); // DELETE user

    // Create user
    const createResult = await http.post('https://api.example.com/users', {
      name: 'John Doe',
      email: 'john@example.com'
    });
    expect(createResult.status).toBe(201);
    expect(createResult.body.id).toBe(123);

    // Get user details
    const getResult = await http.get(`https://api.example.com/users/${createResult.body.id}`);
    expect(getResult.status).toBe(200);
    expect(getResult.body.name).toBe('John Doe');

    // Update user
    const updateResult = await http.put(`https://api.example.com/users/${createResult.body.id}`, {
      name: 'John Smith',
      email: 'john.smith@example.com'
    });
    expect(updateResult.status).toBe(200);
    expect(updateResult.body.name).toBe('John Smith');

    // Delete user
    const deleteResult = await http.delete(`https://api.example.com/users/${createResult.body.id}`);
    expect(deleteResult.status).toBe(204);
    expect(deleteResult.body).toBeNull();

    expect(mockInvoke).toHaveBeenCalledTimes(4);
  });

  it('should handle authentication workflow', async () => {
    const loginResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: { 
        token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...',
        expires: '2023-12-31T23:59:59Z',
        user: { id: 1, username: 'testuser' }
      }
    };

    const protectedResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: { 
        data: 'sensitive information',
        timestamp: '2023-01-01T12:00:00Z'
      }
    };

    mockInvoke
      .mockResolvedValueOnce(loginResponse)
      .mockResolvedValueOnce(protectedResponse);

    // Login to get token
    const loginResult = await post('https://api.example.com/auth/login', {
      username: 'testuser',
      password: 'password123'
    });

    expect(loginResult.status).toBe(200);
    const token = loginResult.body.token;

    // Use token to access protected resource
    const protectedResult = await get('https://api.example.com/protected', {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    });

    expect(protectedResult.status).toBe(200);
    expect(protectedResult.body.data).toBe('sensitive information');

    expect(mockInvoke).toHaveBeenCalledTimes(2);
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'plugin_request', {
      url: 'https://api.example.com/protected',
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      }
    });
  });

  it('should handle file upload workflow', async () => {
    const uploadResponse: Response = {
      status: 201,
      statusText: 'Created',
      headers: { 'content-type': 'application/json' },
      body: {
        fileId: 'file_123',
        filename: 'document.pdf',
        size: 1024000,
        url: 'https://cdn.example.com/files/file_123.pdf'
      }
    };

    const statusResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: {
        fileId: 'file_123',
        status: 'processed',
        downloadUrl: 'https://cdn.example.com/processed/file_123.pdf'
      }
    };

    mockInvoke
      .mockResolvedValueOnce(uploadResponse)
      .mockResolvedValueOnce(statusResponse);

    // Simulate file upload with binary data
    const fileData = new ArrayBuffer(1024);
    const uploadResult = await post('https://api.example.com/files/upload', fileData, {
      headers: {
        'Content-Type': 'application/octet-stream',
        'X-Filename': 'document.pdf'
      },
      timeout: 30000
    });

    expect(uploadResult.status).toBe(201);
    expect(uploadResult.body.fileId).toBe('file_123');

    // Check upload status
    const statusResult = await get(`https://api.example.com/files/${uploadResult.body.fileId}/status`);
    expect(statusResult.status).toBe(200);
    expect(statusResult.body.status).toBe('processed');

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it('should handle binary data download workflow', async () => {
    const base64Data = btoa('PDF file content here...');
    const downloadResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 
        'content-type': 'application/pdf',
        'content-length': '1024',
        'content-disposition': 'attachment; filename="document.pdf"'
      },
      body: base64Data
    };

    mockInvoke.mockResolvedValue(downloadResponse);

    const result = await get('https://api.example.com/files/download/123', {
      responseType: 'arraybuffer',
      timeout: 60000
    });

    expect(result.status).toBe(200);
    expect(result.body).toBeInstanceOf(ArrayBuffer);
    expect(result.headers['content-type']).toBe('application/pdf');

    // Verify the binary content
    const uint8Array = new Uint8Array(result.body as ArrayBuffer);
    const decodedString = String.fromCharCode(...uint8Array);
    expect(decodedString).toBe('PDF file content here...');
  });

  it('should handle API pagination workflow', async () => {
    const page1Response: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: {
        data: [
          { id: 1, name: 'Item 1' },
          { id: 2, name: 'Item 2' }
        ],
        pagination: {
          page: 1,
          limit: 2,
          total: 5,
          hasNext: true,
          nextUrl: 'https://api.example.com/items?page=2&limit=2'
        }
      }
    };

    const page2Response: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'application/json' },
      body: {
        data: [
          { id: 3, name: 'Item 3' },
          { id: 4, name: 'Item 4' }
        ],
        pagination: {
          page: 2,
          limit: 2,
          total: 5,
          hasNext: true,
          nextUrl: 'https://api.example.com/items?page=3&limit=2'
        }
      }
    };

    mockInvoke
      .mockResolvedValueOnce(page1Response)
      .mockResolvedValueOnce(page2Response);

    // Get first page
    const page1 = await get('https://api.example.com/items?page=1&limit=2');
    expect(page1.body.data).toHaveLength(2);
    expect(page1.body.pagination.hasNext).toBe(true);

    // Get next page using the provided URL
    const page2 = await get(page1.body.pagination.nextUrl);
    expect(page2.body.data).toHaveLength(2);
    expect(page2.body.data[0].id).toBe(3);

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it('should handle error recovery and retry scenarios', async () => {
    const networkError = new Error('Network connection failed');
    const parsedError = mockCreateError.http.networkError('Network connection failed', {
      url: 'https://api.example.com/unstable',
      method: 'GET'
    });

    const successResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: {},
      body: { message: 'success after retry' }
    };

    mockInvoke
      .mockRejectedValueOnce(networkError) // First attempt fails
      .mockResolvedValueOnce(successResponse); // Second attempt succeeds

    mockParseHttpError.mockReturnValue(parsedError);

    // First request fails
    await expect(get('https://api.example.com/unstable')).rejects.toThrow(parsedError);

    // Second request succeeds (simulating retry)
    const result = await get('https://api.example.com/stable');
    expect(result.body.message).toBe('success after retry');

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it('should handle different HTTP status code scenarios', async () => {
    const responses = [
      { status: 200, statusText: 'OK', headers: {}, body: 'success' },
      { status: 201, statusText: 'Created', headers: {}, body: { id: 1 } },
      { status: 204, statusText: 'No Content', headers: {}, body: null },
      { status: 400, statusText: 'Bad Request', headers: {}, body: { error: 'Invalid input' } },
      { status: 401, statusText: 'Unauthorized', headers: {}, body: { error: 'Token expired' } },
      { status: 404, statusText: 'Not Found', headers: {}, body: { error: 'Resource not found' } },
      { status: 500, statusText: 'Internal Server Error', headers: {}, body: { error: 'Server error' } }
    ];

    for (const response of responses) {
      mockInvoke.mockResolvedValueOnce(response);

      if (response.status >= 200 && response.status < 300) {
        // Success cases
        const result = await get(`https://api.example.com/status/${response.status}`);
        expect(result.status).toBe(response.status);
      } else {
        // Error cases
        await expect(get(`https://api.example.com/status/${response.status}`)).rejects.toThrow();
      }
    }

    expect(mockInvoke).toHaveBeenCalledTimes(responses.length);
  });

  it('should handle concurrent API calls with different patterns', async () => {
    const responses = [
      { status: 200, statusText: 'OK', headers: {}, body: { type: 'user', data: 'user data' } },
      { status: 200, statusText: 'OK', headers: {}, body: { type: 'posts', data: ['post1', 'post2'] } },
      { status: 200, statusText: 'OK', headers: {}, body: { type: 'comments', data: ['comment1'] } },
      { status: 404, statusText: 'Not Found', headers: {}, body: { error: 'Profile not found' } }
    ];

    mockInvoke
      .mockResolvedValueOnce(responses[0])
      .mockResolvedValueOnce(responses[1])
      .mockResolvedValueOnce(responses[2])
      .mockResolvedValueOnce(responses[3]);

    const results = await Promise.allSettled([
      get('https://api.example.com/user/123'),
      get('https://api.example.com/user/123/posts'),
      get('https://api.example.com/user/123/comments'),
      get('https://api.example.com/user/123/profile') // This will fail
    ]);

    expect(results[0].status).toBe('fulfilled');
    expect(results[1].status).toBe('fulfilled');
    expect(results[2].status).toBe('fulfilled');
    expect(results[3].status).toBe('rejected');

    const userResult = (results[0] as PromiseFulfilledResult<Response>).value;
    const postsResult = (results[1] as PromiseFulfilledResult<Response>).value;
    const commentsResult = (results[2] as PromiseFulfilledResult<Response>).value;

    expect(userResult.body.type).toBe('user');
    expect(postsResult.body.data).toHaveLength(2);
    expect(commentsResult.body.data).toHaveLength(1);
  });

  it('should handle complex request configurations', async () => {
    const complexResponse: Response = {
      status: 200,
      statusText: 'OK',
      headers: { 
        'x-rate-limit-remaining': '99',
        'x-response-time': '150ms'
      },
      body: { success: true, processed: true }
    };

    mockInvoke.mockResolvedValue(complexResponse);

    const complexOptions: RequestOptions = {
      url: 'https://api.example.com/complex',
      method: 'POST',
      headers: {
        'Authorization': 'Bearer complex-token',
        'Content-Type': 'application/json',
        'X-API-Version': '2.0',
        'X-Client-ID': 'test-client',
        'User-Agent': 'TestApp/1.0'
      },
      body: {
        operation: 'complex-operation',
        parameters: {
          mode: 'advanced',
          options: ['option1', 'option2'],
          metadata: {
            source: 'integration-test',
            timestamp: new Date().toISOString()
          }
        }
      },
      timeout: 15000,
      responseType: 'json'
    };

    const result = await request(complexOptions);

    expect(result.status).toBe(200);
    expect(result.body.success).toBe(true);
    expect(result.headers['x-rate-limit-remaining']).toBe('99');

    expect(mockInvoke).toHaveBeenCalledWith('plugin_request', complexOptions);
  });

  it('should handle timeout scenarios', async () => {
    const timeoutError = new Error('Request timeout');
    const parsedTimeoutError = mockCreateError.http.timeout(
      'https://api.example.com/slow',
      10000,
      { originalError: timeoutError }
    );

    mockInvoke.mockRejectedValue(timeoutError);
    mockParseHttpError.mockReturnValue(parsedTimeoutError);

    await expect(get('https://api.example.com/slow', { timeout: 10000 })).rejects.toThrow(parsedTimeoutError);

    expect(mockParseHttpError).toHaveBeenCalledWith(timeoutError, {
      url: 'https://api.example.com/slow',
      method: 'GET',
      timeout: 10000,
      headers: undefined
    });
  });
});