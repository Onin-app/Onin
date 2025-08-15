/**
 * Tests for the communication layer
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PluginError, PluginErrorCode } from './types';

// Mock Tauri's invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Import after mocking
const { CommunicationBridge, createCommunicationBridge, setGlobalBridge, getGlobalBridge, invokeApi } = await import('./communication');

describe('CommunicationBridge', () => {
  let bridge: CommunicationBridge;
  let mockInvoke: any;
  const testPluginId = 'test-plugin';

  beforeEach(async () => {
    // Get the mocked invoke function
    const { invoke } = await import('@tauri-apps/api/core');
    mockInvoke = invoke as any;
    
    bridge = new CommunicationBridge(testPluginId, { debug: false });
    mockInvoke.mockClear();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('should create a bridge with default config', () => {
      const bridge = new CommunicationBridge('test');
      expect(bridge.getPluginId()).toBe('test');
    });

    it('should create a bridge with custom config', () => {
      const config = { maxRetries: 5, timeout: 10000 };
      const bridge = new CommunicationBridge('test', config);
      expect(bridge.getPluginId()).toBe('test');
    });
  });

  describe('invoke', () => {
    it('should make successful API call', async () => {
      const expectedResult = { data: 'test' };
      mockInvoke.mockResolvedValueOnce({ success: true, data: expectedResult });

      const result = await bridge.invoke('test_command', { param: 'value' });

      expect(mockInvoke).toHaveBeenCalledWith('plugin_test_command', {
        param: 'value',
        plugin_id: testPluginId
      });
      expect(result).toEqual(expectedResult);
    });

    it('should handle direct result format', async () => {
      const expectedResult = 'direct result';
      mockInvoke.mockResolvedValueOnce(expectedResult);

      const result = await bridge.invoke('test_command');

      expect(result).toBe(expectedResult);
    });

    it('should handle API error response', async () => {
      const bridge = new CommunicationBridge(testPluginId, { maxRetries: 0 }); // No retries for this test
      
      mockInvoke.mockResolvedValueOnce({
        success: false,
        error: 'Test error',
        code: 'TEST_ERROR'
      });

      await expect(bridge.invoke('test_command')).rejects.toThrow(PluginError);
    });

    it('should retry on failure', async () => {
      const bridge = new CommunicationBridge(testPluginId, { maxRetries: 2, retryDelay: 10 });
      
      mockInvoke
        .mockRejectedValueOnce(new Error('First failure'))
        .mockRejectedValueOnce(new Error('Second failure'))
        .mockResolvedValueOnce({ success: true, data: 'success' });

      const result = await bridge.invoke('test_command');

      expect(mockInvoke).toHaveBeenCalledTimes(3);
      expect(result).toBe('success');
    });

    it('should fail after max retries', async () => {
      const bridge = new CommunicationBridge(testPluginId, { maxRetries: 1, retryDelay: 10 });
      
      mockInvoke.mockRejectedValue(new Error('Persistent failure'));

      await expect(bridge.invoke('test_command')).rejects.toThrow(PluginError);
      expect(mockInvoke).toHaveBeenCalledTimes(2); // Initial + 1 retry
    });

    it('should not retry on permission denied', async () => {
      mockInvoke.mockRejectedValueOnce(
        new PluginError('Permission denied', PluginErrorCode.PERMISSION_DENIED)
      );

      await expect(bridge.invoke('test_command')).rejects.toThrow(PluginError);
      expect(mockInvoke).toHaveBeenCalledTimes(1); // No retry
    });

    it('should timeout on slow calls', async () => {
      const bridge = new CommunicationBridge(testPluginId, { timeout: 50 });
      
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)));

      try {
        await bridge.invoke('test_command');
        expect.fail('Should have thrown a timeout error');
      } catch (error) {
        expect(error).toBeInstanceOf(PluginError);
        expect((error as PluginError).message).toContain('timeout');
      }
    }, 10000); // Increase test timeout
  });

  describe('configuration', () => {
    it('should update configuration', () => {
      bridge.updateConfig({ maxRetries: 10 });
      // Configuration is private, but we can test behavior
      expect(() => bridge.updateConfig({ debug: true })).not.toThrow();
    });

    it('should return plugin ID', () => {
      expect(bridge.getPluginId()).toBe(testPluginId);
    });
  });

  describe('availability check', () => {
    it('should check if bridge is available', () => {
      // Since we mocked the invoke function, it should be available
      expect(bridge.isAvailable()).toBe(true);
    });
  });
});

describe('Factory functions', () => {
  it('should create communication bridge', () => {
    const bridge = createCommunicationBridge('test-plugin');
    expect(bridge.getPluginId()).toBe('test-plugin');
  });

  it('should set and get global bridge', () => {
    const bridge = createCommunicationBridge('global-test');
    setGlobalBridge(bridge);
    
    const retrieved = getGlobalBridge();
    expect(retrieved).toBe(bridge);
  });
});

describe('Global API utilities', () => {
  let mockInvoke: any;

  beforeEach(async () => {
    // Get the mocked invoke function
    const { invoke } = await import('@tauri-apps/api/core');
    mockInvoke = invoke as any;
    
    const bridge = createCommunicationBridge('global-test');
    setGlobalBridge(bridge);
  });

  afterEach(() => {
    setGlobalBridge(null as any); // Reset global bridge
  });

  it('should use global bridge for API calls', async () => {
    mockInvoke.mockResolvedValueOnce({ success: true, data: 'global result' });

    const result = await invokeApi('test_command', { param: 'value' });

    expect(result).toBe('global result');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_test_command', {
      param: 'value',
      plugin_id: 'global-test'
    });
  });

  it('should throw error when no global bridge is set', async () => {
    setGlobalBridge(null as any);

    await expect(invokeApi('test_command')).rejects.toThrow(
      'No global communication bridge available'
    );
  });
});

describe('Error handling', () => {
  let bridge: CommunicationBridge;
  let mockInvoke: any;

  beforeEach(async () => {
    // Get the mocked invoke function
    const { invoke } = await import('@tauri-apps/api/core');
    mockInvoke = invoke as any;
    
    bridge = new CommunicationBridge('error-test');
  });

  it('should create PluginError for API failures', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Network error'));

    try {
      await bridge.invoke('test_command');
    } catch (error) {
      expect(error).toBeInstanceOf(PluginError);
      expect((error as PluginError).code).toBe(PluginErrorCode.API_CALL_FAILED);
      expect((error as PluginError).pluginName).toBe('error-test');
    }
  });

  it('should preserve existing PluginError', async () => {
    const originalError = new PluginError('Original error', PluginErrorCode.NOT_FOUND);
    mockInvoke.mockRejectedValueOnce(originalError);

    try {
      await bridge.invoke('test_command');
    } catch (error) {
      expect(error).toBe(originalError);
    }
  });
});