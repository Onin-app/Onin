import { http, fs, errorCode, errorUtils } from '../src/index';
import { withRetry, createRetryWrapper, retry } from '../src/utils/retry';

/**
 * 重试机制使用示例
 * 展示如何使用改进的重试机制，避免 this 指向问题
 */

// 基本重试示例
export async function basicRetryExample() {
  console.log('=== 基本重试示例 ===');
  
  try {
    const result = await withRetry(async () => {
      const response = await http.get('https://api.example.com/data');
      return response.body;
    }, {
      maxRetries: 3,
      baseDelay: 1000
    });
    
    console.log('请求成功:', result);
  } catch (error) {
    console.error('请求最终失败:', errorUtils.isPluginError(error) ? error.code : error);
  }
}

// 指数退避重试示例
export async function exponentialBackoffExample() {
  console.log('\n=== 指数退避重试示例 ===');
  
  try {
    const result = await withRetry(async () => {
      const response = await http.get('https://api.example.com/heavy-operation');
      return response.body;
    }, {
      maxRetries: 5,
      baseDelay: 1000,
      exponentialBackoff: true,
      maxDelay: 30000,
      onRetry: (error, attempt, delay) => {
        console.log(`第 ${attempt} 次重试，延迟 ${delay}ms`);
        if (errorUtils.isPluginError(error)) {
          console.log(`错误: ${error.code} - ${error.message}`);
        }
      }
    });
    
    console.log('操作成功:', result);
  } catch (error) {
    console.error('操作最终失败:', error);
  }
}

// 自定义重试条件示例
export async function customRetryConditionExample() {
  console.log('\n=== 自定义重试条件示例 ===');
  
  try {
    const result = await withRetry(async () => {
      const response = await http.get('https://api.example.com/data');
      return response.body;
    }, {
      maxRetries: 3,
      shouldRetry: (error) => {
        // 自定义重试条件：只重试网络错误和超时
        if (errorUtils.isPluginError(error)) {
          return errorUtils.isOneOfErrorCodes(error, [
            errorCode.http.NETWORK_ERROR,
            errorCode.http.TIMEOUT
          ]);
        }
        return false;
      },
      getDelay: (error, attempt) => {
        // 自定义延迟计算
        if (errorUtils.isPluginError(error)) {
          switch (error.code) {
            case errorCode.http.TIMEOUT:
              return 2000 * attempt; // 超时错误：2秒 * 尝试次数
            case errorCode.http.NETWORK_ERROR:
              return 5000; // 网络错误：固定5秒
            default:
              return 1000;
          }
        }
        return 1000;
      }
    });
    
    console.log('请求成功:', result);
  } catch (error) {
    console.error('请求失败:', error);
  }
}

// 函数包装器示例
export async function functionWrapperExample() {
  console.log('\n=== 函数包装器示例 ===');
  
  // 创建带重试的HTTP请求函数
  const retryableHttpGet = createRetryWrapper(
    async (url: string) => {
      const response = await http.get(url);
      return response.body;
    },
    {
      maxRetries: 3,
      exponentialBackoff: true,
      baseDelay: 1000
    }
  );
  
  try {
    const result = await retryableHttpGet('https://api.example.com/data');
    console.log('请求成功:', result);
  } catch (error) {
    console.error('请求失败:', error);
  }
}

// 文件操作重试示例
export async function fileOperationRetryExample() {
  console.log('\n=== 文件操作重试示例 ===');
  
  try {
    await withRetry(async () => {
      await fs.writeFile('config.json', JSON.stringify({ version: '1.0.0' }));
    }, {
      maxRetries: 3,
      shouldRetry: (error) => {
        // 只重试特定的文件系统错误
        if (errorUtils.isPluginError(error)) {
          return errorUtils.isOneOfErrorCodes(error, [
            errorCode.fs.DISK_FULL,
            errorCode.common.PERMISSION_DENIED
          ]);
        }
        return false;
      },
      onRetry: (error, attempt, delay) => {
        if (errorUtils.isPluginError(error)) {
          if (error.code === errorCode.fs.DISK_FULL) {
            console.log('磁盘空间不足，尝试清理临时文件...');
            // 这里可以添加清理逻辑
          } else if (error.code === errorCode.common.PERMISSION_DENIED) {
            console.log('权限不足，等待权限恢复...');
          }
        }
        console.log(`${delay}ms 后重试 (${attempt}/3)`);
      }
    });
    
    console.log('文件写入成功');
  } catch (error) {
    console.error('文件写入失败:', error);
  }
}

