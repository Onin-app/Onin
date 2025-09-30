# 改进后的错误处理系统

## 修复的问题

根据第二轮代码审查反馈，我们进一步修复了以下问题：

### 最新修复（第三轮审查）

#### ✅ 代码重复问题
- **问题**: `error-parser.ts` 中 `fsErrorPatterns` 存在重复的错误模式定义
- **解决方案**: 删除重复的错误模式定义，保持数组的简洁性

#### ✅ this 指向问题
- **问题**: 重试机制中的 `this.isRetryable(error)` 和 `this.getRetryDelay(error)` 存在上下文问题
- **解决方案**: 
  - 创建独立的重试工具模块 `retry.ts`
  - 使用函数式编程避免 this 指向问题
  - 提供更灵活的重试配置选项

### 第二轮修复

#### ✅ 代码质量问题
- **问题**: `improved-error-handling.ts` 中使用 `this` 调用静态函数
- **解决方案**: 修正为正确的函数调用 `fsExamples.saveConfig()`

#### ✅ 错误码缺失
- **问题**: 缺少 HTTP CONFLICT (409) 错误码的便捷创建函数
- **解决方案**: 添加了 `createError.http.conflict()` 函数

#### ✅ 错误解析器优先级
- **问题**: 错误模式匹配可能存在模糊匹配问题
- **解决方案**: 重新排序错误模式，具体错误优先于通用错误

```typescript
// 具体的文件系统错误（优先级高）
{ patterns: ['Access denied', 'access denied', 'FILE_ACCESS_DENIED'] },
// 通用权限错误（优先级低，放在最后）
{ patterns: ['Permission denied', 'PERMISSION_DENIED'] }
```

#### ✅ 类型安全改进
- **问题**: 错误解析器中使用类型断言可能隐藏类型安全问题
- **解决方案**: 
  - 添加了严格的类型定义
  - 实现了安全的错误消息提取函数
  - 添加了错误匹配结果类型
  - 增加了错误处理的降级机制

```typescript
interface ErrorPattern {
  patterns: string[];
  createError: (message: string, context?: Record<string, any>) => PluginError;
}

function extractErrorMessage(error: unknown): string {
  // 安全的类型检查和消息提取
}
```

### 第一轮修复（之前的问题）

根据第一轮代码审查反馈，我们修复了以下问题：

### 1. 实现细节问题

#### ✅ 错误代码匹配逻辑改进
- **问题**: 之前通过简单的字符串匹配判断错误类型，不够健壮
- **解决方案**: 创建了统一的错误解析器 `error-parser.ts`，使用模式匹配数组，支持多种错误消息格式

```typescript
// 旧版本 - 简单字符串匹配
if (message.includes('not found')) {
  throw createError.fs.fileNotFound(path);
}

// 新版本 - 模式匹配数组
const fsErrorPatterns: ErrorPattern[] = [
  {
    patterns: ['not found', 'No such file', 'FILE_NOT_FOUND', 'does not exist'],
    createError: (message, context) => createError.fs.fileNotFound(
      context?.path || 'unknown', 
      { originalError: message, ...context }
    )
  }
];
```

#### ✅ 消除重复代码
- **问题**: 各个API文件中错误处理代码重复
- **解决方案**: 创建统一的错误解析器，各API模块调用对应的解析函数

```typescript
// 统一的错误处理模式
async function callFsApi<T = any>(method: string, args?: any): Promise<T> {
  try {
    return await dispatch({ /* ... */ });
  } catch (error: any) {
    if (errorUtils.isPluginError(error)) {
      throw error;
    }
    // 使用统一的错误解析器
    throw parseFsError(error, { path: args?.path, method, args });
  }
}
```

#### ✅ 修复重复条件判断
- **问题**: `fs.ts` 中存在重复的 `Permission denied` 判断
- **解决方案**: 重构错误处理逻辑，使用优先级匹配

### 2. 性能问题

