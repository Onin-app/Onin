# Design Document

## Overview

本设计文档描述了如何将现有的基于 class 的插件 SDK 完全重构为函数式编程方式。设计目标是提供一个更简洁、更现代的 API，使用英文注释提高代码可读性，并简化整体架构。新的函数式系统将完全替代现有的 class 系统。

## Architecture

### 核心设计原则

1. **简洁优先**: 函数式 API 应该比 class 方式更简洁易用
2. **英文注释**: 所有代码注释使用英文，保持代码的国际化标准
3. **完全替换**: 新系统完全替代 class 系统，避免架构复杂性
4. **现代化**: 使用现代 JavaScript/TypeScript 特性和最佳实践

### 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    插件管理器                                │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │   插件加载器     │    │        生命周期管理器            │ │
│  └─────────────────┘    └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
┌───────────────▼──┐    ┌──────▼──────┐    ┌──▼──────────────┐
│   函数式插件     │    │   Hook 系统  │    │   上下文管理     │
│   定义接口       │    │             │    │                 │
└──────────────────┘    └─────────────┘    └─────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
┌───────────────▼──┐    ┌──────▼──────┐    ┌──▼──────────────┐
│   应用 API       │    │   事件系统   │    │   存储 API       │
│                  │    │             │    │                 │
└──────────────────┘    └─────────────┘    └─────────────────┘
```

## Components and Interfaces

### 1. 函数式插件接口

#### 核心类型定义

```typescript
// Plugin definition configuration
export interface PluginDefinition {
  /** Callback function when plugin is activated */
  onActivate?: (context: PluginContext) => Promise<void> | void;
  /** Callback function when plugin is deactivated */
  onDeactivate?: () => Promise<void> | void;
  /** Plugin metadata */
  meta?: {
    /** Plugin name */
    name?: string;
    /** Plugin version */
    version?: string;
    /** Plugin description */
    description?: string;
    /** Plugin author */
    author?: string;
  };
}

// 插件定义函数
export function definePlugin(definition: PluginDefinition): Plugin;
```

#### Hook 风格的 API 函数

```typescript
// React 风格的 Hook API
export function useApp(): AppAPI;
export function useEvents(): EventAPI; 
export function useStorage(): StorageAPI;
export function useContext(): PluginContext;

// 生命周期 Hook
export function onActivate(handler: () => Promise<void> | void): void;
export function onDeactivate(handler: () => Promise<void> | void): void;
```

### 2. 上下文管理系统

上下文管理系统负责在插件执行期间提供 API 访问：

```typescript
// Global context manager
class ContextManager {
  private currentContext: PluginContext | null = null;
  
  /** Set current plugin context */
  setContext(context: PluginContext): void {
    this.currentContext = context;
  }
  
  /** Get current plugin context */
  getContext(): PluginContext {
    if (!this.currentContext) {
      throw new Error('Plugin context not available, ensure calling within plugin lifecycle');
    }
    return this.currentContext;
  }
  
  /** Clear current context */
  clearContext(): void {
    this.currentContext = null;
  }
}

export const contextManager = new ContextManager();
```

### 3. 插件生命周期管理器

```typescript
export class PluginLifecycleManager {
  /** Activate plugin */
  async activatePlugin(plugin: Plugin, context: PluginContext): Promise<void> {
    // Set context
    contextManager.setContext(context);
    
    try {
      // Call plugin activation function
      if (plugin.onActivate) {
        await plugin.onActivate();
      }
    } finally {
      // Ensure context remains available after activation
    }
  }
  
  /** Deactivate plugin */
  async deactivatePlugin(plugin: Plugin): Promise<void> {
    try {
      // Call plugin deactivation function
      if (plugin.onDeactivate) {
        await plugin.onDeactivate();
      }
    } finally {
      // Clear context
      contextManager.clearContext();
    }
  }
}
```

### 4. 简化的插件加载器

```typescript
export class PluginLoader {
  /** Load plugin module */
  async loadPlugin(pluginPath: string): Promise<Plugin> {
    const module = await import(pluginPath);
    const plugin = module.default || module;
    
    // Validate plugin format
    if (!this.isValidPlugin(plugin)) {
      throw new PluginError('Invalid plugin format', PluginErrorCode.INVALID_MANIFEST);
    }
    
    return plugin;
  }
  
  /** Validate if plugin is valid */
  private isValidPlugin(plugin: any): plugin is Plugin {
    return plugin && typeof plugin === 'object';
  }
}
```

## Data Models

### 插件数据结构

```typescript
// Plugin interface (simplified version)
export interface Plugin {
  /** Callback function when plugin is activated */
  onActivate?: () => Promise<void> | void;
  /** Callback function when plugin is deactivated */
  onDeactivate?: () => Promise<void> | void;
  /** Plugin metadata */
  meta?: PluginMeta;
}

