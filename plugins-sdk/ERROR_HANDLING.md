# 错误处理指南

## 新的函数式错误处理设计

我们重新设计了错误处理系统，采用函数式编程和命名空间的方式，使其更加简洁和易用。

### 核心概念

1. **统一的错误接口** - 所有插件 API 错误都使用 `PluginError` 接口
2. **命名空间化的错误码** - 使用 `errorCode` 对象来组织不同类型的错误
3. **上下文信息** - 错误可以携带额外的上下文信息
4. **函数式检查工具** - 使用 `errorUtils` 命名空间提供检查函数

### 基本用法

```typescript
import { http, errorCode, errorUtils } from '@baize/plugin-sdk';

try {
  const response = await http.get('https://api.example.com/data');
  console.log(response.body);
} catch (error) {
  if (errorUtils.isPluginError(error)) {
    // 使用命名空间化的错误码进行处理
    switch (error.code) {
      case errorCode.common.PERMISSION_DENIED:
        console.error('需要在 manifest.json 中添加 URL 权限');
        break;
      case errorCode.http.TIMEOUT:
        console.error('请求超时');
        break;
      case errorCode.http.HTTP_ERROR:
        console.error(`HTTP 错误: ${error.message}`);
        // 访问上下文信息
        console.error('状态码:', error.context?.status);
        break;
      default:
        console.error('其他错误:', error.message);
    }
  }
}
```

### 便捷的检查方法

```typescript
// 检查特定错误码
if (errorUtils.isErrorCode(error, errorCode.common.PERMISSION_DENIED)) {
  // 处理权限错误
}

// 检查多个错误码
if (errorUtils.isOneOfErrorCodes(error, [
  errorCode.http.TIMEOUT, 
  errorCode.http.NETWORK_ERROR
])) {
  // 处理网络相关错误
}
```

### 错误码列表（按命名空间组织）

#### 通用错误 (`errorCode.common`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `UNKNOWN` | 未知错误 | 所有 API |
| `PERMISSION_DENIED` | 权限被拒绝 | 所有 API |
| `INVALID_ARGUMENT` | 无效参数 | 所有 API |

#### HTTP 错误 (`errorCode.http`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `NETWORK_ERROR` | 网络错误 | HTTP 请求 |
| `TIMEOUT` | 超时 | HTTP 请求 |
| `HTTP_ERROR` | HTTP 错误 | HTTP 请求 |

#### 文件系统错误 (`errorCode.fs`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `FILE_NOT_FOUND` | 文件未找到 | 文件系统 |
| `FILE_ACCESS_DENIED` | 文件访问被拒绝 | 文件系统 |
| `DIRECTORY_NOT_FOUND` | 目录未找到 | 文件系统 |
| `DISK_FULL` | 磁盘空间不足 | 文件系统 |

#### 剪贴板错误 (`errorCode.clipboard`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `UNAVAILABLE` | 剪贴板不可用 | 剪贴板 |
| `FORMAT_UNSUPPORTED` | 格式不支持 | 剪贴板 |

#### 对话框错误 (`errorCode.dialog`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `CANCELLED` | 对话框被取消 | 对话框 |
| `UNAVAILABLE` | 对话框不可用 | 对话框 |

#### 存储错误 (`errorCode.storage`)
| 错误码 | 说明 | 适用场景 |
|--------|------|----------|
| `QUOTA_EXCEEDED` | 存储配额超出 | 存储 |
| `UNAVAILABLE` | 存储不可用 | 存储 |

### 与旧版本的对比

#### 旧版本（复杂）
```typescript
// 需要导入多个检查函数
import { 
  isPermissionDeniedError,
  isTimeoutError,
  isHttpError,
  isNetworkError 
} from '@baize/plugin-sdk';

// 冗长的错误检查
if (isPermissionDeniedError(error)) {
  // ...
} else if (isTimeoutError(error)) {
  // ...
} else if (isHttpError(error)) {
  // ...
} else if (isNetworkError(error)) {
  // ...
}
```

#### 新版本（函数式 + 命名空间）
```typescript
// 只需要导入错误工具和错误码命名空间
import { errorUtils, errorCode } from '@baize/plugin-sdk';

// 简洁的错误处理
if (errorUtils.isPluginError(error)) {
  switch (error.code) {
    case errorCode.common.PERMISSION_DENIED:
    case errorCode.http.TIMEOUT:
    case errorCode.http.HTTP_ERROR:
    case errorCode.http.NETWORK_ERROR:
      // 处理各种错误
  }
}
```

### 优势

1. **API 表面积更小** - 只暴露必要的函数
2. **更易维护** - 统一的错误处理逻辑
3. **更好的类型安全** - TypeScript 支持更好
4. **更丰富的上下文** - 错误可以携带额外信息
5. **更灵活的检查** - 支持多种检查方式

### 迁移指南

如果你正在从旧版本迁移，只需要：

1. 将所有 `isXxxError()` 函数替换为 `errorUtils.isPluginError()`
2. 使用 `error.code` 和 `errorCode` 命名空间进行错误类型判断
3. 通过 `error.context` 访问额外的错误信息
4. 使用 `errorUtils.isErrorCode()` 和 `errorUtils.isOneOfErrorCodes()` 进行便捷检查

### 函数式编程优势

1. **无副作用** - 所有错误处理函数都是纯函数
2. **可组合** - 错误检查函数可以轻松组合使用
3. **命名空间隔离** - 不同类型的错误码按功能分组
4. **类型安全** - TypeScript 提供完整的类型检查