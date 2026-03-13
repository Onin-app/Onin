import {
  http,
  fs,
  clipboard,
  dialog,
  errorCode,
  errorUtils,
} from '../src/index';

/**
 * 完整的错误处理示例
 * 展示如何在不同 API 中使用统一的错误处理
 */

// HTTP 请求错误处理
async function handleHttpErrors() {
  try {
    const response = await http.get('https://api.example.com/data');
    console.log('HTTP 请求成功:', response.body);
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.common.PERMISSION_DENIED:
          console.error('❌ 需要在 manifest.json 中添加 URL 权限');
          console.error('   请添加:', error.context?.url);
          break;
        case errorCode.http.TIMEOUT:
          console.error('⏰ 请求超时，请检查网络连接');
          break;
        case errorCode.http.HTTP_ERROR:
          const status = error.context?.status;
          if (status === 404) {
            console.error('🔍 资源未找到');
          } else if (status >= 500) {
            console.error('🔥 服务器错误');
          } else {
            console.error(`📡 HTTP 错误: ${error.message}`);
          }
          break;
        case errorCode.http.NETWORK_ERROR:
          console.error('🌐 网络连接失败');
          break;
        default:
          console.error('❓ 未知错误:', error.message);
      }
    }
  }
}

// 文件系统错误处理
async function handleFileSystemErrors() {
  try {
    const content = await fs.readFile('config.json');
    console.log('文件读取成功:', content);
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.fs.FILE_NOT_FOUND:
          console.error('📄 文件不存在:', error.context?.path);
          // 可以尝试创建默认文件
          break;
        case errorCode.fs.FILE_ACCESS_DENIED:
          console.error('🔒 文件访问被拒绝:', error.context?.path);
          break;
        case errorCode.common.PERMISSION_DENIED:
          console.error('❌ 没有文件系统权限');
          break;
        default:
          console.error('💾 文件系统错误:', error.message);
      }
    }
  }
}

// 剪贴板错误处理
async function handleClipboardErrors() {
  try {
    const text = await clipboard.readText();
    console.log('剪贴板内容:', text);
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.clipboard.UNAVAILABLE:
          console.error('📋 剪贴板不可用');
          break;
        case errorCode.common.PERMISSION_DENIED:
          console.error('❌ 没有剪贴板权限');
          break;
        default:
          console.error('📋 剪贴板错误:', error.message);
      }
    }
  }
}

// 对话框错误处理
async function handleDialogErrors() {
  try {
    const result = await dialog.showConfirm({
      title: '确认',
      message: '是否继续？',
    });
    console.log('用户选择:', result ? '确认' : '取消');
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.dialog.CANCELLED:
          console.log('👤 用户取消了对话框');
          break;
        case errorCode.common.PERMISSION_DENIED:
          console.error('❌ 没有对话框权限');
          break;
        default:
          console.error('💬 对话框错误:', error.message);
      }
    }
  }
}

// 统一的错误处理函数
function handleAnyPluginError(error: unknown, operation: string) {
  if (errorUtils.isPluginError(error)) {
    console.error(`操作 "${operation}" 失败:`);
    console.error(`错误码: ${error.code}`);
    console.error(`错误信息: ${error.message}`);

    if (error.context) {
      console.error('上下文信息:', error.context);
    }

    // 根据错误类型提供建议
    switch (error.code) {
      case errorCode.common.PERMISSION_DENIED:
        console.error('💡 建议: 检查 manifest.json 中的权限配置');
        break;
      case errorCode.http.TIMEOUT:
        console.error('💡 建议: 检查网络连接或增加超时时间');
        break;
      case errorCode.fs.FILE_NOT_FOUND:
        console.error('💡 建议: 检查文件路径是否正确');
        break;
    }
  } else {
    console.error(`操作 "${operation}" 发生未知错误:`, error);
  }
}

// 使用便捷的检查方法
async function useConvenientChecking() {
  try {
    await http.get('https://api.example.com/data');
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      // 检查是否为网络相关错误
      if (
        errorUtils.isOneOfErrorCodes(error, [
          errorCode.http.NETWORK_ERROR,
          errorCode.http.TIMEOUT,
        ])
      ) {
        console.error('🌐 网络问题，请稍后重试');
        return;
      }

      // 检查是否为权限错误
      if (errorUtils.isErrorCode(error, errorCode.common.PERMISSION_DENIED)) {
        console.error('🔐 权限不足，请检查配置');
        return;
      }

      // 其他错误
      handleAnyPluginError(error, 'HTTP 请求');
    }
  }
}

// 导出示例函数
export {
  handleHttpErrors,
  handleFileSystemErrors,
  handleClipboardErrors,
  handleDialogErrors,
  handleAnyPluginError,
  useConvenientChecking,
};
