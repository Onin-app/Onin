import type { PluginSDKFunctions, NotificationOptions, ApiResponse } from '../types';
import { InvokeError } from '../types';
import { validateNotificationOptions } from '../core/validation';
import { MESSAGES } from '../utils/constants';

/**
 * 创建 UI 环境适配器
 * @returns PluginSDKFunctions 实现
 */
export function createUIAdapter(): PluginSDKFunctions {
  return {
    getEnvironment: () => 'ui',

    async showNotification(options: NotificationOptions): Promise<ApiResponse<void>> {
      try {
        // 验证参数
        validateNotificationOptions(options);

        // 动态导入 Tauri API
        const { invoke } = await import('@tauri-apps/api/core');

        // 调用 Tauri 命令
        await invoke('show_notification', { options });

        return {
          success: true,
          data: undefined
        };
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : MESSAGES.INVOKE_FAILED;
        
        if (error instanceof InvokeError) {
          return {
            success: false,
            error: errorMessage
          };
        }

        // 处理 Tauri 调用错误
        if (error && typeof error === 'object' && 'message' in error) {
          return {
            success: false,
            error: String(error.message)
          };
        }

        // 如果是验证错误，重新抛出
        if (error instanceof Error && error.name === 'ValidationError') {
          throw error;
        }

        return {
          success: false,
          error: errorMessage
        };
      }
    }
  };
}