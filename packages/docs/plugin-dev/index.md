# 核心概念

## 什么是 Onin 插件？

Onin 插件是一个独立的 Web 应用，通过 `manifest.json` 声明它提供的指令和权限，并使用 `onin-plugin-sdk` 与 Onin 主程序进行通信。

插件的本质就是一个你熟悉的前端项目——可以用 React、Vue、Svelte 或原生 HTML 开发，只需满足两个条件：

1. 根目录有 `manifest.json` 文件
2. 使用 `onin-plugin-sdk` 接入 Onin 能力

## 插件的两种类型

### UI 插件（`type: "ui"`）

UI 插件拥有独立的前端界面，在 Onin 的视图区域展示 HTML 页面。适合需要复杂交互的场景。

### 脚本插件（`type: "script"`）

脚本插件没有 UI，只包含后台逻辑（JavaScript）。适合数据处理、自动化任务等场景。

## 两种显示模式

UI 插件支持两种显示方式：

- **Inline 模式**（默认）：在 Onin 主窗口的内容区域内嵌展示，通过 iframe 加载
- **Window 模式**：在独立的浮动窗口中打开，支持拖拽、调整大小、置顶等

## 插件通信

插件通过 `onin-plugin-sdk` 提供的 API 与主程序通信：

```typescript
import { command, storage, notification } from 'onin-plugin-sdk';

// 注册指令处理器
await command.handle(async (code, args) => {
  // 处理来自主程序的指令
  return { result: 'done' };
});
```

所有 API 调用均为异步，基于消息传递机制，安全且高效。

## 插件隔离

每个插件运行在独立的沙盒环境中：

- **存储隔离**：每个插件有自己独立的存储空间
- **权限控制**：插件只能使用 `manifest.json` 中声明过的权限
- **网络限制**：HTTP 请求需要显式声明允许的域名
