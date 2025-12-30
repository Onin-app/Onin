import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the dependencies using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn()
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn()
}));

// Import after mocking
import {
  setItem,
  getItem,
  removeItem,
  clear,
  keys,
  setItems,
  getItems,
  storage,
  createStorageError,
  isStorageError,
  type StorageError,
  type StorageOptions
} from '../../../src/api/storage';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);

describe('Storage API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock dispatch to call the webview handler by default
    mockDispatch.mockImplementation(({ webview }) => webview());
  });

  describe('setItem', () => {
    it('should set string value', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('username', 'john_doe');

      expect(mockDispatch).toHaveBeenCalledWith({
        webview: expect.any(Function),
        headless: expect.any(Function)
      });
      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'username',
        value: 'john_doe'
      });
    });

    it('should set number value', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('count', 42);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'count',
        value: 42
      });
    });

    it('should set boolean value', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('isEnabled', true);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'isEnabled',
        value: true
      });
    });

    it('should set object value', async () => {
      const userData = {
        name: 'John Doe',
        age: 30,
        preferences: {
          theme: 'dark',
          language: 'en'
        }
      };
      mockInvoke.mockResolvedValue(undefined);

      await setItem('user', userData);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'user',
        value: userData
      });
    });

    it('should set array value', async () => {
      const tags = ['javascript', 'typescript', 'react'];
      mockInvoke.mockResolvedValue(undefined);

      await setItem('tags', tags);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'tags',
        value: tags
      });
    });

    it('should handle null value', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('nullable', null);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'nullable',
        value: null
      });
    });

    it('should handle undefined value', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('undefined', undefined);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'undefined',
        value: undefined
      });
    });

    it('should handle empty string key', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('', 'empty key value');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: '',
        value: 'empty key value'
      });
    });

    it('should handle special characters in key', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItem('key-with.special_chars@123', 'special value');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'key-with.special_chars@123',
        value: 'special value'
      });
    });

    it('should handle errors', async () => {
      const error = new Error('Storage quota exceeded');
      mockInvoke.mockRejectedValue(error);

      await expect(setItem('large-data', 'x'.repeat(10000))).rejects.toThrow('Storage quota exceeded');
    });
  });

  describe('getItem', () => {
    it('should get existing string value', async () => {
      mockInvoke.mockResolvedValue('john_doe');

      const result = await getItem('username');

      expect(result).toBe('john_doe');
      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_get', { key: 'username' });
    });

    it('should get existing number value', async () => {
      mockInvoke.mockResolvedValue(42);

      const result = await getItem<number>('count');

      expect(result).toBe(42);
      expect(typeof result).toBe('number');
    });

    it('should get existing boolean value', async () => {
      mockInvoke.mockResolvedValue(true);

      const result = await getItem<boolean>('isEnabled');

      expect(result).toBe(true);
      expect(typeof result).toBe('boolean');
    });

    it('should get existing object value', async () => {
      const userData = {
        name: 'John Doe',
        age: 30,
        preferences: { theme: 'dark' }
      };
      mockInvoke.mockResolvedValue(userData);

      const result = await getItem<typeof userData>('user');

      expect(result).toEqual(userData);
      expect(result?.name).toBe('John Doe');
    });

    it('should get existing array value', async () => {
      const tags = ['javascript', 'typescript', 'react'];
      mockInvoke.mockResolvedValue(tags);

      const result = await getItem<string[]>('tags');

      expect(result).toEqual(tags);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should return null for non-existing key', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await getItem('nonexistent');

      expect(result).toBeNull();
    });

    it('should handle null stored value', async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await getItem('nullable');

      expect(result).toBeNull();
    });

    it('should handle errors', async () => {
      const error = new Error('Storage access denied');
      mockInvoke.mockRejectedValue(error);

      await expect(getItem('protected-key')).rejects.toThrow('Storage access denied');
    });
  });

  describe('removeItem', () => {
    it('should remove existing item', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeItem('username');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_remove', { key: 'username' });
    });

    it('should handle removing non-existing item', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeItem('nonexistent');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_remove', { key: 'nonexistent' });
    });

    it('should handle special characters in key', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeItem('key-with.special_chars@123');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_remove', { key: 'key-with.special_chars@123' });
    });

    it('should handle errors', async () => {
      const error = new Error('Storage locked');
      mockInvoke.mockRejectedValue(error);

      await expect(removeItem('locked-key')).rejects.toThrow('Storage locked');
    });
  });

  describe('clear', () => {
    it('should clear all storage', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await clear();

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_clear', undefined);
    });

    it('should handle errors', async () => {
      const error = new Error('Clear operation failed');
      mockInvoke.mockRejectedValue(error);

      await expect(clear()).rejects.toThrow('Clear operation failed');
    });
  });

  describe('keys', () => {
    it('should return all keys', async () => {
      const allKeys = ['username', 'count', 'isEnabled', 'user', 'tags'];
      mockInvoke.mockResolvedValue(allKeys);

      const result = await keys();

      expect(result).toEqual(allKeys);
      expect(Array.isArray(result)).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_keys', undefined);
    });

    it('should return empty array when no keys exist', async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await keys();

      expect(result).toEqual([]);
      expect(Array.isArray(result)).toBe(true);
    });

    it('should handle keys with special characters', async () => {
      const specialKeys = ['key-1', 'key.2', 'key_3', 'key@4', 'key with spaces'];
      mockInvoke.mockResolvedValue(specialKeys);

      const result = await keys();

      expect(result).toEqual(specialKeys);
    });

    it('should handle errors', async () => {
      const error = new Error('Keys enumeration failed');
      mockInvoke.mockRejectedValue(error);

      await expect(keys()).rejects.toThrow('Keys enumeration failed');
    });
  });

  describe('setItems', () => {
    it('should set multiple items', async () => {
      const items = {
        username: 'john_doe',
        count: 42,
        isEnabled: true,
        user: { name: 'John', age: 30 }
      };
      mockInvoke.mockResolvedValue(undefined);

      await setItems(items);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set_items', { items });
    });

    it('should handle empty items object', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setItems({});

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set_items', { items: {} });
    });

    it('should handle items with various data types', async () => {
      const items = {
        string: 'text',
        number: 123,
        boolean: false,
        array: [1, 2, 3],
        object: { nested: true },
        null: null,
        undefined: undefined
      };
      mockInvoke.mockResolvedValue(undefined);

      await setItems(items);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set_items', { items });
    });

    it('should handle errors', async () => {
      const error = new Error('Batch set operation failed');
      mockInvoke.mockRejectedValue(error);

      await expect(setItems({ key1: 'value1', key2: 'value2' })).rejects.toThrow('Batch set operation failed');
    });
  });

  describe('getItems', () => {
    it('should get multiple items', async () => {
      const requestedKeys = ['username', 'count', 'isEnabled'];
      const returnedItems = {
        username: 'john_doe',
        count: 42,
        isEnabled: true
      };
      mockInvoke.mockResolvedValue(returnedItems);

      const result = await getItems(requestedKeys);

      expect(result).toEqual(returnedItems);
      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_get_items', { keys: requestedKeys });
    });

    it('should handle empty keys array', async () => {
      mockInvoke.mockResolvedValue({});

      const result = await getItems([]);

      expect(result).toEqual({});
      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_get_items', { keys: [] });
    });

    it('should handle non-existing keys', async () => {
      const requestedKeys = ['existing', 'nonexistent'];
      const returnedItems = {
        existing: 'value',
        nonexistent: null
      };
      mockInvoke.mockResolvedValue(returnedItems);

      const result = await getItems(requestedKeys);

      expect(result).toEqual(returnedItems);
      expect(result.existing).toBe('value');
      expect(result.nonexistent).toBeNull();
    });

    it('should handle various data types', async () => {
      const requestedKeys = ['string', 'number', 'boolean', 'array', 'object'];
      const returnedItems = {
        string: 'text',
        number: 123,
        boolean: false,
        array: [1, 2, 3],
        object: { nested: true }
      };
      mockInvoke.mockResolvedValue(returnedItems);

      const result = await getItems<any>(requestedKeys);

      expect(result).toEqual(returnedItems);
      expect(typeof result.string).toBe('string');
      expect(typeof result.number).toBe('number');
      expect(typeof result.boolean).toBe('boolean');
      expect(Array.isArray(result.array)).toBe(true);
      expect(typeof result.object).toBe('object');
    });

    it('should handle errors', async () => {
      const error = new Error('Batch get operation failed');
      mockInvoke.mockRejectedValue(error);

      await expect(getItems(['key1', 'key2'])).rejects.toThrow('Batch get operation failed');
    });
  });

  describe('storage namespace', () => {
    it('should have all expected methods', () => {
      expect(storage.setItem).toBe(setItem);
      expect(storage.getItem).toBe(getItem);
      expect(storage.removeItem).toBe(removeItem);
      expect(storage.clear).toBe(clear);
      expect(storage.keys).toBe(keys);
      expect(storage.setItems).toBe(setItems);
      expect(storage.getItems).toBe(getItems);
    });

    it('should work through namespace methods', async () => {
      mockInvoke.mockResolvedValue(undefined);

      await storage.setItem('namespace-test', 'value');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'namespace-test',
        value: 'value'
      });
    });
  });

  describe('error handling utilities', () => {
    describe('createStorageError', () => {
      it('should create storage error with message only', () => {
        const error = createStorageError('Storage operation failed');

        expect(error).toBeInstanceOf(Error);
        expect(error.name).toBe('StorageError');
        expect(error.message).toBe('Storage operation failed');
        expect(error.key).toBeUndefined();
      });

      it('should create storage error with message and key', () => {
        const error = createStorageError('Key not found', 'missing-key');

        expect(error.name).toBe('StorageError');
        expect(error.message).toBe('Key not found');
        expect(error.key).toBe('missing-key');
      });
    });

    describe('isStorageError', () => {
      it('should identify storage errors', () => {
        const storageError = createStorageError('Test error');
        const regularError = new Error('Regular error');
        const notAnError = { message: 'Not an error' };

        expect(isStorageError(storageError)).toBe(true);
        expect(isStorageError(regularError)).toBe(false);
        expect(isStorageError(notAnError)).toBe(false);
        expect(isStorageError(null)).toBeFalsy();
        expect(isStorageError(undefined)).toBeFalsy();
      });

      it('should identify storage errors with key', () => {
        const storageError = createStorageError('Test error', 'test-key');

        expect(isStorageError(storageError)).toBe(true);
        expect(storageError.key).toBe('test-key');
      });
    });
  });

  describe('concurrent operations', () => {
    it('should handle multiple simultaneous operations', async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined) // setItem
        .mockResolvedValueOnce('value1') // getItem
        .mockResolvedValueOnce(undefined) // removeItem
        .mockResolvedValueOnce(['key1', 'key2']); // keys

      const [, getValue, , allKeys] = await Promise.all([
        setItem('key1', 'value1'),
        getItem('key1'),
        removeItem('key2'),
        keys()
      ]);

      expect(getValue).toBe('value1');
      expect(allKeys).toEqual(['key1', 'key2']);
      expect(mockInvoke).toHaveBeenCalledTimes(4);
    });

    it('should handle mixed success and failure scenarios', async () => {
      const error = new Error('Operation failed');
      
      mockInvoke
        .mockResolvedValueOnce(undefined) // success
        .mockRejectedValueOnce(error); // failure

      const results = await Promise.allSettled([
        setItem('success-key', 'value'),
        getItem('failure-key')
      ]);

      expect(results[0].status).toBe('fulfilled');
      expect(results[1].status).toBe('rejected');
      expect((results[1] as PromiseRejectedResult).reason).toBe(error);
    });
  });

  describe('data persistence scenarios', () => {
    it('should handle complete data lifecycle', async () => {
      // Simulate a complete data lifecycle: set -> get -> update -> get -> remove -> get
      const initialData = { version: 1, data: 'initial' };
      const updatedData = { version: 2, data: 'updated' };

      mockInvoke
        .mockResolvedValueOnce(undefined) // setItem initial
        .mockResolvedValueOnce(initialData) // getItem initial
        .mockResolvedValueOnce(undefined) // setItem update
        .mockResolvedValueOnce(updatedData) // getItem updated
        .mockResolvedValueOnce(undefined) // removeItem
        .mockResolvedValueOnce(null); // getItem after removal

      // Set initial data
      await setItem('lifecycle-test', initialData);

      // Get initial data
      let result = await getItem('lifecycle-test');
      expect(result).toEqual(initialData);

      // Update data
      await setItem('lifecycle-test', updatedData);

      // Get updated data
      result = await getItem('lifecycle-test');
      expect(result).toEqual(updatedData);

      // Remove data
      await removeItem('lifecycle-test');

      // Verify removal
      result = await getItem('lifecycle-test');
      expect(result).toBeNull();

      expect(mockInvoke).toHaveBeenCalledTimes(6);
    });

    it('should handle batch operations workflow', async () => {
      const batchData = {
        user: { name: 'John', id: 1 },
        settings: { theme: 'dark', lang: 'en' },
        cache: { lastUpdate: Date.now() }
      };

      const requestKeys = ['user', 'settings', 'cache'];

      mockInvoke
        .mockResolvedValueOnce(undefined) // setItems
        .mockResolvedValueOnce(requestKeys) // keys
        .mockResolvedValueOnce(batchData) // getItems
        .mockResolvedValueOnce(undefined); // clear

      // Batch set
      await setItems(batchData);

      // Get all keys
      const allKeys = await keys();
      expect(allKeys).toEqual(requestKeys);

      // Batch get
      const retrievedData = await getItems(requestKeys);
      expect(retrievedData).toEqual(batchData);

      // Clear all
      await clear();

      expect(mockInvoke).toHaveBeenCalledTimes(4);
    });
  });

  describe('edge cases', () => {
    it('should handle very long keys', async () => {
      const longKey = 'a'.repeat(1000);
      mockInvoke.mockResolvedValue(undefined);

      await setItem(longKey, 'value');

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: longKey,
        value: 'value'
      });
    });

    it('should handle very large values', async () => {
      const largeValue = { data: 'x'.repeat(10000) };
      mockInvoke.mockResolvedValue(undefined);

      await setItem('large-data', largeValue);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'large-data',
        value: largeValue
      });
    });

    it('should handle unicode characters in keys and values', async () => {
      const unicodeKey = '测试键名🔑';
      const unicodeValue = '测试值内容🎯';
      mockInvoke.mockResolvedValue(undefined);

      await setItem(unicodeKey, unicodeValue);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: unicodeKey,
        value: unicodeValue
      });
    });

    it('should handle circular references in objects', async () => {
      const circularObj: any = { name: 'test' };
      circularObj.self = circularObj;

      mockInvoke.mockResolvedValue(undefined);

      // This should be handled by the serialization layer
      await setItem('circular', circularObj);

      expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
        key: 'circular',
        value: circularObj
      });
    });
  });
});