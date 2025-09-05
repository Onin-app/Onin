import { describe, it, expect, beforeEach, vi } from 'vitest';
import { detectEnvironment, isHeadlessEnvironment, isUIEnvironment } from './environment';
import { PluginSDKError, ERROR_CODES } from '../types';

describe('环境检测', () => {
  beforeEach(() => {
    // 清理全局对象
    vi.unstubAllGlobals();
    delete (globalThis as any).Deno;
    delete (window as any).__TAURI__;
  });

  describe('detectEnvironment', () => {
    it('应该检测到 headless 环境', () => {
      // 模拟 Deno 环境
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: vi.fn()
          }
        }
      });

      expect(detectEnvironment()).toBe('headless');
    });

    it('应该检测到 UI 环境', () => {
      // 模拟 Tauri 环境
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      expect(detectEnvironment()).toBe('ui');
    });

    it('应该在未知环境中抛出错误', () => {
      expect(() => detectEnvironment()).toThrow(PluginSDKError);
      
      try {
        detectEnvironment();
      } catch (error) {
        expect(error).toBeInstanceOf(PluginSDKError);
        expect((error as PluginSDKError).code).toBe(ERROR_CODES.UNSUPPORTED_ENVIRONMENT);
      }
    });

    it('Deno 对象不完整时应该检测为未知环境', () => {
      vi.stubGlobal('Deno', {
        core: {} // 缺少 ops
      });

      expect(() => detectEnvironment()).toThrow(PluginSDKError);
    });
  });

  describe('isHeadlessEnvironment', () => {
    it('在 headless 环境中应该返回 true', () => {
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: vi.fn()
          }
        }
      });

      expect(isHeadlessEnvironment()).toBe(true);
    });

    it('在非 headless 环境中应该返回 false', () => {
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      expect(isHeadlessEnvironment()).toBe(false);
    });

    it('在未知环境中应该返回 false', () => {
      expect(isHeadlessEnvironment()).toBe(false);
    });
  });

  describe('isUIEnvironment', () => {
    it('在 UI 环境中应该返回 true', () => {
      Object.defineProperty(window, '__TAURI__', {
        value: {},
        configurable: true
      });

      expect(isUIEnvironment()).toBe(true);
    });

    it('在非 UI 环境中应该返回 false', () => {
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: vi.fn()
          }
        }
      });

      expect(isUIEnvironment()).toBe(false);
    });

    it('在未知环境中应该返回 false', () => {
      expect(isUIEnvironment()).toBe(false);
    });
  });
});