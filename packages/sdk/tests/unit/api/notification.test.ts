import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

// Import after mocking
import { showNotification, show, notification } from '../../../src/api/notification';
import type { NotificationOptions } from '../../../src/api/notification';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);

describe('Notification API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock dispatch to call the webview handler by default
    mockDispatch.mockImplementation(({ webview }) => webview());
  });

  describe('showNotification', () => {
    it('should call dispatch with correct handlers', async () => {
      const options: NotificationOptions = {
        title: 'Test Title',
        body: 'Test Body'
      };

      mockInvoke.mockResolvedValue(undefined);

      await showNotification(options);

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
    });

    it('should call invoke with correct parameters in webview environment', async () => {
      const options: NotificationOptions = {
        title: 'Test Title',
        body: 'Test Body'
      };

      mockInvoke.mockResolvedValue(undefined);
      mockDispatch.mockImplementation(({ webview }) => webview());

      await showNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options });
    });

    it('should call invoke with correct parameters in headless environment', async () => {
      const options: NotificationOptions = {
        title: 'Test Title',
        body: 'Test Body'
      };

      mockInvoke.mockResolvedValue(undefined);
      mockDispatch.mockImplementation(({ headless }) => headless());

      await showNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith('show_notification', options);
    });

    it('should handle successful notification display', async () => {
      const options: NotificationOptions = {
        title: 'Success',
        body: 'Operation completed successfully'
      };

      mockInvoke.mockResolvedValue(undefined);

      await expect(showNotification(options)).resolves.toBeUndefined();
    });

    it('should handle errors from invoke', async () => {
      const options: NotificationOptions = {
        title: 'Error Test',
        body: 'This should fail'
      };

      const error = new Error('Permission denied');
      mockInvoke.mockRejectedValue(error);

      await expect(showNotification(options)).rejects.toThrow('Permission denied');
    });

    it('should validate required title field', async () => {
      const invalidOptions = {
        body: 'Missing title'
      } as NotificationOptions;

      // TypeScript should catch this, but let's test runtime behavior
      mockInvoke.mockResolvedValue(undefined);

      await showNotification(invalidOptions);
      
      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options: invalidOptions });
    });

    it('should validate required body field', async () => {
      const invalidOptions = {
        title: 'Missing body'
      } as NotificationOptions;

      mockInvoke.mockResolvedValue(undefined);

      await showNotification(invalidOptions);
      
      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options: invalidOptions });
    });

    it('should handle empty strings', async () => {
      const options: NotificationOptions = {
        title: '',
        body: ''
      };

      mockInvoke.mockResolvedValue(undefined);

      await expect(showNotification(options)).resolves.toBeUndefined();
      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options });
    });

    it('should handle special characters in title and body', async () => {
      const options: NotificationOptions = {
        title: 'Test 🎉 Title with émojis & spëcial chars',
        body: 'Body with\nnewlines and\ttabs'
      };

      mockInvoke.mockResolvedValue(undefined);

      await expect(showNotification(options)).resolves.toBeUndefined();
      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options });
    });
  });

  describe('show alias', () => {
    it('should be the same function as showNotification', () => {
      expect(show).toBe(showNotification);
    });

    it('should work identically to showNotification', async () => {
      const options: NotificationOptions = {
        title: 'Alias Test',
        body: 'Testing the show alias'
      };

      mockInvoke.mockResolvedValue(undefined);

      await show(options);

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
    });
  });

  describe('notification object', () => {
    it('should have show method', () => {
      expect(notification.show).toBe(showNotification);
    });

    it('should work through notification.show', async () => {
      const options: NotificationOptions = {
        title: 'Object Test',
        body: 'Testing the notification object'
      };

      mockInvoke.mockResolvedValue(undefined);

      await notification.show(options);

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
    });
  });

  describe('error scenarios', () => {
    it('should handle network errors', async () => {
      const options: NotificationOptions = {
        title: 'Network Error',
        body: 'Testing network failure'
      };

      const networkError = new Error('Network unavailable');
      mockInvoke.mockRejectedValue(networkError);

      await expect(showNotification(options)).rejects.toThrow('Network unavailable');
    });

    it('should handle permission errors', async () => {
      const options: NotificationOptions = {
        title: 'Permission Error',
        body: 'Testing permission denial'
      };

      const permissionError = new Error('Notification permission denied');
      mockInvoke.mockRejectedValue(permissionError);

      await expect(showNotification(options)).rejects.toThrow('Notification permission denied');
    });

    it('should handle timeout errors', async () => {
      const options: NotificationOptions = {
        title: 'Timeout Error',
        body: 'Testing timeout'
      };

      const timeoutError = new Error('Request timeout');
      mockInvoke.mockRejectedValue(timeoutError);

      await expect(showNotification(options)).rejects.toThrow('Request timeout');
    });
  });

  describe('concurrent notifications', () => {
    it('should handle multiple simultaneous notifications', async () => {
      const notifications = [
        { title: 'Notification 1', body: 'First notification' },
        { title: 'Notification 2', body: 'Second notification' },
        { title: 'Notification 3', body: 'Third notification' }
      ];

      mockInvoke.mockResolvedValue(undefined);

      const promises = notifications.map(options => showNotification(options));
      await Promise.all(promises);

      expect(mockInvoke).toHaveBeenCalledTimes(3);
      expect(mockDispatch).toHaveBeenCalledTimes(3);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const options1: NotificationOptions = { title: 'Success', body: 'This will succeed' };
      const options2: NotificationOptions = { title: 'Failure', body: 'This will fail' };

      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockRejectedValueOnce(new Error('Failed'));

      const results = await Promise.allSettled([
        showNotification(options1),
        showNotification(options2)
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect(results[1].status).toBe('rejected');
      expect((results[1] as PromiseRejectedResult).reason.message).toBe('Failed');
    });
  });
});