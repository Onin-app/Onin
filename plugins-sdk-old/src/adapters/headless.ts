import type { PluginSDKFunctions, NotificationOptions, ApiResponse } from '../types';
import { InvokeError } from '../types';
import { validateNotificationOptions } from '../core/validation';
import { MESSAGES } from '../utils/constants';

/**
 * 创建 headless 环境适配器
 * @returns PluginSDKFunctions 实现
 */
export function createHeadlessAdapter(): PluginSDKFunctions {
  return {
    getEnvironment: () => 'headless',

    async showNotification(options: NotificationOptions): Promise<ApiResponse<void>> {
      try {
        // 验证参数
        validateNotificationOptions(options);

        // 检查 Deno.core.ops 是否可用
        if (!globalThis.Deno?.core?.ops?.op_invoke) {
          throw new InvokeError('Deno.core.ops.op_invoke 不可用');
        }

        // 调用 op_invoke
        const result = await globalThis.Deno.core.ops.op_invoke(
          'show_notification',
          options
        );

        // 处理结果
        if (result && typeof result === 'object' && 'Ok' in result) {
          return {
            success: true,
            data: undefined
          };
        } else if (result && typeof result === 'object' && 'Err' in result) {
          return {
            success: false,
            error: typeof result.Err === 'string' ? result.Err : '调用失败'
          };
        } else {
          // 兼容直接返回成功的情况
          return {
            success: true,
            data: undefined
          };
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : MESSAGES.INVOKE_FAILED;
        
        if (error instanceof InvokeError) {
          return {
            success: false,
            error: errorMessage
          };
        }

        throw error; // 重新抛出验证错误等其他错误
      }
    }
  };
}