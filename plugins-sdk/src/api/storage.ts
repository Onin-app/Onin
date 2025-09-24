import { invoke } from '../core/ipc';
import { dispatch } from '../core/dispatch';

export interface StorageOptions {
  // 预留配置选项
}

export interface StorageError extends Error {
  name: 'StorageError';
  key?: string;
}

// 错误工厂函数
export function createStorageError(message: string, key?: string): StorageError {
  const error = new Error(message) as StorageError;
  error.name = 'StorageError';
  error.key = key;
  return error;
}

// 类型检查函数
export function isStorageError(error: any): error is StorageError {
  return error && error.name === 'StorageError';
}

/**
 * 设置一个键值对到存储中
 * @param key 存储键
 * @param value 存储值（会自动序列化）
 */
export function setItem(key: string, value: any): Promise<void> {
  return dispatch({
    webview: () => invoke("plugin_storage_set", { key, value }),
    headless: () => invoke("plugin_storage_set", { key, value }),
  });
}

/**
 * 从存储中获取一个值
 * @param key 存储键
 * @returns 存储的值，如果不存在则返回 null
 */
export function getItem<T = any>(key: string): Promise<T | null> {
  return dispatch({
    webview: () => invoke<T | null>("plugin_storage_get", { key }),
    headless: () => invoke<T | null>("plugin_storage_get", { key }),
  });
}

/**
 * 从存储中删除一个键
 * @param key 要删除的键
 */
export function removeItem(key: string): Promise<void> {
  return dispatch({
    webview: () => invoke("plugin_storage_remove", { key }),
    headless: () => invoke("plugin_storage_remove", { key }),
  });
}

/**
 * 清空当前插件的所有存储数据
 */
export function clear(): Promise<void> {
  return dispatch({
    webview: () => invoke("plugin_storage_clear"),
    headless: () => invoke("plugin_storage_clear"),
  });
}

/**
 * 获取当前插件存储的所有键
 * @returns 所有键的数组
 */
export function keys(): Promise<string[]> {
  return dispatch({
    webview: () => invoke<string[]>("plugin_storage_keys"),
    headless: () => invoke<string[]>("plugin_storage_keys"),
  });
}

/**
 * 批量设置多个键值对
 * @param items 要设置的键值对对象
 */
export function setItems(items: Record<string, any>): Promise<void> {
  return dispatch({
    webview: () => invoke("plugin_storage_set_items", { items }),
    headless: () => invoke("plugin_storage_set_items", { items }),
  });
}

/**
 * 批量获取多个键的值
 * @param keys 要获取的键数组
 * @returns 包含键值对的对象
 */
export function getItems<T = any>(keys: string[]): Promise<Record<string, T>> {
  return dispatch({
    webview: () => invoke<Record<string, T>>("plugin_storage_get_items", { keys }),
    headless: () => invoke<Record<string, T>>("plugin_storage_get_items", { keys }),
  });
}

// 导出默认的存储对象
export const storage = {
  setItem,
  getItem,
  removeItem,
  clear,
  keys,
  setItems,
  getItems
};
