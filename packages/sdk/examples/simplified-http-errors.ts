import { http, errorCode, errorUtils } from '../src/index';

/**
 * 简化的 HTTP 错误处理示例
 * 使用标准 HTTP 状态码，而不是自定义错误码
 */

export async function simplifiedHttpErrorHandling() {
  try {
    const response = await http.get('https://api.example.com/users/123');
    return response.body;
  } catch (error) {
    if (errorUtils.isPluginError(error)) {
      switch (error.code) {
        case errorCode.http.HTTP_ERROR:
          // 直接使用标准 HTTP 状态码
          const status = error.context?.status;
          switch (status) {
            case 400:
              throw new Error('请求参数错误');
            case 401:
              throw new Error('请先登录');
            case 403:
              throw new Error('没有权限访问此资源');
            case 404:
              throw new Error('用户不存在');
            case 409:
              throw new Error('资源冲突');
            case 429:
              throw new Error('请求过于频繁，请稍后重试');
            case 500:
              throw new Error('服务器内部错误');
            case 502:
              throw new Error('网关错误');
            case 503:
              throw new Error('服务暂时不可用');
            default:
              throw new Error(`HTTP 错误 ${status}: ${error.message}`);
          }
          break;

        case errorCode.http.TIMEOUT:
          throw new Error('请求超时，请检查网络连接');

        case errorCode.http.NETWORK_ERROR:
          throw new Error('网络连接失败');

        case errorCode.common.PERMISSION_DENIED:
          throw new Error('需要在插件配置中添加 API 访问权限');

        default:
          throw new Error(`请求失败: ${error.message}`);
      }
    }
    throw error;
  }
}

// 通用的 HTTP 状态码处理函数
export function getHttpErrorMessage(error: unknown): string {
  if (
    errorUtils.isPluginError(error) &&
    error.code === errorCode.http.HTTP_ERROR
  ) {
    const status = error.context?.status;

    // 使用标准的 HTTP 状态码分类
    if (status >= 400 && status < 500) {
      // 客户端错误
      const clientErrorMessages: Record<number, string> = {
        400: '请求参数错误',
        401: '身份验证失败，请重新登录',
        403: '访问被拒绝，权限不足',
        404: '请求的资源不存在',
        405: '请求方法不被允许',
        409: '资源冲突',
        429: '请求过于频繁，请稍后重试',
      };
      return clientErrorMessages[status] || `客户端错误 ${status}`;
    }

    if (status >= 500 && status < 600) {
      // 服务器错误
      const serverErrorMessages: Record<number, string> = {
        500: '服务器内部错误',
        502: '网关错误',
        503: '服务暂时不可用',
        504: '网关超时',
      };
      return serverErrorMessages[status] || `服务器错误 ${status}`;
    }

    return `HTTP 错误 ${status}`;
  }

  return '未知错误';
}

// 检查是否为特定类型的 HTTP 错误
export function isHttpStatus(error: unknown, status: number): boolean {
  return (
    errorUtils.isPluginError(error) &&
    error.code === errorCode.http.HTTP_ERROR &&
    error.context?.status === status
  );
}

// 检查是否为客户端错误 (4xx)
export function isClientError(error: unknown): boolean {
  if (
    errorUtils.isPluginError(error) &&
    error.code === errorCode.http.HTTP_ERROR
  ) {
    const status = error.context?.status;
    return status >= 400 && status < 500;
  }
  return false;
}

// 检查是否为服务器错误 (5xx)
export function isServerError(error: unknown): boolean {
  if (
    errorUtils.isPluginError(error) &&
    error.code === errorCode.http.HTTP_ERROR
  ) {
    const status = error.context?.status;
    return status >= 500 && status < 600;
  }
  return false;
}

// 检查是否为可重试的错误
export function isRetryableHttpError(error: unknown): boolean {
  if (errorUtils.isPluginError(error)) {
    switch (error.code) {
      case errorCode.http.TIMEOUT:
      case errorCode.http.NETWORK_ERROR:
        return true;

      case errorCode.http.HTTP_ERROR:
        const status = error.context?.status;
        // 只重试特定的服务器错误和限流错误
        return (
          status === 429 ||
          status === 500 ||
          status === 502 ||
          status === 503 ||
          status === 504
        );

      default:
        return false;
    }
  }
  return false;
}

// 使用示例
export async function exampleUsage() {
  try {
    const userData = await http.get('https://api.example.com/user/profile');
    console.log('用户数据:', userData.body);
  } catch (error) {
    // 简洁的错误处理
    if (isHttpStatus(error, 401)) {
      console.log('需要重新登录');
      // 跳转到登录页面
    } else if (isHttpStatus(error, 404)) {
      console.log('用户不存在');
    } else if (isClientError(error)) {
      console.log('请求错误:', getHttpErrorMessage(error));
    } else if (isServerError(error)) {
      console.log('服务器错误:', getHttpErrorMessage(error));
      if (isRetryableHttpError(error)) {
        console.log('可以重试此请求');
      }
    } else {
      console.log('其他错误:', error);
    }
  }
}

export {
  simplifiedHttpErrorHandling,
  getHttpErrorMessage,
  isHttpStatus,
  isClientError,
  isServerError,
  isRetryableHttpError,
  exampleUsage,
};