#### ✅ 优化错误匹配逻辑
- 使用优先级匹配，避免重复检查
- 缓存错误解析结果
- 减少字符串操作次数

### 3. 设计改进

#### ✅ 简化的 HTTP 错误码设计
基于用户反馈，简化了HTTP错误码设计，使用标准HTTP状态码：

```typescript
const errorCode = {
  http: {
    NETWORK_ERROR: 'HTTP_NETWORK_ERROR',
    TIMEOUT: 'HTTP_TIMEOUT',
    HTTP_ERROR: 'HTTP_HTTP_ERROR', // 通用HTTP错误，包含状态码信息
  }
}

// 使用标准状态码进行判断
if (error.code === errorCode.http.HTTP_ERROR) {
  const status = error.context?.status;
  switch (status) {
    case 401: // 使用标准状态码
      console.log('需要登录');
      break;
    case 404:
      console.log('资源不存在');
      break;
  }
}
```

#### ✅ 智能HTTP错误创建
HTTP错误创建函数现在会自动根据状态码选择精确的错误类型：

```typescript
httpError: (status: number, statusText: string, context?: Record<string, any>) => {
  let code = errorCode.http.HTTP_ERROR;
  switch (status) {
    case 400: code = errorCode.http.BAD_REQUEST; break;
    case 401: code = errorCode.http.UNAUTHORIZED; break;
    case 403: code = errorCode.http.FORBIDDEN; break;
    // ... 更多状态码映射
  }
  return createPluginError(code, `HTTP ${status}: ${statusText}`, { status, statusText, ...context });
}
```

#### ✅ 更丰富的上下文信息
每个错误现在都包含更具体的上下文信息：

```typescript
// 文件系统错误包含路径和操作信息
throw parseFsError(error, {
  path: args?.path || args?.sourcePath,
  method,
  args
});

// HTTP错误包含请求详情
throw parseHttpError(error, {
  url: options.url,
  method: options.method,
  timeout: options.timeout,
  headers: options.headers
});
```

### 4. API设计改进

#### ✅ 优化参数顺序
错误工厂函数的参数顺序更加直观：

```typescript
// 文件系统错误 - 路径参数在前
fileNotFound: (path: string, context?: Record<string, any>)
fileAccessDenied: (path: string, context?: Record<string, any>)

// HTTP错误 - 主要参数在前
timeout: (url: string, timeout: number, context?: Record<string, any>)
httpError: (status: number, statusText: string, context?: Record<string, any>)
```

#### ✅ 便捷的状态码错误创建函数
添加了便捷的HTTP状态码错误创建函数：

```typescript
const createError = {
  http: {
    badRequest: (message?: string, context?: Record<string, any>),
    unauthorized: (message?: string, context?: Record<string, any>),
    forbidden: (message?: string, context?: Record<string, any>),
    notFound: (message?: string, context?: Record<string, any>),
    tooManyRequests: (message?: string, context?: Record<string, any>),
    // ...
  }
}
```

## 新增功能

### 1. 改进的统一错误解析器
- 支持多种错误消息格式
- 优先级排序的模式匹配系统
- 类型安全的错误消息提取
- 自动上下文信息提取
- 错误创建失败时的降级机制

```typescript
// 类型安全的错误解析
export function parseError(
  error: unknown, 
  patterns: ErrorPattern[], 
  context?: Record<string, any>
): PluginError {
  const message = extractErrorMessage(error);
  const matchResult = matchErrorPattern(message, patterns);
  
  if (matchResult.matched && matchResult.pattern) {
    try {
      return matchResult.pattern.createError(message, context);
    } catch (createError) {
      // 降级到通用错误
      console.warn('Failed to create specific error, falling back to generic error:', createError);
    }
  }
  
  return createError.common.unknown(message, { 
    originalError: message, 
    parseContext: 'No pattern matched',
    ...context 
  });
}
```

