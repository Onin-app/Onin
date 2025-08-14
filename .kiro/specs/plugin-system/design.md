# 插件系统设计文档

## 概述

基于 Tauri + SvelteKit 架构设计一个渐进式插件系统。系统采用前后端分离的架构，Rust 后端负责插件的加载、管理和安全隔离，前端负责用户界面和插件交互。采用 monorepo 结构管理 SDK 和示例插件。

## 架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    主应用 (Tauri App)                        │
├─────────────────────────────────────────────────────────────┤
│  前端 (SvelteKit)           │  后端 (Rust)                   │
│  ┌─────────────────────┐   │  ┌─────────────────────────┐   │
│  │ 插件管理界面         │   │  │ 插件管理器               │   │
│  │ - 插件列表          │   │  │ - 插件发现               │   │
│  │ - 启用/禁用         │   │  │ - 插件加载               │   │
│  │ - 配置界面          │   │  │ - 生命周期管理           │   │
│  └─────────────────────┘   │  └─────────────────────────┘   │
│  ┌─────────────────────┐   │  ┌─────────────────────────┐   │
│  │ 插件通信层           │   │  │ 安全沙箱                │   │
│  │ - API 调用          │   │  │ - 权限控制               │   │
│  │ - 事件系统          │   │  │ - 资源隔离               │   │
│  └─────────────────────┘   │  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    插件生态系统                              │
├─────────────────────────────────────────────────────────────┤
│  SDK Package                │  示例插件                      │
│  ┌─────────────────────┐   │  ┌─────────────────────────┐   │
│  │ @baize/plugin-sdk   │   │  │ hello-world-plugin      │   │
│  │ - 类型定义          │   │  │ - 基础示例               │   │
│  │ - API 接口          │   │  └─────────────────────────┘   │
│  │ - 构建工具          │   │  ┌─────────────────────────┐   │
│  │ - 开发工具          │   │  │ advanced-plugin         │   │
│  └─────────────────────┘   │  │ - 高级功能示例           │   │
│                            │  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 项目结构 (Monorepo)

```
baize/
├── src/                          # 主应用前端
├── src-tauri/                    # 主应用后端
├── packages/                     # Monorepo 包管理
│   ├── plugin-sdk/               # 插件 SDK
│   │   ├── src/
│   │   │   ├── types.ts          # 类型定义
│   │   │   ├── api.ts            # API 接口
│   │   │   ├── events.ts         # 事件系统
│   │   │   └── index.ts          # 主入口
│   │   ├── package.json
│   │   └── README.md
│   └── create-plugin/            # 插件脚手架工具
│       ├── templates/
│       ├── src/
│       └── package.json
├── plugins/                      # 示例插件
│   ├── hello-world/
│   └── advanced-example/
├── docs/                         # 文档
│   ├── plugin-development.md
│   └── api-reference.md
└── pnpm-workspace.yaml          # Monorepo 配置
```

## 组件和接口

### 1. 插件 SDK (@baize/plugin-sdk)

#### 核心接口定义

```typescript
// types.ts
export interface PluginManifest {
  name: string;
  version: string;
  description: string;
  author: string;
  main: string;
  permissions: string[];
  engines: {
    baize: string;
  };
}

export interface PluginContext {
  app: AppAPI;
  events: EventAPI;
  storage: StorageAPI;
}

export interface Plugin {
  activate(context: PluginContext): Promise<void>;
  deactivate(): Promise<void>;
}

// API 接口
export interface AppAPI {
  showNotification(message: string): Promise<void>;
  getAppVersion(): Promise<string>;
  openDialog(options: DialogOptions): Promise<string | null>;
}

export interface EventAPI {
  on(event: string, handler: Function): void;
  emit(event: string, data?: any): void;
  off(event: string, handler: Function): void;
}

export interface StorageAPI {
  get(key: string): Promise<any>;
  set(key: string, value: any): Promise<void>;
  remove(key: string): Promise<void>;
}
```

### 2. 插件管理器 (Rust)

#### 核心结构