// 批量操作重试示例
export async function batchOperationRetryExample() {
  console.log('\n=== 批量操作重试示例 ===');
  
  const urls = [
    'https://api.example.com/users/1',
    'https://api.example.com/users/2',
    'https://api.example.com/users/3'
  ];
  
  const results = await Promise.allSettled(
    urls.map(url => 
      withRetry(async () => {
        const response = await http.get(url);
        return response.body;
      }, {
        maxRetries: 2,
        baseDelay: 500
      })
    )
  );
  
  results.forEach((result, index) => {
    if (result.status === 'fulfilled') {
      console.log(`用户 ${index + 1} 数据获取成功:`, result.value);
    } else {
      console.error(`用户 ${index + 1} 数据获取失败:`, result.reason);
    }
  });
}

// 智能重试示例 - 根据错误类型调整策略
export async function smartRetryExample() {
  console.log('\n=== 智能重试示例 ===');
  
  try {
    const result = await withRetry(async () => {
      const response = await http.post('https://api.example.com/upload', {
        data: 'large file content'
      });
      return response.body;
    }, {
      maxRetries: 5,
      shouldRetry: (error) => {
        if (errorUtils.isPluginError(error)) {
          // 不同错误类型的重试策略
          switch (error.code) {
            case errorCode.http.TOO_MANY_REQUESTS:
              return true; // 限流错误总是重试
            case errorCode.http.TIMEOUT:
              return true; // 超时错误重试
            case errorCode.http.INTERNAL_SERVER_ERROR:
              return true; // 服务器错误重试
            case errorCode.http.BAD_REQUEST:
            case errorCode.http.UNAUTHORIZED:
            case errorCode.http.FORBIDDEN:
              return false; // 客户端错误不重试
            default:
              return retry.isRetryableError(error);
          }
        }
        return false;
      },
      getDelay: (error, attempt) => {
        if (errorUtils.isPluginError(error)) {
          switch (error.code) {
            case errorCode.http.TOO_MANY_REQUESTS:
              // 限流错误：使用服务器建议的重试时间
              const retryAfter = error.context?.retryAfter || 60;
              return retryAfter * 1000;
            case errorCode.http.TIMEOUT:
              // 超时错误：逐渐增加延迟
              return Math.min(5000 * attempt, 30000);
            case errorCode.http.INTERNAL_SERVER_ERROR:
              // 服务器错误：指数退避
              return Math.min(1000 * Math.pow(2, attempt - 1), 60000);
            default:
              return 3000 * attempt;
          }
        }
        return 3000 * attempt;
      },
      onRetry: (error, attempt, delay) => {
        if (errorUtils.isPluginError(error)) {
          console.log(`智能重试策略: ${error.code}`);
          console.log(`第 ${attempt} 次重试，延迟 ${delay}ms`);
          
          // 根据错误类型提供不同的提示
          switch (error.code) {
            case errorCode.http.TOO_MANY_REQUESTS:
              console.log('💡 遇到限流，等待服务器建议的时间');
              break;
            case errorCode.http.TIMEOUT:
              console.log('⏰ 请求超时，增加延迟时间');
              break;
            case errorCode.http.INTERNAL_SERVER_ERROR:
              console.log('🔥 服务器错误，使用指数退避策略');
              break;
          }
        }
      }
    });
    
    console.log('上传成功:', result);
  } catch (error) {
    console.error('上传最终失败:', error);
  }
}

// 运行所有示例
export async function runAllRetryExamples() {
  console.log('🔄 开始重试机制示例...\n');
  
  await basicRetryExample();
  await exponentialBackoffExample();
  await customRetryConditionExample();
  await functionWrapperExample();
  await fileOperationRetryExample();
  await batchOperationRetryExample();
  await smartRetryExample();
  
  console.log('\n✨ 重试机制示例完成！');
}

// 如果直接运行此文件，执行示例
if (typeof window === 'undefined' && typeof process !== 'undefined') {
  runAllRetryExamples().catch(console.error);
}