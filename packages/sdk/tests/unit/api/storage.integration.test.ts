import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the modules using factory functions
vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn(),
}));

vi.mock('../../../src/core/dispatch', () => ({
  dispatch: vi.fn(),
}));

vi.mock('../../../src/core/environment', async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    getEnvironment: vi.fn(),
  };
});

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
} from '../../../src/api/storage';
import { invoke } from '../../../src/core/ipc';
import { dispatch } from '../../../src/core/dispatch';
import {
  getEnvironment,
  RuntimeEnvironment,
} from '../../../src/core/environment';

// Get the mocked functions
const mockInvoke = vi.mocked(invoke);
const mockDispatch = vi.mocked(dispatch);
const mockGetEnvironment = vi.mocked(getEnvironment);

describe('Storage API Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock dispatch to call the webview handler by default
    mockDispatch.mockImplementation(({ webview }) => webview());
  });

  it('should work correctly in both webview and headless environments', async () => {
    // Test webview environment
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);
    mockInvoke.mockResolvedValue(undefined);

    await setItem('test-key', 'webview-value');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set', {
      key: 'test-key',
      value: 'webview-value',
    });

    // Reset and test headless environment
    mockInvoke.mockClear();
    mockInvoke.mockResolvedValue('headless-value');
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const result = await getItem('test-key');
    expect(result).toBe('headless-value');
    expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_get', {
      key: 'test-key',
    });
  });

  it('should handle complete application state management workflow', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Simulate a complete application state management workflow
    const initialState = {
      user: {
        id: 1,
        name: 'John Doe',
        email: 'john@example.com',
        preferences: {
          theme: 'dark',
          language: 'en',
          notifications: true,
        },
      },
      session: {
        token: 'abc123',
        expires: Date.now() + 3600000,
        lastActivity: Date.now(),
      },
      cache: {
        recentFiles: ['file1.txt', 'file2.txt'],
        searchHistory: ['query1', 'query2'],
        tempData: { processing: false },
      },
    };

    mockInvoke
      .mockResolvedValueOnce(undefined) // setItems - initial state
      .mockResolvedValueOnce(['user', 'session', 'cache']) // keys
      .mockResolvedValueOnce(initialState.user) // getItem user
      .mockResolvedValueOnce(undefined) // setItem - update user preferences
      .mockResolvedValueOnce({
        user: {
          ...initialState.user,
          preferences: { ...initialState.user.preferences, theme: 'light' },
        },
        session: initialState.session,
      }) // getItems - get updated state
      .mockResolvedValueOnce(undefined) // removeItem - clear cache
      .mockResolvedValueOnce(['user', 'session']) // keys - after cache removal
      .mockResolvedValueOnce(undefined); // clear - clear all

    // Initialize application state
    await storage.setItems(initialState);
    expect(mockInvoke).toHaveBeenCalledWith('plugin_storage_set_items', {
      items: initialState,
    });

    // Get all stored keys
    const allKeys = await storage.keys();
    expect(allKeys).toEqual(['user', 'session', 'cache']);

    // Get user data
    const userData = await storage.getItem('user');
    expect(userData).toEqual(initialState.user);

    // Update user preferences
    const updatedUser = {
      ...initialState.user,
      preferences: { ...initialState.user.preferences, theme: 'light' },
    };
    await storage.setItem('user', updatedUser);

    // Get multiple items to verify state
    const currentState = await storage.getItems(['user', 'session']);
    expect(currentState.user.preferences.theme).toBe('light');
    expect(currentState.session).toEqual(initialState.session);

    // Clear cache data
    await storage.removeItem('cache');

    // Verify cache is removed
    const remainingKeys = await storage.keys();
    expect(remainingKeys).toEqual(['user', 'session']);

    // Clear all data
    await storage.clear();

    expect(mockInvoke).toHaveBeenCalledTimes(8);
  });

  it('should handle plugin configuration management', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const pluginConfig = {
      version: '1.0.0',
      settings: {
        autoSave: true,
        interval: 5000,
        maxRetries: 3,
        endpoints: {
          api: 'https://api.example.com',
          cdn: 'https://cdn.example.com',
        },
      },
      features: {
        experimental: false,
        beta: ['feature1', 'feature2'],
        disabled: [],
      },
      userOverrides: {
        setting1: 'custom-value',
        setting2: false,
      },
    };

    mockInvoke
      .mockResolvedValueOnce(undefined) // setItem config
      .mockResolvedValueOnce(pluginConfig) // getItem config
      .mockResolvedValueOnce(undefined) // setItem - update settings
      .mockResolvedValueOnce({
        ...pluginConfig,
        settings: {
          ...pluginConfig.settings,
          autoSave: false,
          interval: 10000,
        },
      }) // getItem - updated config
      .mockResolvedValueOnce(undefined); // setItem - save final config

    // Save initial configuration
    await setItem('plugin-config', pluginConfig);

    // Load configuration
    const loadedConfig = await getItem('plugin-config');
    expect(loadedConfig).toEqual(pluginConfig);

    // Update specific settings
    const updatedConfig = {
      ...pluginConfig,
      settings: {
        ...pluginConfig.settings,
        autoSave: false,
        interval: 10000,
      },
    };
    await setItem('plugin-config', updatedConfig);

    // Verify updated configuration
    const finalConfig = await getItem('plugin-config');
    expect(finalConfig.settings.autoSave).toBe(false);
    expect(finalConfig.settings.interval).toBe(10000);

    // Save configuration with version bump
    await setItem('plugin-config', {
      ...finalConfig,
      version: '1.0.1',
    });

    expect(mockInvoke).toHaveBeenCalledTimes(5);
  });

  it('should handle data migration scenarios', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Simulate data migration from v1 to v2 format
    const v1Data = {
      userSettings: 'old-format-string',
      preferences: 'theme:dark,lang:en',
      version: 1,
    };

    const v2Data = {
      user: {
        settings: { migrated: true },
        preferences: { theme: 'dark', lang: 'en' },
      },
      meta: {
        version: 2,
        migratedAt: Date.now(),
      },
    };

    mockInvoke
      .mockResolvedValueOnce(['userSettings', 'preferences', 'version']) // keys - get old keys
      .mockResolvedValueOnce(v1Data) // getItems - get old data
      .mockResolvedValueOnce(undefined) // setItems - save new format
      .mockResolvedValueOnce(undefined) // removeItem - remove old userSettings
      .mockResolvedValueOnce(undefined) // removeItem - remove old preferences
      .mockResolvedValueOnce(undefined) // removeItem - remove old version
      .mockResolvedValueOnce(['user', 'meta']) // keys - verify new structure
      .mockResolvedValueOnce(v2Data); // getItems - verify migrated data

    // Check existing data structure
    const existingKeys = await keys();
    expect(existingKeys).toEqual(['userSettings', 'preferences', 'version']);

    // Get old data for migration
    const oldData = await getItems(existingKeys);
    expect(oldData).toEqual(v1Data);

    // Save new data structure
    await setItems(v2Data);

    // Remove old data keys
    await removeItem('userSettings');
    await removeItem('preferences');
    await removeItem('version');

    // Verify new structure
    const newKeys = await keys();
    expect(newKeys).toEqual(['user', 'meta']);

    // Verify migrated data
    const migratedData = await getItems(['user', 'meta']);
    expect(migratedData).toEqual(v2Data);
    expect(migratedData.meta.version).toBe(2);

    expect(mockInvoke).toHaveBeenCalledTimes(8);
  });

  it('should handle caching and performance optimization', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    // Simulate caching frequently accessed data
    const cacheData = {
      'api-response-1': {
        data: { users: [{ id: 1, name: 'John' }] },
        timestamp: Date.now(),
        ttl: 300000, // 5 minutes
      },
      'api-response-2': {
        data: { posts: [{ id: 1, title: 'Post 1' }] },
        timestamp: Date.now(),
        ttl: 600000, // 10 minutes
      },
      'user-preferences': {
        theme: 'dark',
        lastAccessed: Date.now(),
      },
    };

    const cacheKeys = Object.keys(cacheData);

    mockInvoke
      .mockResolvedValueOnce(undefined) // setItems - populate cache
      .mockResolvedValueOnce(cacheKeys) // keys - get all cache keys
      .mockResolvedValueOnce(cacheData) // getItems - get all cached data
      .mockResolvedValueOnce(undefined) // removeItem - remove expired cache
      .mockResolvedValueOnce(['api-response-2', 'user-preferences']) // keys - after cleanup
      .mockResolvedValueOnce({
        'api-response-2': cacheData['api-response-2'],
        'user-preferences': cacheData['user-preferences'],
      }); // getItems - remaining cache

    // Populate cache
    await setItems(cacheData);

    // Get all cache keys for cleanup check
    const allCacheKeys = await keys();
    expect(allCacheKeys).toEqual(cacheKeys);

    // Get all cached data
    const allCachedData = await getItems(allCacheKeys);
    expect(allCachedData).toEqual(cacheData);

    // Simulate cache cleanup (remove expired item)
    await removeItem('api-response-1');

    // Verify remaining cache
    const remainingKeys = await keys();
    expect(remainingKeys).toEqual(['api-response-2', 'user-preferences']);

    const remainingCache = await getItems(remainingKeys);
    expect(remainingCache['api-response-1']).toBeUndefined();
    expect(remainingCache['api-response-2']).toBeDefined();

    expect(mockInvoke).toHaveBeenCalledTimes(6);
  });

  it('should handle concurrent storage operations safely', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    // Simulate concurrent operations that might happen in real usage
    mockInvoke
      .mockResolvedValueOnce(undefined) // setItem user1
      .mockResolvedValueOnce(undefined) // setItem user2
      .mockResolvedValueOnce(undefined) // setItem user3
      .mockResolvedValueOnce('user1-data') // getItem user1
      .mockResolvedValueOnce('user2-data') // getItem user2
      .mockResolvedValueOnce('user3-data') // getItem user3
      .mockResolvedValueOnce(undefined) // removeItem user2
      .mockResolvedValueOnce(['user1', 'user3']); // keys after removal

    const operations = await Promise.all([
      // Concurrent writes
      setItem('user1', 'user1-data'),
      setItem('user2', 'user2-data'),
      setItem('user3', 'user3-data'),
    ]);

    // Concurrent reads
    const [user1, user2, user3] = await Promise.all([
      getItem('user1'),
      getItem('user2'),
      getItem('user3'),
    ]);

    expect(user1).toBe('user1-data');
    expect(user2).toBe('user2-data');
    expect(user3).toBe('user3-data');

    // Mixed operations
    const [, finalKeys] = await Promise.all([removeItem('user2'), keys()]);

    expect(finalKeys).toEqual(['user1', 'user3']);
    expect(mockInvoke).toHaveBeenCalledTimes(8);
  });

  it('should handle error recovery scenarios', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const storageError = new Error('Storage quota exceeded');
    const fallbackData = { fallback: true, timestamp: Date.now() };

    mockInvoke
      .mockRejectedValueOnce(storageError) // setItem fails
      .mockResolvedValueOnce(undefined) // setItem succeeds (retry)
      .mockResolvedValueOnce(fallbackData) // getItem succeeds
      .mockRejectedValueOnce(new Error('Storage locked')) // removeItem fails
      .mockResolvedValueOnce(undefined); // removeItem succeeds (retry)

    // First operation fails
    await expect(setItem('large-data', 'x'.repeat(10000))).rejects.toThrow(
      'Storage quota exceeded',
    );

    // Retry with smaller data succeeds
    await setItem('small-data', fallbackData);

    // Verify data was stored
    const result = await getItem('small-data');
    expect(result).toEqual(fallbackData);

    // Remove operation fails first
    await expect(removeItem('locked-key')).rejects.toThrow('Storage locked');

    // Retry succeeds
    await removeItem('unlocked-key');

    expect(mockInvoke).toHaveBeenCalledTimes(5);
  });

  it('should handle complex data structures and serialization', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Webview);

    const complexData = {
      metadata: {
        created: new Date('2023-01-01'),
        tags: new Set(['tag1', 'tag2', 'tag3']),
        map: new Map([
          ['key1', 'value1'],
          ['key2', 'value2'],
        ]),
      },
      nested: {
        level1: {
          level2: {
            level3: {
              data: 'deep nested value',
              array: [1, 2, { nested: true }],
            },
          },
        },
      },
      functions: {
        // Functions should be handled appropriately
        callback: () => 'test',
      },
    };

    // Note: The actual serialization behavior depends on the implementation
    // This test assumes the storage layer handles serialization
    mockInvoke
      .mockResolvedValueOnce(undefined) // setItem
      .mockResolvedValueOnce(complexData); // getItem

    await setItem('complex-data', complexData);
    const retrieved = await getItem('complex-data');

    expect(retrieved).toEqual(complexData);
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it('should handle storage quota and cleanup scenarios', async () => {
    mockGetEnvironment.mockReturnValue(RuntimeEnvironment.Headless);

    const quotaError = createStorageError(
      'Storage quota exceeded',
      'large-file',
    );
    const cleanupData = {
      'temp-1': { data: 'temporary', priority: 'low' },
      'temp-2': { data: 'temporary', priority: 'low' },
      important: { data: 'important', priority: 'high' },
    };

    mockInvoke
      .mockRejectedValueOnce(quotaError) // setItem fails due to quota
      .mockResolvedValueOnce(['temp-1', 'temp-2', 'important']) // keys for cleanup
      .mockResolvedValueOnce(cleanupData) // getItems for cleanup analysis
      .mockResolvedValueOnce(undefined) // removeItem temp-1
      .mockResolvedValueOnce(undefined) // removeItem temp-2
      .mockResolvedValueOnce(undefined) // setItem succeeds after cleanup
      .mockResolvedValueOnce(['important', 'new-data']); // keys after cleanup

    // Attempt to store large data fails
    await expect(setItem('large-file', 'x'.repeat(100000))).rejects.toThrow();
    expect(isStorageError(quotaError)).toBe(true);

    // Get existing data for cleanup analysis
    const existingKeys = await keys();
    const existingData = await getItems(existingKeys);

    // Remove low priority items
    await removeItem('temp-1');
    await removeItem('temp-2');

    // Retry storing data after cleanup
    await setItem('new-data', 'important data');

    // Verify final state
    const finalKeys = await keys();
    expect(finalKeys).toEqual(['important', 'new-data']);

    expect(mockInvoke).toHaveBeenCalledTimes(7);
  });
});