```rust
// src-tauri/src/plugin_manager.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub main: String,
    pub permissions: Vec<String>,
    pub engines: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
    pub loaded: bool,
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self;
    pub async fn discover_plugins(&mut self) -> Result<(), PluginError>;
    pub async fn load_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub fn get_plugin_list(&self) -> Vec<&PluginInfo>;
    pub async fn enable_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub async fn disable_plugin(&mut self, name: &str) -> Result<(), PluginError>;
}
```

### 3. 前端插件管理界面

#### 组件结构

```typescript
// src/lib/components/plugins/PluginManager.svelte
interface PluginState {
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  loaded: boolean;
  status: 'active' | 'inactive' | 'error' | 'loading';
}

// 主要功能
- 插件列表展示
- 启用/禁用切换
- 插件详情查看
- 错误状态显示
- 搜索和过滤
```

## 数据模型

### 插件清单文件 (plugin.json)

```json
{
  "name": "hello-world-plugin",
  "version": "1.0.0",
  "description": "A simple hello world plugin",
  "author": "Developer Name",
  "main": "dist/index.js",
  "permissions": [
    "notifications",
    "storage"
  ],
  "engines": {
    "baize": ">=0.1.0"
  },
  "keywords": ["example", "hello-world"],
  "repository": "https://github.com/user/hello-world-plugin"
}
```

### 插件配置存储

```rust
// 使用 tauri-plugin-store 存储插件配置
{
  "plugins": {
    "hello-world-plugin": {
      "enabled": true,
      "config": {
        "greeting": "Hello from plugin!"
      }
    }
  }
}
```

## 错误处理

### 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Version incompatible: required {required}, found {found}")]
    VersionIncompatible { required: String, found: String },
}
```

### 错误处理策略

1. **插件发现阶段**：记录错误但不中断整个发现过程
2. **插件加载阶段**：显示具体错误信息，允许用户重试
3. **运行时错误**：隔离错误插件，不影响其他插件和主应用
4. **权限错误**：明确提示所需权限，允许用户授权

## 测试策略

### 单元测试

1. **SDK 测试**
   - API 接口测试
   - 事件系统测试
   - 类型定义验证

2. **插件管理器测试**
   - 插件发现逻辑
   - 加载/卸载流程
   - 错误处理机制

3. **前端组件测试**
   - 插件列表渲染
   - 状态切换功能
   - 用户交互测试

### 集成测试

1. **端到端测试**
   - 插件完整生命周期
   - 主应用与插件通信
   - 多插件协同工作

2. **示例插件测试**
   - 基础功能验证
   - API 调用测试
   - 错误场景测试

### 性能测试

1. **插件加载性能**
   - 启动时间测试
   - 内存使用监控
   - 并发加载测试

2. **运行时性能**
   - 事件传递延迟
   - API 调用响应时间
   - 资源占用监控

## 安全考虑

### 权限系统

1. **声明式权限**：插件必须在清单文件中声明所需权限
2. **最小权限原则**：只授予插件必需的最小权限集
3. **用户确认**：敏感权限需要用户明确授权

### 代码隔离

1. **沙箱环境**：插件运行在受限的 JavaScript 环境中
2. **API 白名单**：只暴露安全的 API 接口给插件
3. **资源限制**：限制插件的 CPU 和内存使用

### 数据安全

1. **数据隔离**：插件只能访问自己的存储空间
2. **敏感数据保护**：主应用敏感数据不暴露给插件
3. **通信加密**：插件与主应用间的通信进行加密

## 部署和分发

### 开发环境

1. **本地开发**：支持热重载的开发环境
2. **调试工具**：提供插件调试和日志功能
3. **测试工具**：自动化测试和验证工具

### 生产环境

1. **插件打包**：标准化的插件打包格式
2. **版本管理**：语义化版本控制
3. **依赖管理**：处理插件间的依赖关系

### 分发渠道

1. **本地安装**：支持从本地文件安装插件
2. **在线商店**：未来可扩展的插件商店功能
3. **开发者工具**：插件开发和发布工具链