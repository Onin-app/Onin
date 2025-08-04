# SDK 核心架构设计

## 1. 模块划分

### 1.1 核心API模块
```typescript
interface CoreAPI {
  // UI相关
  showNotification(title: string, message: string): void;
  registerCommand(command: string, handler: () => void): void;
  
  // 系统访问
  readFile(path: string): Promise<string>;
  executeCommand(command: string): Promise<string>;
}
```

### 1.2 扩展模块
```typescript
interface PluginLifecycle {
  onActivate(): void;
  onDeactivate(): void;
  onError(error: Error): void;
}
```

### 1.3 工具模块
```typescript
class DevTools {
  static generatePluginTemplate(name: string): void;
  static validateManifest(manifest: PluginManifest): boolean;
}
```

## 2. 版本控制策略

采用语义化版本(SemVer)：
- 主版本号：不兼容的API修改
- 次版本号：向下兼容的功能新增
- 修订号：向下兼容的问题修正

## 3. 架构图

```mermaid
graph TD
    A[SDK Core] --> B[API Modules]
    A --> C[Lifecycle Hooks]
    A --> D[Dev Tools]
    B --> E[UI APIs]
    B --> F[System APIs]
    C --> G[Activation]
    C --> H[Deactivation]
    D --> I[CLI]
    D --> J[Template Generator]