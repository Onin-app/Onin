/**
 * Tests for API implementations
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AppAPIImpl, StorageAPIImpl, createAPIImplementations, ApiUtils } from './api';
import { CommunicationBridge } from './communication';

// Mock the communication bridge
const mockBridge = {
  invoke: vi.fn(),
  getPluginId: vi.fn().mockReturnValue('test-plugin'),
  updateConfig: vi.fn(),
  isAvailable: vi.fn().mockReturnValue(true)
} as unknown as CommunicationBridge;

describe('AppAPIImpl', () => {
  let appApi: AppAPIImpl;

  beforeEach(() => {
    appApi = new AppAPIImpl(mockBridge);
    vi.clearAllMocks();
  });

  describe('showNotification', () => {
    it('should call bridge with correct parameters', async () => {
      const message = 'Test notification';
      
      await appApi.showNotification(message);

      expect(mockBridge.invoke).toHaveBeenCalledWith('show_notification', { message });
    });
  });

  describe('getAppVersion', () => {
    it('should return app version', async () => {
      const expectedVersion = '1.0.0';
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedVersion);

      const version = await appApi.getAppVersion();

      expect(mockBridge.invoke).toHaveBeenCalledWith('get_app_version');
      expect(version).toBe(expectedVersion);
    });
  });

  describe('openDialog', () => {
    it('should open dialog with options', async () => {
      const options = {
        title: 'Test Dialog',
        message: 'Test message',
        type: 'info' as const,
        buttons: ['OK', 'Cancel']
      };
      const expectedResult = 'OK';
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedResult);

      const result = await appApi.openDialog(options);

      expect(mockBridge.invoke).toHaveBeenCalledWith('open_dialog', { options });
      expect(result).toBe(expectedResult);
    });
  });

  describe('getAppInfo', () => {
    it('should return app info', async () => {
      const expectedInfo = {
        name: 'Test App',
        version: '1.0.0',
        platform: 'windows'
      };
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedInfo);

      const info = await appApi.getAppInfo();

      expect(mockBridge.invoke).toHaveBeenCalledWith('get_app_info');
      expect(info).toEqual(expectedInfo);
    });
  });

  describe('hasPermission', () => {
    it('should check permission', async () => {
      const permission = 'notifications';
      (mockBridge.invoke as any).mockResolvedValueOnce(true);

      const hasPermission = await appApi.hasPermission(permission);

      expect(mockBridge.invoke).toHaveBeenCalledWith('has_permission', { permission });
      expect(hasPermission).toBe(true);
    });
  });

  describe('requestPermission', () => {
    it('should request permission', async () => {
      const permission = 'storage';
      (mockBridge.invoke as any).mockResolvedValueOnce(true);

      const granted = await appApi.requestPermission(permission);

      expect(mockBridge.invoke).toHaveBeenCalledWith('request_permission', { permission });
      expect(granted).toBe(true);
    });
  });
});

describe('StorageAPIImpl', () => {
  let storageApi: StorageAPIImpl;

  beforeEach(() => {
    storageApi = new StorageAPIImpl(mockBridge);
    vi.clearAllMocks();
  });

  describe('get', () => {
    it('should get value from storage', async () => {
      const key = 'test-key';
      const expectedValue = { data: 'test' };
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedValue);

      const value = await storageApi.get(key);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_get', { key });
      expect(value).toEqual(expectedValue);
    });
  });

  describe('set', () => {
    it('should set value in storage', async () => {
      const key = 'test-key';
      const value = { data: 'test' };

      await storageApi.set(key, value);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_set', { key, value });
    });
  });

  describe('remove', () => {
    it('should remove value from storage', async () => {
      const key = 'test-key';

      await storageApi.remove(key);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_remove', { key });
    });
  });

  describe('clear', () => {
    it('should clear all storage', async () => {
      await storageApi.clear();

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_clear');
    });
  });

  describe('keys', () => {
    it('should get all keys', async () => {
      const expectedKeys = ['key1', 'key2', 'key3'];
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedKeys);

      const keys = await storageApi.keys();

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_keys');
      expect(keys).toEqual(expectedKeys);
    });
  });

  describe('getMany', () => {
    it('should get multiple values', async () => {
      const keys = ['key1', 'key2'];
      const expectedValues = { key1: 'value1', key2: 'value2' };
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedValues);

      const values = await storageApi.getMany(keys);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_get_many', { keys });
      expect(values).toEqual(expectedValues);
    });
  });

  describe('setMany', () => {
    it('should set multiple values', async () => {
      const data = { key1: 'value1', key2: 'value2' };

      await storageApi.setMany(data);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_set_many', { data });
    });
  });

  describe('has', () => {
    it('should check if key exists', async () => {
      const key = 'test-key';
      (mockBridge.invoke as any).mockResolvedValueOnce(true);

      const exists = await storageApi.has(key);

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_has', { key });
      expect(exists).toBe(true);
    });
  });

  describe('getSize', () => {
    it('should get storage size info', async () => {
      const expectedSize = { itemCount: 5, estimatedSize: 1024 };
      (mockBridge.invoke as any).mockResolvedValueOnce(expectedSize);

      const size = await storageApi.getSize();

      expect(mockBridge.invoke).toHaveBeenCalledWith('storage_get_size');
      expect(size).toEqual(expectedSize);
    });
  });
});

describe('createAPIImplementations', () => {
  it('should create API implementations', () => {
    const apis = createAPIImplementations(mockBridge);

    expect(apis.app).toBeInstanceOf(AppAPIImpl);
    expect(apis.storage).toBeInstanceOf(StorageAPIImpl);
  });
});

describe('ApiUtils', () => {
  // Mock the global invokeApi function
  vi.mock('./communication', async () => {
    const actual = await vi.importActual('./communication');
    return {
      ...actual,
      invokeApi: vi.fn()
    };
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('showNotificationSafe', () => {
    it('should show notification successfully', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockResolvedValueOnce(undefined);

      await ApiUtils.showNotificationSafe('Test message');

      expect(invokeApi).toHaveBeenCalledWith('show_notification', { message: 'Test message' });
    });

    it('should use fallback on error', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockRejectedValueOnce(new Error('API failed'));
      
      const fallback = vi.fn();
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      await ApiUtils.showNotificationSafe('Test message', fallback);

      expect(fallback).toHaveBeenCalledWith('Test message');
      expect(consoleSpy).toHaveBeenCalled();
      
      consoleSpy.mockRestore();
    });

    it('should use console fallback when no fallback provided', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockRejectedValueOnce(new Error('API failed'));
      
      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      await ApiUtils.showNotificationSafe('Test message');

      expect(consoleLogSpy).toHaveBeenCalledWith('[Notification] Test message');
      
      consoleLogSpy.mockRestore();
      consoleWarnSpy.mockRestore();
    });
  });

  describe('getAppVersionSafe', () => {
    it('should return app version', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockResolvedValueOnce('1.0.0');

      const version = await ApiUtils.getAppVersionSafe();

      expect(version).toBe('1.0.0');
    });

    it('should return fallback on error', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockRejectedValueOnce(new Error('API failed'));
      
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const version = await ApiUtils.getAppVersionSafe('fallback');

      expect(version).toBe('fallback');
      expect(consoleWarnSpy).toHaveBeenCalled();
      
      consoleWarnSpy.mockRestore();
    });
  });

  describe('checkPermissionSafe', () => {
    it('should check permission successfully', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockResolvedValueOnce(true);

      const hasPermission = await ApiUtils.checkPermissionSafe('notifications');

      expect(hasPermission).toBe(true);
    });

    it('should return false on error', async () => {
      const { invokeApi } = await import('./communication');
      (invokeApi as any).mockRejectedValueOnce(new Error('API failed'));
      
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const hasPermission = await ApiUtils.checkPermissionSafe('notifications');

      expect(hasPermission).toBe(false);
      expect(consoleWarnSpy).toHaveBeenCalled();
      
      consoleWarnSpy.mockRestore();
    });
  });
});