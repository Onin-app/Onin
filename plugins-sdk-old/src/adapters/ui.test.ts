import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createUIAdapter } from './ui';
import { ValidationError } from '../types';

// 模拟 @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('UI 适配器', () => {
  let adapter: ReturnType<typeof createUIAdapter>;

  beforeEach(() => {
    vi.clearAllMocks();
    adapter = createUIAdapter();
  });

  describe('getEnvironment', () => {
    it('应该返回 ui', () => {
      expect(adapter.getEnvironment()).toBe('ui');
    });
  });

  describe('showNotification', () => {
    it('应该成功调用通知', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockResolvedValue(undefined);

      const options = {
        title: '测试标题',
        body: '测试内容'
      };

      const result = await adapter.showNotification(options);

      expect(mockInvoke).toHaveBeenCalledWith('show_notification', { options });
      expect(result).toEqual({
        success: true,
        data: undefined
      });
    });

    it('应该处理 Tauri 调用错误', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue({ message: 'Tauri 调用失败' });

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: 'Tauri 调用失败'
      });
    });

    it('应该处理一般错误', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue(new Error('网络错误'));

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: '网络错误'
      });
    });

    it('应该处理未知错误', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue('未知错误');

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: '调用失败'
      });
    });

    it('应该在参数验证失败时抛出错误', async () => {
      const invalidOptions = {
        title: 123 // 无效类型
      };

      await expect(adapter.showNotification(invalidOptions as any))
        .rejects.toThrow(ValidationError);
    });
  });
});