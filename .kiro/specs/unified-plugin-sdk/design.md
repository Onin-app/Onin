# 设计文档

## 概述

统一插件 SDK 旨在为 headless 插件和 UI 插件提供一致的 API 接口。当前系统中存在两种不同的插件执行环境：

1. **Headless 插件**：使用 Deno 运行时执行 JavaScript 代码，通过 `Deno.core.ops.op_invoke` 调用系统功能
2. **UI 插件**：在 Tauri webview 中运行，通过 `@tauri-apps/api/core` 的 `invoke` 函数调用系统功能

SDK 将抽象这些差异，提供统一的 TypeScript API，让插件开发者无需关心底层实现细节。

## 架构

### 整体架构图

```mermaid
graph TB
    subgraph "插件开发者层"
        PD[插件开发者代码]
    end
    
    subgraph "统一 SDK 层"
        SDK[createPluginSDK()]
        ED[detectEnvironment()]
        HA[createHeadlessAdapter()]
        UA[createUIAdapter()]
    end
    
    subgraph "运行时环境层"
        subgraph "Headless 环境"
            DR[Deno Runtime]
            OPS[Deno.core.ops]
        end
        
        subgraph "UI 环境"
            WV[Tauri Webview]
            API[Tauri API]
        end
    end
    
    subgraph "系统 API 层"
        RUST[Rust Backend APIs]
    end
    
    PD --> SDK
    SDK --> ED
    ED --> HA
    ED --> UA
    HA --> DR
    UA --> WV
    DR --> OPS
    WV --> API
    OPS --> RUST
    API --> RUST
```

### 设计决策

1. **环境检测策略**：通过检测全局对象的存在来判断运行环境
   - Headless 环境：检测 `Deno.core.ops` 的存在
   - UI 环境：检测 `window` 对象和 `__TAURI__` 的存在
   - 理由：这是最可靠的环境区分方式，避免了复杂的配置

2. **函数式适配器模式**：为每种环境创建返回函数对象的适配器创建函数
   - 理由：函数式编程提供更好的可测试性和组合性，避免了类的复杂性

3. **统一错误格式**：所有 API 调用返回统一的错误格式
   - 理由：简化错误处理，提供一致的开发体验

4. **Result 类型**：使用 Result 类型进行函数式错误处理
   - 理由：避免异常抛出，使错误处理更加显式和可预测

## 组件和接口

### 核心类型定义

```typescript
// 环境类型
export type PluginEnvironment = 'headless' | 'ui' | 'unknown';

// 统一的 API 响应格式
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// 通知选项接口
export interface NotificationOptions {
  title: string;
  body?: string;
}

// SDK 功能函数类型
export interface PluginSDKFunctions {
  getEnvironment: () => PluginEnvironment;
  showNotification: (options: NotificationOptions) => Promise<ApiResponse<void>>;
}
```

### 环境检测函数

```typescript
export const detectEnvironment = (): PluginEnvironment => {
  // 检测 Headless 环境
  if (typeof globalThis !== 'undefined' && 
      globalThis.Deno && 
      globalThis.Deno.core && 
      globalThis.Deno.core.ops) {
    return 'headless';
  }
  
  // 检测 UI 环境
  if (typeof window !== 'undefined' && 
      window.__TAURI__) {
    return 'ui';
  }
  
  return 'unknown';
};
```

### Headless 环境适配器函数

```typescript
export const createHeadlessAdapter = (): PluginSDKFunctions => ({
  getEnvironment: () => 'headless',
  
  showNotification: async (options: NotificationOptions): Promise<ApiResponse<void>> => {
    try {
      const result = Deno.core.ops.op_invoke('show_notification', options);
      
      if (result.error) {
        return {
          success: false,
          error: result.error
        };
      }
      
      return {
        success: true,
        data: undefined
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }
});
```

### UI 环境适配器函数

```typescript
import { invoke } from '@tauri-apps/api/core';

export const createUIAdapter = (): PluginSDKFunctions => ({
  getEnvironment: () => 'ui',
  
  showNotification: async (options: NotificationOptions): Promise<ApiResponse<void>> => {
    try {
      await invoke('show_notification', options);
      
      return {
        success: true,
        data: undefined
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }
});
```

### 主 SDK 创建函数

```typescript
export const createPluginSDK = (): PluginSDKFunctions => {
  const environment = detectEnvironment();
  
  switch (environment) {
    case 'headless':
      return createHeadlessAdapter();
    case 'ui':
      return createUIAdapter();
    default:
      throw new Error(`Unsupported environment: ${environment}`);
  }
};

// 便捷的全局 SDK 实例创建
export const sdk = createPluginSDK();
```

## 数据模型

### 通知数据模型

```typescript
export interface NotificationOptions {
  title: string;      // 必需：通知标题
  body?: string;      // 可选：通知内容
}
```

### API 响应数据模型

```typescript
export interface ApiResponse<T = any> {
  success: boolean;   // 操作是否成功
  data?: T;          // 成功时的返回数据
  error?: string;    // 失败时的错误信息
}
```

## 错误处理

### 错误分类

1. **环境检测错误**：无法识别当前运行环境
2. **参数验证错误**：传入的参数不符合要求
3. **API 调用错误**：底层 API 调用失败
4. **网络错误**：在 UI 环境中与后端通信失败

### 错误处理策略

