import { http } from '../src/api/request';
import { errorCode, errorUtils } from '../src/types/errors';

// 新的函数式错误处理方式 - 简洁明了
async function exampleNewErrorHandling() {
  try {
    const response = await http.get('https://api.example.com/data');
    console.log('Success:', response.body);
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      // 使用命名空间化的错误码进行处理
      switch (error.code) {
        case errorCode.common.PERMISSION_DENIED:
          console.error('需要在 manifest.json 中添加 URL 权限');
          break;
        case errorCode.http.TIMEOUT:
          console.error('请求超时，请检查网络连接');
          break;
        case errorCode.http.HTTP_ERROR:
          console.error(`HTTP 错误: ${error.message}`);
          // 可以访问上下文信息
          console.error('状态码:', error.context?.status);
          break;
        case errorCode.http.NETWORK_ERROR:
          console.error('网络错误，请检查连接');
          break;
        default:
          console.error('未知错误:', error.message);
      }
    } else {
      console.error('非插件错误:', error);
    }
  }
}

// 使用便捷的检查方法
async function exampleConvenientChecking() {
  try {
    const response = await http.get('https://api.example.com/data');
    console.log('Success:', response.body);
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      // 检查是否为特定错误
      if (errorUtils.isErrorCode(error, errorCode.common.PERMISSION_DENIED)) {
        console.error('权限被拒绝');
      } else if (
        errorUtils.isOneOfErrorCodes(error, [
          errorCode.http.TIMEOUT,
          errorCode.http.NETWORK_ERROR,
        ])
      ) {
        console.error('网络相关错误');
      } else {
        console.error('其他错误:', error.message);
      }
    }
  }
}

// 对比：旧的错误处理方式（复杂且冗长）
/*
async function exampleOldErrorHandling() {
  try {
    const response = await http.get('https://api.example.com/data');
    console.log('Success:', response.body);
  } catch (error) {
    // 需要导入并使用多个检查函数
    if (isPermissionDeniedError(error)) {
      console.error('权限被拒绝');
    } else if (isTimeoutError(error)) {
      console.error('请求超时');
    } else if (isHttpError(error)) {
      console.error('HTTP 错误');
    } else if (isNetworkError(error)) {
      console.error('网络错误');
    } else if (isOninRequestError(error)) {
      console.error('请求错误');
    } else {
      console.error('未知错误');
    }
  }
}
*/
