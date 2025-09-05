import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createHeadlessAdapter } from './headless';
import { ValidationError } from '../types';

describe('Headless 适配器', () => {
  let adapter: ReturnType<typeof createHeadlessAdapter>;

  beforeEach(() => {
    vi.unstubAllGlobals();
    adapter = createHeadlessAdapter();
  });

  describe('getEnvironment', () => {
    it('应该返回 headless', () => {
      expect(adapter.getEnvironment()).toBe('headless');
    });
  });

  describe('showNotification', () => {
    it('应该成功调用通知', async () => {
      // 模拟 Deno.core.ops.op_invoke
      const mockOpInvoke = vi.fn().mockResolvedValue({ Ok: null });
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: mockOpInvoke
          }
        }
      });

      const options = {
        title: '测试标题',
        body: '测试内容'
      };

      const result = await adapter.showNotification(options);

      expect(mockOpInvoke).toHaveBeenCalledWith('show_notification', options);
      expect(result).toEqual({
        success: true,
        data: undefined
      });
    });

    it('应该处理错误响应', async () => {
      const mockOpInvoke = vi.fn().mockResolvedValue({ Err: '调用失败' });
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: mockOpInvoke
          }
        }
      });

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: '调用失败'
      });
    });

    it('应该处理直接成功响应', async () => {
      const mockOpInvoke = vi.fn().mockResolvedValue(null);
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: mockOpInvoke
          }
        }
      });

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: true,
        data: undefined
      });
    });

    it('应该在 op_invoke 不可用时返回错误', async () => {
      // 不设置 Deno 对象
      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: 'Deno.core.ops.op_invoke 不可用'
      });
    });

    it('应该在参数验证失败时抛出错误', async () => {
      const invalidOptions = {
        title: 123 // 无效类型
      };

      await expect(adapter.showNotification(invalidOptions as any))
        .rejects.toThrow(ValidationError);
    });

    it('应该处理 op_invoke 抛出的异常', async () => {
      const mockOpInvoke = vi.fn().mockRejectedValue(new Error('网络错误'));
      vi.stubGlobal('Deno', {
        core: {
          ops: {
            op_invoke: mockOpInvoke
          }
        }
      });

      const options = {
        title: '测试标题'
      };

      const result = await adapter.showNotification(options);

      expect(result).toEqual({
        success: false,
        error: '网络错误'
      });
    });
  });
});