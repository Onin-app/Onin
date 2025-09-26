# HTTP 错误处理设计决策

## 问题背景

在错误处理系统的设计过程中，我们最初为每个HTTP状态码创建了自定义的错误码，如：

```typescript
// 原始设计（已废弃）
const errorCode = {
  http: {
    BAD_REQUEST: 'HTTP_BAD_REQUEST', // 400
    UNAUTHORIZED: 'HTTP_UNAUTHORIZED', // 401
    FORBIDDEN: 'HTTP_FORBIDDEN', // 403
    NOT_FOUND: 'HTTP_NOT_FOUND', // 404
    // ... 更多自定义错误码
  }
}
```

## 设计决策

经过深入思考和用户反馈，我们决定**简化HTTP错误处理**，直接使用标准HTTP状态码。

### 最终设计

```typescript
// 简化后的设计
const errorCode = {
  http: {
    NETWORK_ERROR: 'HTTP_NETWORK_ERROR',
    TIMEOUT: 'HTTP_TIMEOUT',
    HTTP_ERROR: 'HTTP_HTTP_ERROR', // 通用HTTP错误，包含状态码信息
  }
}

// 使用方式
if (error.code === errorCode.http.HTTP_ERROR) {
  const status = error.context?.status; // 直接使用标准状态码
  switch (status) {
    case 401: // 标准HTTP状态码
      handleUnauthorized();
      break;
    case 404:
      handleNotFound();
      break;
  }
}
```

## 决策理由

### 1. 降低学习成本
- **问题**: 自定义错误码增加了开发者的学习成本
- **解决**: HTTP状态码是Web开发的标准，所有开发者都熟悉

### 2. 保持一致性
- **问题**: 自定义错误码与Web标准不一致
- **解决**: 直接使用标准状态码，与现有知识体系一致

### 3. 减少维护负担
- **问题**: 需要维护大量的自定义错误码映射
- **解决**: 只需要维护通用的HTTP_ERROR类型

### 4. 提高可读性
- **问题**: `errorCode.http.UNAUTHORIZED` vs `401`
- **解决**: `401` 更直观，开发者一眼就知道是什么错误

## 对比分析

### 自定义错误码方式（已废弃）

**优点:**
- 类型安全，编译时检查
- 统一的命名规范

**缺点:**
- 学习成本高，需要记忆自定义错误码
- 与Web标准不一致
- 维护成本高，需要为每个状态码创建对应的错误码
- API表面积大，暴露过多概念

### 标准状态码方式（当前设计）

**优点:**
- 零学习成本，开发者都熟悉HTTP状态码
- 与Web标准完全一致
- 维护成本低
- API简洁，概念清晰

**缺点:**
- 失去了编译时的状态码检查（但可以通过工具函数弥补）

## 实现细节

### 错误创建
```typescript
// 统一创建HTTP错误，包含状态码信息
createError.http.httpError(404, 'Not Found', { url: '/api/users/123' })
```

### 错误检查
```typescript
// 提供便捷的检查函数
function isHttpStatus(error: unknown, status: number): boolean {
  return errorUtils.isPluginError(error) && 
         error.code === errorCode.http.HTTP_ERROR && 
         error.context?.status === status;
}

// 使用
if (isHttpStatus(error, 404)) {
  // 处理404错误
}
```

### 错误分类
```typescript
// 提供状态码分类函数
function isClientError(error: unknown): boolean {
  if (errorUtils.isPluginError(error) && error.code === errorCode.http.HTTP_ERROR) {
    const status = error.context?.status;
    return status >= 400 && status < 500;
  }
  return false;
}
```

## 迁移指南

如果你之前使用了自定义错误码，迁移很简单：

```typescript
// 旧版本
if (errorUtils.isErrorCode(error, errorCode.http.UNAUTHORIZED)) {
  // 处理401错误
}

// 新版本
if (isHttpStatus(error, 401)) {
  // 处理401错误
}

// 或者直接检查
if (error.code === errorCode.http.HTTP_ERROR && error.context?.status === 401) {
  // 处理401错误
}
```

## 总结

这个设计决策体现了"简单胜过复杂"的原则。通过使用标准HTTP状态码，我们：

1. **降低了学习成本** - 开发者无需学习新的错误码体系
2. **提高了一致性** - 与Web标准保持一致
3. **简化了API** - 减少了概念数量和API表面积
4. **保持了功能性** - 通过工具函数提供便捷的错误检查

这是一个在简洁性和功能性之间找到平衡的优秀设计决策。