import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createPluginSDK } from './sdk';
import { PluginSDKError } from './types';

describe('主 SDK', () => {
  beforeEach(() => {
    vi.unstubAllGlobals();
    delete (globalThis as any).Deno;
    delete (window as any).__TAURI__;
  });

  describe('createPluginSDK', () => {
    it('应该在 headless 环境中创建 headless 适配器', () => {
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: vi.fn()
          }
        }
      });

      const sdk = createPluginSDK();
      expect(sdk.getEnvironment()).toBe('headless');
    });

    it('应该在 UI 环境中创建 UI 适配器', () => {
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      const sdk = createPluginSDK();
      expect(sdk.getEnvironment()).toBe('ui');
    });

    it('应该在未知环境中抛出错误', () => {
      expect(() => createPluginSDK()).toThrow(PluginSDKError);
    });

    it('应该优先检测 headless 环境', () => {
      // 同时设置两个环境
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: vi.fn()
          }
        }
      });
      
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      const sdk = createPluginSDK();
      expect(sdk.getEnvironment()).toBe('headless');
    });
  });

  describe('SDK 功能集成', () => {
    it('应该在 headless 环境中正确调用通知', async () => {
      const mockOpInvoke = vi.fn().mockResolvedValue({ Ok: null });
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: mockOpInvoke
          }
        }
      });

      const sdk = createPluginSDK();
      const result = await sdk.showNotification({
        title: '测试标题'
      });

      expect(mockOpInvoke).toHaveBeenCalledWith('show_notification', {
        title: '测试标题'
      });
      expect(result.success).toBe(true);
    });

    it('应该在 UI 环境中正确调用通知', async () => {
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      // 模拟动态导入
      vi.doMock('@tauri-apps/api/core', () => ({
        invoke: vi.fn().mockResolvedValue(undefined)
      }));

      const sdk = createPluginSDK();
      const result = await sdk.showNotification({
        title: '测试标题'
      });

      expect(result.success).toBe(true);
    });
  });
});