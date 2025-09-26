# 错误处理系统重构总结

## 问题分析

原有的错误处理系统存在以下问题：

### 1. API 表面积过大
```typescript
// 旧版本需要导出大量检查函数
export function isBaizeRequestError(error: any): error is BaizeRequestError;
export function isPermissionDeniedError(error: any): error is PermissionDeniedError;
export function isTimeoutError(error: any): error is TimeoutError;
export function isNetworkError(error: any): error is NetworkError;
export function isHttpError(error: any): error is HttpError;
export function isFileSystemError(error: any): error is FileSystemError;
export function isClipboardError(error: any): error is ClipboardError;
export function isDialogError(error: any): error is DialogError;
// ... 还有更多
```

### 2. 不符合函数式编程原则
- 使用了 class 而不是纯函数和接口
- 错误码没有按命名空间组织
- 缺乏函数式的组合性

### 2. 使用复杂
```typescript
// 用户需要记住每种错误的检查函数
if (isPermissionDeniedError(error)) {
  // 处理权限错误
} else if (isTimeoutError(error)) {
  // 处理超时错误
} else if (isHttpError(error)) {
  // 处理 HTTP 错误
} else if (isNetworkError(error)) {
  // 处理网络错误
}
```

### 3. 维护困难
- 每增加一种错误类型就要添加对应的接口、工厂函数和检查函数
- 不同模块的错误处理方式不统一
- 错误信息缺乏结构化的上下文

## 解决方案

### 1. 函数式错误接口
```typescript
export interface PluginError extends Error {
  readonly name: 'PluginError';
  readonly code: ErrorCode;
  readonly context?: Record<string, any>;
}
```

### 2. 命名空间化的错误码
```typescript
export const errorCode = {
  common: {
    UNKNOWN: 'COMMON_UNKNOWN',
    PERMISSION_DENIED: 'COMMON_PERMISSION_DENIED',
  },
  http: {
    TIMEOUT: 'HTTP_TIMEOUT',
    HTTP_ERROR: 'HTTP_HTTP_ERROR',
  },
  fs: {
    FILE_NOT_FOUND: 'FS_FILE_NOT_FOUND',
  },
  // ...
} as const;
```

### 3. 函数式工具集
```typescript
// 错误检查工具
export const errorUtils = {
  isPluginError: (error: any): error is PluginError => { ... },
  isErrorCode: (error: any, code: ErrorCode): boolean => { ... },
  isOneOfErrorCodes: (error: any, codes: ErrorCode[]): boolean => { ... },
};

// 命名空间化的错误工厂
export const createError = {
  http: {
    timeout: (url: string, timeout: number) => createPluginError(...),
    httpError: (status: number, statusText: string) => createPluginError(...),
  },
  fs: {
    fileNotFound: (path: string) => createPluginError(...),
  },
  // ...
};
```

## 改进效果

### 1. API 表面积减少 80%
- **旧版本**: 20+ 个错误检查函数
- **新版本**: 1 个错误检查函数 + 错误码枚举

### 2. 使用更简洁
```typescript
// 新版本 - 函数式 + 命名空间
if (errorUtils.isPluginError(error)) {
  switch (error.code) {
    case errorCode.common.PERMISSION_DENIED:
    case errorCode.http.TIMEOUT:
    case errorCode.http.HTTP_ERROR:
      // 统一处理
  }
}
```

### 3. 更好的类型安全和函数式组合
```typescript
// 编译时检查错误码
if (errorUtils.isErrorCode(error, errorCode.http.TIMEOUT)) {
  // TypeScript 知道这是超时错误
  console.log(error.context?.url); // 类型安全
}

// 函数式组合
const isNetworkError = (error: any) => 
  errorUtils.isOneOfErrorCodes(error, [
    errorCode.http.NETWORK_ERROR,
    errorCode.http.TIMEOUT
  ]);
```

### 4. 更丰富的上下文信息
```typescript
// 错误可以携带结构化的上下文
throw createError.httpError(404, 'Not Found', {
  url: 'https://api.example.com/data',
  method: 'GET',
  headers: { ... }
});
```

### 5. 函数式的检查方法
```typescript
// 检查单个错误码
if (errorUtils.isErrorCode(error, errorCode.common.PERMISSION_DENIED)) { ... }

// 检查多个错误码
if (errorUtils.isOneOfErrorCodes(error, [
  errorCode.http.TIMEOUT, 
  errorCode.http.NETWORK_ERROR
])) { ... }

// 可组合的检查函数
const isRetryableError = (error: any) => 
  errorUtils.isOneOfErrorCodes(error, [
    errorCode.http.TIMEOUT,
    errorCode.http.NETWORK_ERROR
  ]);
```

## 向后兼容性

虽然这是一个破坏性变更，但迁移成本很低：

1. 将所有 `isXxxError()` 替换为 `errorUtils.isPluginError()`
2. 使用 `error.code` 和 `errorCode` 命名空间进行类型判断
3. 通过 `error.context` 访问额外信息
4. 使用函数式的检查工具进行错误处理

## 文件变更

### 新增文件
- `plugins-sdk/src/types/errors.ts` - 统一的错误系统
- `plugins-sdk/ERROR_HANDLING.md` - 错误处理指南
- `plugins-sdk/examples/error-handling.ts` - 使用示例

### 修改文件
- `plugins-sdk/src/api/request.ts` - 重构 HTTP 错误处理
- `plugins-sdk/src/api/fs.ts` - 重构文件系统错误处理
- `plugins-sdk/src/api/clipboard.ts` - 重构剪贴板错误处理
- `plugins-sdk/src/api/dialog.ts` - 重构对话框错误处理
- `plugins-sdk/src/index.ts` - 导出新的错误系统

## 总结

这次重构大大简化了错误处理系统，采用函数式编程原则，提供了更好的开发体验：

- ✅ **简洁**: API 表面积减少 80%
- ✅ **函数式**: 纯函数、无副作用、可组合
- ✅ **命名空间**: 错误码按功能模块组织
- ✅ **统一**: 所有模块使用相同的错误处理方式
- ✅ **类型安全**: 更好的 TypeScript 支持
- ✅ **可扩展**: 易于添加新的错误类型
- ✅ **信息丰富**: 结构化的错误上下文
- ✅ **易用**: 函数式的检查工具

用户现在只需要学习一套函数式的错误处理 API，就能处理所有插件相关的错误，同时享受函数式编程的所有优势。