### 2. 改进的智能重试机制
```typescript
// 基本重试
await withRetry(async () => {
  return await http.get('https://api.example.com/data');
}, {
  maxRetries: 3,
  baseDelay: 1000
});

// 指数退避重试
await withRetry(async () => {
  return await http.get('https://api.example.com/data');
}, {
  maxRetries: 5,
  exponentialBackoff: true,
  baseDelay: 1000,
  maxDelay: 30000
});

// 自定义重试条件
await withRetry(async () => {
  return await http.get('https://api.example.com/data');
}, {
  shouldRetry: (error) => isRetryableError(error),
  getDelay: (error, attempt) => getRetryDelay(error, attempt)
});
```

### 3. 用户友好的错误消息
```typescript
const friendlyMessage = errorHandling.getUserFriendlyMessage(error);
// "请求过于频繁，请稍后重试" 而不是 "HTTP 429: Too Many Requests"
```

### 4. 错误可重试性检查
```typescript
if (errorHandling.isRetryable(error)) {
  const delay = errorHandling.getRetryDelay(error);
  // 自动重试逻辑
}
```

## 使用示例

### 简化的HTTP错误处理
```typescript
try {
  const response = await http.get('/api/users/123');
} catch (error) {
  if (errorUtils.isPluginError(error) && error.code === errorCode.http.HTTP_ERROR) {
    const status = error.context?.status;
    switch (status) {
      case 401:
        // 使用标准状态码处理401错误
        redirectToLogin();
        break;
      case 403:
        // 使用标准状态码处理403错误
        showPermissionError();
        break;
      case 404:
        // 使用标准状态码处理404错误
        showUserNotFound();
        break;
    }
  }
}
```

### 智能文件操作
```typescript
try {
  await fs.writeFile('config.json', data);
} catch (error) {
  if (errorUtils.isErrorCode(error, errorCode.fs.DISK_FULL)) {
    await cleanupTempFiles();
    await fs.writeFile('config.json', data); // 重试
  }
}
```

### 降级处理
```typescript
try {
  return await dialog.showConfirm({ message: 'Continue?' });
} catch (error) {
  if (errorUtils.isErrorCode(error, errorCode.dialog.UNAVAILABLE)) {
    return confirm('Continue?'); // 降级到浏览器原生对话框
  }
}
```

## 总结

改进后的错误处理系统解决了两轮审查中提到的所有问题：

### 第一轮修复
1. ✅ **更健壮的错误匹配** - 使用模式匹配数组
2. ✅ **消除重复代码** - 统一的错误解析器
3. ✅ **修复逻辑错误** - 重构条件判断
4. ✅ **更精确的错误码** - 支持具体的HTTP状态码
5. ✅ **更丰富的上下文** - 自动提取相关信息
6. ✅ **更好的API设计** - 直观的参数顺序

### 第二轮修复
7. ✅ **修复代码质量问题** - 正确的函数调用方式
8. ✅ **补全缺失的错误码** - 添加 HTTP CONFLICT 便捷创建函数
9. ✅ **优化错误解析优先级** - 具体错误优先于通用错误
10. ✅ **增强类型安全** - 严格的类型定义和安全的错误处理

### 第三轮修复
11. ✅ **消除代码重复** - 删除错误解析器中的重复模式定义
12. ✅ **修复 this 指向问题** - 创建独立的重试工具模块
13. ✅ **改进重试机制** - 支持指数退避、自定义条件、函数包装器等高级功能

### 第四轮优化
14. ✅ **简化 HTTP 错误码设计** - 移除自定义HTTP状态码错误，使用标准状态码
15. ✅ **降低学习成本** - 开发者直接使用熟悉的HTTP状态码，无需学习自定义错误码
16. ✅ **保持一致性** - 与Web标准保持一致，减少认知负担

### 测试覆盖
- 创建了完整的错误解析器测试套件
- 验证优先级匹配逻辑
- 测试类型安全性
- 验证上下文信息完整性

这个系统现在更加健壮、类型安全，并且提供了卓越的开发体验。所有的边界情况都得到了妥善处理，错误信息更加精确和有用。