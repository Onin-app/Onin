# 插件生命周期架构

## 概述

插件系统支持两种类型的插件，采用简单统一的生命周期模型：

- **Headless 插件**：无界面的后台插件（入口：`index.js`）
- **View 插件**：带界面的插件（入口：`index.html`，生命周期：`lifecycle.js`）

## 生命周期钩子

所有插件都可以使用以下生命周期钩子：

### `onLoad()`
插件被系统加载时调用。用于：
- 注册设置模式
- 注册命令处理器
- 初始化存储
- 设置插件状态

### `onUnload()`
插件被卸载或禁用时调用。用于：
- 清理资源
- 保存状态
- 取消待处理的操作
- 关闭连接

### `onWindowShow()`
插件窗口显示或从最小化恢复时调用。用于：
- 刷新窗口数据
- 恢复定时任务
- 更新 UI 状态

**支持的运行模式：**
- `display_mode="window"`: 独立窗口获得焦点时触发
- `display_mode="inline"`: 主窗口显示时触发（因为 iframe 在主窗口内）

### `onWindowHide()`
插件窗口隐藏或最小化时调用。用于：
- 暂停后台任务
- 保存临时状态
- 释放资源

**支持的运行模式：**
- `display_mode="window"`: 独立窗口失去焦点时触发
- `display_mode="inline"`: 主窗口隐藏时触发（因为 iframe 在主窗口内）

## 代码格式支持

插件代码支持以下格式：

| 格式 | 支持 | 说明 |
|------|------|------|
| **ES 模块** (ESM) | ✅ | 推荐！可以使用 `import`/`export`，支持顶层 await |
| **IIFE** | ✅ | 立即执行函数，兼容性最好 |
| **纯 JS** | ✅ | 无模块系统的普通 JS，会自动包装成异步函数 |

**推荐使用 ES 模块格式**，这是现代 JavaScript 的标准。

## 插件类型

### Headless 插件

Headless 插件在后台运行，没有任何界面。

**文件结构：**
```
my-headless-plugin/
├── src/
│   └── index.ts      # 源代码（TypeScript）
├── dist/
│   └── index.js      # 打包后的代码（ES 模块或 IIFE）
└── manifest.json
```

**manifest.json：**
```json
{
  "id": "com.example.my-plugin",
  "name": "我的 Headless 插件",
  "version": "1.0.0",
  "description": "一个后台插件",
  "entry": "dist/index.js",
  "commands": [...],
  "permissions": {...}
}
```

**src/index.ts（源代码）：**
```typescript
import { lifecycle, command, storage } from 'baize-plugin-sdk';

lifecycle.onLoad(async () => {
  console.log('插件加载');
  
  // 注册命令处理器
  command.register(async (cmd, args) => {
    // 处理命令
  });
  
  // 初始化存储
  await storage.setItem('initialized', true);
});

lifecycle.onUnload(async () => {
  // 清理
  await storage.setItem('last-unload', new Date().toISOString());
});
```

### View 插件

View 插件有用户界面（HTML/CSS/JS）。

**文件结构：**
```
my-view-plugin/
├── src/
│   ├── lifecycle.ts  # 生命周期源代码
│   ├── ui.ts        # UI 逻辑源代码
│   └── index.html   # HTML 模板
├── dist/
│   ├── lifecycle.js # 打包后的生命周期代码
│   ├── index.html   # 打包后的 HTML
│   └── assets/      # 打包后的资源
└── manifest.json
```

**manifest.json：**
```json
{
  "id": "com.example.my-ui-plugin",
  "name": "我的 View 插件",
  "version": "1.0.0",
  "description": "一个带界面的插件",
  "entry": "dist/index.html",
  "lifecycle": "dist/lifecycle.js",
  "display_mode": "inline",
  "commands": [...],
  "permissions": {...}
}
```