```typescript
// 错误类型定义
export interface SDKError {
  message: string;
  code: string;
  originalError?: Error;
}

// 错误代码常量
export const ERROR_CODES = {
  UNSUPPORTED_ENVIRONMENT: 'UNSUPPORTED_ENVIRONMENT',
  INVALID_PARAMETERS: 'INVALID_PARAMETERS',
  API_CALL_FAILED: 'API_CALL_FAILED',
  NETWORK_ERROR: 'NETWORK_ERROR'
} as const;

// 错误创建函数
export const createSDKError = (
  message: string,
  code: string,
  originalError?: Error
): SDKError => ({
  message,
  code,
  originalError
});

// Result 类型用于函数式错误处理
export type Result<T, E = SDKError> = 
  | { success: true; data: T }
  | { success: false; error: E };
```

### 参数验证函数

```typescript
// 通知选项验证函数
export const validateNotificationOptions = (options: any): Result<NotificationOptions> => {
  if (!options || typeof options !== 'object') {
    return {
      success: false,
      error: createSDKError(
        'Notification options must be an object',
        ERROR_CODES.INVALID_PARAMETERS
      )
    };
  }
  
  if (!options.title || typeof options.title !== 'string') {
    return {
      success: false,
      error: createSDKError(
        'Notification title is required and must be a string',
        ERROR_CODES.INVALID_PARAMETERS
      )
    };
  }
  
  if (options.body !== undefined && typeof options.body !== 'string') {
    return {
      success: false,
      error: createSDKError(
        'Notification body must be a string if provided',
        ERROR_CODES.INVALID_PARAMETERS
      )
    };
  }
  
  return {
    success: true,
    data: {
      title: options.title,
      body: options.body
    }
  };
};

// 通用验证管道函数
export const pipe = <T, U>(
  value: T,
  ...fns: Array<(arg: T) => U>
): U => fns.reduce((acc, fn) => fn(acc as any), value as any);

// 验证组合函数
export const validateAndExecute = <T, U>(
  input: T,
  validator: (input: T) => Result<T>,
  executor: (validInput: T) => Promise<Result<U>>
): Promise<Result<U>> => {
  const validationResult = validator(input);
  
  if (!validationResult.success) {
    return Promise.resolve({
      success: false,
      error: validationResult.error
    });
  }
  
  return executor(validationResult.data);
};
```

## 测试策略

### 单元测试

1. **环境检测测试**
   - 模拟不同的全局对象组合
   - 验证环境检测的准确性

2. **适配器测试**
   - 模拟 Deno.core.ops 和 Tauri API
   - 测试错误处理和响应格式

3. **参数验证测试**
   - 测试各种无效参数组合
   - 验证错误消息的准确性

### 集成测试

1. **Headless 环境测试**
   - 在真实的 Deno 运行时中测试
   - 验证与后端 Rust API 的集成

2. **UI 环境测试**
   - 在 Tauri webview 中测试
   - 验证与 Tauri API 的集成

### 测试工具和框架

- **单元测试**：Vitest
- **类型检查**：TypeScript 编译器
- **代码覆盖率**：c8
- **端到端测试**：自定义测试插件

### 测试数据和模拟

```typescript
// 测试用的模拟环境设置函数
export const setupMockDenoEnvironment = () => {
  (globalThis as any).Deno = {
    core: {
      ops: {
        op_invoke: vi.fn()
      }
    }
  };
};

export const setupMockTauriEnvironment = () => {
  (globalThis as any).window = {
    __TAURI__: {}
  };
};

// 环境清理函数
export const cleanupMockEnvironment = () => {
  delete (globalThis as any).Deno;
  delete (globalThis as any).window;
};

// 测试用例数据
export const testNotificationOptions = {
  valid: {
    title: 'Test Notification',
    body: 'This is a test notification'
  },
  invalidTitle: {
    title: '',
    body: 'Test body'
  },
  invalidType: {
    title: 123,
    body: 'Test body'
  }
};

// 测试辅助函数
export const createMockApiResponse = <T>(
  success: boolean,
  data?: T,
  error?: string
): ApiResponse<T> => ({
  success,
  data,
  error
});
```

## 构建和分发

### 构建配置

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    lib: {
      entry: 'src/index.ts',
      name: 'UnifiedPluginSDK',
      fileName: (format) => `unified-plugin-sdk.${format}.js`,
      formats: ['es', 'umd']
    },
    rollupOptions: {
      external: ['@tauri-apps/api/core'],
      output: {
        globals: {
          '@tauri-apps/api/core': 'TauriCore'
        }
      }
    }
  }
});
```

### 包结构

```
plugins-sdk/
├── src/
│   ├── index.ts              # 主入口文件
│   ├── core/
│   │   ├── sdk.ts           # 主 SDK 创建函数
│   │   ├── environment.ts   # 环境检测函数
│   │   └── errors.ts        # 错误处理函数
│   ├── adapters/
│   │   ├── headless.ts      # Headless 适配器函数
│   │   └── ui.ts            # UI 适配器函数
│   ├── types/
│   │   └── index.ts         # 类型定义
│   └── utils/
│       └── validation.ts    # 参数验证函数
├── dist/                     # 构建输出
├── tests/                    # 测试文件
└── package.json
```

### 版本管理

- 使用语义化版本控制 (SemVer)
- 主版本号：破坏性变更
- 次版本号：新功能添加
- 修订版本号：错误修复

## 扩展性设计

### 添加新功能

1. 在 `PluginSDKFunctions` 接口中添加新方法类型
2. 在所有适配器创建函数中实现该方法
3. 添加相应的类型定义和参数验证函数
4. 编写测试用例

### 添加新环境支持

1. 创建新的适配器创建函数返回 `PluginSDKFunctions`
2. 在 `detectEnvironment` 函数中添加检测逻辑
3. 在 `createPluginSDK` 函数中添加适配器选择逻辑
4. 添加相应的测试

### 向后兼容性

- 使用 TypeScript 的可选参数和默认值
- 保持现有 API 接口不变
- 通过版本号管理破坏性变更
- 提供迁移指南和废弃警告