// Plugin metadata
export interface PluginMeta {
  /** Plugin name */
  name?: string;
  /** Plugin version */
  version?: string;
  /** Plugin description */
  description?: string;
  /** Plugin author */
  author?: string;
}

// Plugin context (keep existing structure)
export interface PluginContext {
  /** Application API */
  app: AppAPI;
  /** Event system API */
  events: EventAPI;
  /** Storage API */
  storage: StorageAPI;
}
```

### 插件信息结构

```typescript
// Simplified plugin info structure
export interface PluginInfo {
  /** Plugin manifest data */
  manifest: PluginManifest;
  /** Current plugin status */
  status: PluginStatus;
  /** Whether the plugin is enabled by user */
  enabled: boolean;
  /** Error message if plugin is in error state */
  error?: string;
  /** Plugin file path */
  path?: string;
}
```

## Error Handling

### 错误处理策略

1. **上下文错误**: 当插件尝试在错误的上下文中访问 API 时的错误处理
2. **生命周期错误**: 在插件激活或停用过程中的错误处理
3. **API 调用错误**: 插件调用应用 API 时的错误处理

```typescript
// Extended existing error types
export enum PluginErrorCode {
  // ... existing error codes
  /** Plugin context not available */
  CONTEXT_NOT_AVAILABLE = 'CONTEXT_NOT_AVAILABLE',
  /** Plugin lifecycle error */
  LIFECYCLE_ERROR = 'LIFECYCLE_ERROR',
  /** Hook call error */
  HOOK_ERROR = 'HOOK_ERROR'
}

// Error handling utility function
export function handlePluginError(error: Error, pluginName?: string): void {
  if (error instanceof PluginError) {
    // Known plugin error, use error message
    console.error(`[Plugin ${pluginName}] ${error.message}`);
  } else {
    // Unknown error, wrap as plugin error
    const pluginError = new PluginError(
      `Plugin runtime error: ${error.message}`,
      PluginErrorCode.LIFECYCLE_ERROR,
      pluginName
    );
    console.error(pluginError);
  }
}
```

## Testing Strategy

### 测试层次

1. **单元测试**
   - `definePlugin` 函数
   - Hook 函数（`useApp`, `useEvents`, `useStorage`）
   - 上下文管理器
   - 生命周期管理器

2. **集成测试**
   - 插件加载和激活流程
   - API 功能完整性
   - 错误处理机制

3. **端到端测试**
   - 完整的插件生命周期
   - 插件间的交互
   - 错误恢复场景

### 测试用例示例

```typescript
// 插件定义测试
describe('definePlugin', () => {
  it('should create a valid plugin object', () => {
    const plugin = definePlugin({
      onActivate: async () => {
        const app = useApp();
        await app.showNotification('Hello');
      }
    });
    
    expect(plugin).toBeDefined();
    expect(typeof plugin.onActivate).toBe('function');
  });
});

// Hook tests
describe('useApp', () => {
  it('should return application API object', () => {
    // Set mock context
    const mockContext = createMockContext();
    contextManager.setContext(mockContext);
    
    const app = useApp();
    expect(app).toBe(mockContext.app);
  });
  
  it('should throw error when no context available', () => {
    contextManager.clearContext();
    
    expect(() => useApp()).toThrow('Plugin context not available');
  });
});
```

## Implementation Details

### 文件结构

```
packages/plugin-sdk/src/
├── index.ts              # 主要导出（函数式 API）
├── plugin.ts             # 插件定义函数
├── hooks.ts              # Hook 函数实现
├── context.ts            # 上下文管理器
├── lifecycle.ts          # 生命周期管理器
├── loader.ts             # 插件加载器
├── types.ts              # 类型定义（更新为函数式）
├── api.ts                # API 实现（保持现有）
├── events.ts             # 事件系统（保持现有）
├── communication.ts      # 通信层（保持现有）
└── utils/
    ├── migration.ts      # 迁移工具
    └── validation.ts     # 验证工具
```

### 迁移策略

1. **自动迁移工具**: 提供脚本自动转换现有 class 插件
2. **渐进式重构**: 逐步替换现有系统组件
3. **文档更新**: 更新所有文档为函数式 API
4. **示例更新**: 更新所有示例插件为函数式风格

### 性能考虑

1. **轻量级设计**: 函数式 API 比 class 方式更轻量
2. **上下文复用**: 高效的上下文管理避免重复创建
3. **内存管理**: 自动清理插件资源和事件监听器
4. **类型优化**: 使用 TypeScript 的类型推断提高性能