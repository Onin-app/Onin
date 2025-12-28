import { 
  http, 
  fs, 
  clipboard, 
  dialog, 
  errorCode, 
  errorUtils 
} from '../src/index';
import { withRetry, isRetryableError, getRetryDelay } from '../src/utils/retry';

/**
 * 改进后的错误处理示例
 * 展示如何使用更精确的错误码和更好的错误处理模式
 */

// HTTP 错误处理 - 使用精确的状态码错误
export const httpExamples = {
  async fetchUserData(userId: string) {
    try {
      const response = await http.get(`https://api.example.com/users/${userId}`);
      return response.body;
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.http.HTTP_ERROR:
            const status = error.context?.status;
            switch (status) {
              case 401:
                throw new Error('请先登录');
              case 403:
                throw new Error('没有权限访问此用户信息');
              case 404:
                throw new Error('用户不存在');
              case 429:
                throw new Error('请求过于频繁，请稍后重试');
              case 500:
                throw new Error('服务器内部错误，请联系管理员');
              default:
                throw new Error(`HTTP 错误 ${status}: ${error.message}`);
            }
            break;
          case errorCode.http.TIMEOUT:
            throw new Error('请求超时，请检查网络连接');
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('需要在插件配置中添加 API 访问权限');
          default:
            throw new Error(`获取用户数据失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async uploadFile(file: File) {
    try {
      const formData = new FormData();
      formData.append('file', file);
      
      const response = await http.post('https://api.example.com/upload', formData, {
        timeout: 60000 // 上传文件需要更长的超时时间
      });
      
      return response.body;
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.http.HTTP_ERROR)) {
          const status = error.context?.status;
          if (status === 400 || status === 409) {
            throw new Error('文件格式不正确或文件已存在');
          }
          
          if (status === 429) {
            // 可以从上下文中获取重试时间
            const retryAfter = error.context?.retryAfter || 60;
            throw new Error(`上传频率限制，请 ${retryAfter} 秒后重试`);
          }
        }
      }
      throw error;
    }
  }
};

// 文件系统错误处理 - 使用更精确的文件系统错误
export const fsExamples = {
  async readConfigWithFallback(configPath: string) {
    try {
      const content = await fs.readFile(configPath);
      return JSON.parse(content);
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.fs.FILE_NOT_FOUND:
            console.log('配置文件不存在，创建默认配置');
            const defaultConfig = { version: '1.0.0', settings: {} };
            await fsExamples.saveConfig(configPath, defaultConfig);
            return defaultConfig;
          
          case errorCode.fs.FILE_ACCESS_DENIED:
            throw new Error('无法读取配置文件，请检查文件权限');
          
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('插件没有文件系统访问权限');
          
          default:
            throw new Error(`读取配置失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async saveConfig(configPath: string, config: any) {
    try {
      await fs.writeFile(configPath, JSON.stringify(config, null, 2));
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.fs.DISK_FULL:
            throw new Error('磁盘空间不足，无法保存配置');
          
          case errorCode.fs.FILE_ACCESS_DENIED:
            throw new Error('无法写入配置文件，请检查文件权限');
          
          case errorCode.fs.INVALID_PATH:
            throw new Error('配置文件路径无效');
          
          default:
            throw new Error(`保存配置失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async backupFile(sourcePath: string, backupPath: string) {
    try {
      await fs.copyFile(sourcePath, backupPath);
      console.log('文件备份成功');
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.fs.FILE_ALREADY_EXISTS)) {
          // 备份文件已存在，询问是否覆盖
          const shouldOverwrite = await dialog.showConfirm({
            title: '备份文件已存在',
            message: '是否覆盖现有备份文件？'
          });
          
          if (shouldOverwrite) {
            await fs.deleteFile(backupPath);
            await fs.copyFile(sourcePath, backupPath);
            console.log('备份文件已覆盖');
          }
          return;
        }
      }
      throw error;
    }
  }
};

// 剪贴板错误处理 - 使用更精确的剪贴板错误
export const clipboardExamples = {
  async smartCopy(text: string) {
    try {
      await clipboard.writeText(text);
      console.log('文本已复制到剪贴板');
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.clipboard.UNAVAILABLE:
            console.warn('剪贴板不可用，文本已保存到临时存储');
            // 可以保存到本地存储作为备选方案
            localStorage.setItem('clipboard_backup', text);
            break;
          
          case errorCode.clipboard.ACCESS_DENIED:
            throw new Error('剪贴板访问被拒绝，请检查权限设置');
          
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('插件没有剪贴板访问权限');
          
          default:
            throw new Error(`复制失败: ${error.message}`);
        }
      }
    }
  },

  async smartPaste(): Promise<string> {
    try {
      const text = await clipboard.readText();
      return text || '';
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.clipboard.EMPTY:
            console.log('剪贴板为空');
            return '';
          
          case errorCode.clipboard.FORMAT_UNSUPPORTED:
            console.warn('剪贴板内容格式不支持，尝试获取纯文本');
            // 可以尝试其他格式或返回空字符串
            return '';
          
          case errorCode.clipboard.UNAVAILABLE:
            console.warn('剪贴板不可用，尝试从临时存储恢复');
            return localStorage.getItem('clipboard_backup') || '';
          
          default:
            throw new Error(`粘贴失败: ${error.message}`);
        }
      }
      throw error;
    }
  }
};

// 对话框错误处理 - 使用更精确的对话框错误
export const dialogExamples = {
  async safeConfirm(message: string, title?: string): Promise<boolean> {
    try {
      return await dialog.showConfirm({ message, title });
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        switch (error.code) {
          case errorCode.dialog.CANCELLED:
            return false; // 用户取消等同于选择"否"
          
          case errorCode.dialog.UNAVAILABLE:
            console.warn('对话框不可用，使用控制台确认');
            return confirm(message); // 降级到浏览器原生确认框
          
          case errorCode.dialog.INVALID_OPTIONS:
            throw new Error('对话框选项无效');
          
          case errorCode.common.PERMISSION_DENIED:
            throw new Error('插件没有对话框访问权限');
          
          default:
            throw new Error(`显示确认对话框失败: ${error.message}`);
        }
      }
      throw error;
    }
  },

  async selectFileWithFallback(filters?: any[]): Promise<string | null> {
    try {
      const result = await dialog.showOpen({
        title: '选择文件',
        filters,
        multiple: false
      });
      return result as string | null;
    } catch (error) {
      if (errorUtils.isPluginError(error)) {
        if (errorUtils.isErrorCode(error, errorCode.dialog.CANCELLED)) {
          return null; // 用户取消选择
        }
        
        if (errorUtils.isErrorCode(error, errorCode.dialog.UNAVAILABLE)) {
          console.warn('文件对话框不可用，请手动输入文件路径');
          // 可以显示一个输入框让用户手动输入路径
          const path = prompt('请输入文件路径:');
          return path;
        }
      }
      throw error;
    }
  }
};

// 统一的错误处理工具
export const errorHandling = {
  /**
   * 获取用户友好的错误消息
   */
  getUserFriendlyMessage(error: unknown): string {
    if (errorUtils.isPluginError(error)) {
      // 根据错误码返回本地化的用户友好消息
      const errorMessages: Record<string, string> = {
        [errorCode.common.PERMISSION_DENIED]: '权限不足，请检查插件配置',
        [errorCode.http.HTTP_ERROR]: '请求失败',
        [errorCode.http.TIMEOUT]: '请求超时，请检查网络连接',
        [errorCode.http.NETWORK_ERROR]: '网络连接失败',
        [errorCode.fs.FILE_NOT_FOUND]: '文件不存在',
        [errorCode.fs.DISK_FULL]: '磁盘空间不足',
        [errorCode.clipboard.UNAVAILABLE]: '剪贴板不可用',
        [errorCode.dialog.CANCELLED]: '操作已取消',
      };
      
      // 对于 HTTP 错误，提供更具体的消息
      if (error.code === errorCode.http.HTTP_ERROR) {
        const status = error.context?.status;
        const httpMessages: Record<number, string> = {
          400: '请求参数错误',
          401: '身份验证失败，请重新登录',
          403: '访问被拒绝，权限不足',
          404: '请求的资源不存在',
          429: '请求过于频繁，请稍后重试',
          500: '服务器内部错误',
          502: '网关错误',
          503: '服务暂时不可用',
        };
        return httpMessages[status] || `HTTP 错误 ${status}`;
      }
      
      return errorMessages[error.code] || `操作失败: ${error.message}`;
    }
    return '发生未知错误';
  },

  /**
   * 检查错误是否可以重试
   */
  isRetryable: isRetryableError,

  /**
   * 获取建议的重试延迟时间（毫秒）
   */
  getRetryDelay,

  /**
   * 带重试的操作执行器
   */
  withRetry
};