**src/lifecycle.ts：**
```typescript
import { lifecycle, command, settings, storage } from 'baize-plugin-sdk';

// 此文件在 UI 加载之前运行
// 不要在这里访问 DOM 元素

lifecycle.onLoad(async () => {
  console.log('生命周期加载');
  
  // 注册设置
  await settings.useSettingsSchema([
    {
      key: 'theme',
      label: '主题',
      type: 'select',
      options: [
        { label: '亮色', value: 'light' },
        { label: '暗色', value: 'dark' }
      ]
    }
  ]);
  
  // 注册命令
  command.register(async (cmd, args) => {
    // 处理命令
  });
  
  // 初始化存储
  await storage.setItem('initialized', true);
});

lifecycle.onUnload(async () => {
  // 清理
  await storage.setItem('last-unload', new Date().toISOString());
});

// 窗口显示/隐藏钩子（支持 window 和 inline 模式）
lifecycle.onWindowShow(() => {
  console.log('窗口已显示');
  // 刷新数据或恢复任务
});

lifecycle.onWindowHide(() => {
  console.log('窗口已隐藏');
  // 暂停定时器或保存状态
});
```

**src/index.html：**
```html
<!DOCTYPE html>
<html>
<head>
  <title>我的插件</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="app">
    <h1>我的插件界面</h1>
    <button id="test-btn">点击我</button>
  </div>
  <script type="module" src="ui.ts"></script>
</body>
</html>
```

**src/ui.ts：**
```typescript
import { storage, notification } from 'baize-plugin-sdk';

// 此文件处理 UI 交互
// 在 HTML 加载时运行

document.addEventListener('DOMContentLoaded', async () => {
  const btn = document.getElementById('test-btn');
  
  btn?.addEventListener('click', async () => {
    await notification.show({
      title: '按钮被点击',
      body: '来自 UI 的问候！'
    });
  });
});
```

## 核心原则

### 1. 关注点分离

**lifecycle.js**（View 插件）：
- ✅ 注册设置模式
- ✅ 注册命令处理器
- ✅ 初始化存储
- ✅ 设置插件状态
- ❌ 不访问 DOM
- ❌ 不进行 UI 交互

**ui.js**（View 插件）：
- ✅ DOM 操作
- ✅ 事件监听器
- ✅ UI 状态管理
- ✅ 用户交互

### 2. 执行顺序

**View 插件**：
1. 执行 `lifecycle.js`（注册设置、命令）
2. 加载 `index.html`
3. 执行 `ui.js`（初始化 UI）

**Headless 插件**：
1. 执行 `index.js`（注册设置、命令）

### 3. 生命周期独立性

- `lifecycle.js` 独立于 UI 运行
- 即使 UI 从未显示，它也会执行
- UI 代码可以安全地假设生命周期设置已完成

## 打包配置

### 推荐：使用 ES 模块格式

**vite.config.ts：**
```typescript
import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    outDir: 'dist',
    rollupOptions: {
      input: {
        lifecycle: resolve(__dirname, 'src/lifecycle.ts'),
      },
      output: {
        // 输出为 ES 模块
        format: 'es',
        entryFileNames: '[name].js',
      },
    },
  },
});
```

### 备选：使用 IIFE 格式

如果你需要最大兼容性，可以使用 IIFE：

```typescript
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        format: 'iife',
        inlineDynamicImports: true,
      },
    },
  },
});
```

## 示例

查看模板目录获取完整示例：
- `templates/headless-plugin/` - Headless 插件示例
- `templates/view-plugin/` - View 插件示例

## 最佳实践

1. **使用 ES 模块** - 现代标准，支持 `import`/`export` 和顶层 await
2. **使用 TypeScript** - 类型安全，更好的开发体验
3. **保持 lifecycle.js 简洁** - 只做注册和初始化
4. **lifecycle.js 中不要操作 DOM** - 所有 UI 代码放在单独的文件中
5. **使用 onUnload 清理** - 始终清理资源
6. **优雅处理错误** - 生命周期错误会阻止插件加载
7. **测试两个钩子** - 确保 onLoad 和 onUnload 都正常工作

## 常见问题

**问：View 插件可以跳过 lifecycle.js 吗？**
答：可以，如果你的插件不需要注册设置或命令，可以省略它。

**问：Headless 插件可以显示通知吗？**
答：可以！Headless 插件可以使用除 UI 相关之外的所有 API。

**问：什么时候应该使用 headless 还是 view 插件？**
答：后台任务、自动化和数据处理使用 headless。需要 UI 的面向用户的功能使用 view。

**问：可以在 ui.js 中访问设置吗？**
答：可以！设置 API 在 lifecycle.js 和 ui.js 中都可以使用。

**问：如果 lifecycle.js 有错误会怎样？**
答：插件将无法加载，并向用户显示错误。

**问：支持顶层 await 吗？**
答：支持！使用 ES 模块格式即可。
