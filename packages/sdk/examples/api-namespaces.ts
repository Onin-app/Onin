import {
  http,
  fs,
  clipboard,
  dialog,
  errorCode,
  errorUtils,
} from '../src/index';

/**
 * 展示如何在各个 API 命名空间中使用统一的错误处理
 */

// HTTP API 命名空间使用示例
export const httpExamples = {
  async fetchData() {
    try {
      const response = await http.get('https://api.example.com/data');
      return response.body;
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.http.TIMEOUT:
            throw new Error('网络超时，请稍后重试');
          case errorCode.http.HTTP_ERROR:
            throw new Error(`服务器错误: ${error.context?.status}`);
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('需要配置 HTTP 权限');
          default:
            throw new Error(`HTTP 请求失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async uploadFile(data: any) {
    try {
      return await http.post('https://api.example.com/upload', data);
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        // 使用便捷的检查方法
        if (
          errorUtils.isOneOfErrorCodes(error, [
            errorCode.http.NETWORK_ERROR,
            errorCode.http.TIMEOUT,
          ])
        ) {
          console.error('网络问题，上传失败');
          return null;
        }
      }
      throw error;
    }
  },
};

// 文件系统 API 命名空间使用示例
export const fsExamples = {
  async readConfig(path: string) {
    try {
      const content = await fs.readFile(path);
      return JSON.parse(content);
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.fs.FILE_NOT_FOUND:
            console.log('配置文件不存在，使用默认配置');
            return {};
          case errorCode.fs.FILE_ACCESS_DENIED:
            throw new Error('无法访问配置文件');
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('需要配置文件系统权限');
          default:
            throw new Error(`读取配置失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async saveConfig(path: string, config: any) {
    try {
      await fs.writeFile(path, JSON.stringify(config, null, 2));
      console.log('配置保存成功');
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.fs.FILE_ACCESS_DENIED)) {
          throw new Error('无法写入配置文件，请检查权限');
        }
      }
      throw error;
    }
  },
};

// 剪贴板 API 命名空间使用示例
export const clipboardExamples = {
  async copyText(text: string) {
    try {
      await clipboard.writeText(text);
      console.log('文本已复制到剪贴板');
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.clipboard.UNAVAILABLE:
            console.error('剪贴板不可用');
            break;
          case errorCode.common.PERMISSION_DENIED:
            console.error('需要配置剪贴板权限');
            break;
          default:
            console.error('复制失败:', error.message);
        }
      }
    }
  },

  async pasteText() {
    try {
      const text = await clipboard.readText();
      return text || '';
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.clipboard.UNAVAILABLE)) {
          console.warn('剪贴板不可用，返回空字符串');
          return '';
        }
      }
      throw error;
    }
  },
};

// 对话框 API 命名空间使用示例
export const dialogExamples = {
  async confirmAction(message: string) {
    try {
      return await dialog.showConfirm({
        title: '确认操作',
        message,
      });
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.dialog.CANCELLED:
            console.log('用户取消了操作');
            return false;
          case errorCode.common.PERMISSION_DENIED:
            console.error('需要配置对话框权限');
            return false;
          default:
            console.error('对话框错误:', error.message);
            return false;
        }
      }
      return false;
    }
  },

  async selectFile() {
    try {
      const result = await dialog.showOpen({
        title: '选择文件',
        multiple: false,
      });
      return result as string | null;
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.dialog.CANCELLED)) {
          return null; // 用户取消选择
        }
      }
      throw error;
    }
  },
};

// 统一的错误处理工具
export const errorHandling = {
  /**
   * 处理任何插件错误并返回用户友好的消息
   */
  getUserFriendlyMessage(error: unknown): string {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.common.PERMISSION_DENIED:
          return '权限不足，请检查插件配置';
        case errorCode.http.NETWORK_ERROR:
          return '网络连接失败，请检查网络';
        case errorCode.http.TIMEOUT:
          return '请求超时，请稍后重试';
        case errorCode.fs.FILE_NOT_FOUND:
          return '文件不存在';
        case errorCode.clipboard.UNAVAILABLE:
          return '剪贴板不可用';
        case errorCode.dialog.CANCELLED:
          return '操作已取消';
        default:
          return `操作失败: ${error.message}`;
      }
    }
    return '发生未知错误';
  },

  /**
   * 检查错误是否可以重试
   */
  isRetryable(error: unknown): boolean {
    if (errorUtils.isPluginError(error)) {
      return errorUtils.isOneOfErrorCodes(error, [
        errorCode.http.NETWORK_ERROR,
        errorCode.http.TIMEOUT,
        errorCode.clipboard.UNAVAILABLE,
      ]);
    }
    return false;
  },

  /**
   * 记录错误详情（用于调试）
   */
  logError(error: unknown, context: string) {
    if (errorUtils.isPluginError(error)) {
      console.error(`[${context}] 插件错误:`, {
        code: error.code,
        message: error.message,
        context: error.context,
      });
    } else {
      console.error(`[${context}] 未知错误:`, error);
    }
  },
};
