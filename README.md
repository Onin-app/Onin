# Baize

一个快速启动应用程序，类似于 Raycast、uTools、Alfred、Wox。

## 功能特性

- 🚀 快速启动：全局快捷键快速唤起
- 🔍 智能搜索：拼音搜索和模糊匹配
- 🔌 插件系统：支持 Headless 和 Webview 两种插件类型
- 📦 轻量体积：基于 Tauri，安装包体积小，内存占用低
- ⚡ 自定义命令：灵活配置自定义文件和命令

## 技术栈

- **Tauri**: 用于构建跨平台桌面应用程序
- **SvelteKit**: 用于构建用户界面
- **TypeScript**: 类型安全的开发体验
- **Tailwind CSS**: 用于样式设计
- **Bits UI**: Headless UI 组件库
- **Rust**: 用于编写 Tauri 后端代码

## 快速开始

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 确保已安装 Rust 和 Tauri CLI
# 参考: https://tauri.app/zh-cn/v1/guides/getting-started/prerequisites
```

### 开发模式

```bash
# 启动开发服务器
pnpm dev

# 或使用 Tauri 开发模式
pnpm tauri dev
```

### 构建应用

```bash
# 构建生产版本
pnpm build

# 构建 Tauri 应用
pnpm tauri build
```

## 插件开发

Baize 支持两种类型的插件：

### 插件生命周期

插件的 entry 文件（.js）在加载时自动执行，可以使用生命周期 API 注册设置和命令：

```json
{
  "id": "com.example.myplugin",
  "name": "我的插件",
  "entry": "index.js"
}
```

**index.js** - 在插件加载时自动执行（类似 Vue 的 onMounted）：
```javascript
import { lifecycle, settings, command } from 'baize-plugin-sdk';

// 🎯 使用 lifecycle.onLoad() 注册初始化逻辑
lifecycle.onLoad(async () => {
  // 注册设置（无需用户执行插件即可显示设置按钮）
  await settings.useSettingsSchema([
    {
      key: 'apiKey',
      label: 'API 密钥',
      type: 'password',
      required: true
    }
  ]);

  // 注册命令处理器
  command.register(async (cmd, args) => {
    if (cmd === 'get-status') {
      return { status: 'ready' };
    }
  });
});

// 执行所有 onLoad 回调
await lifecycle._executeLoadCallbacks();
```

### Headless 插件

适合纯逻辑处理，无需 UI 界面的场景。

```javascript
// index.js
import { lifecycle, command, notification } from 'baize-plugin-sdk';

lifecycle.onLoad(async () => {
  command.register(async (cmd, args) => {
    if (cmd === 'hello') {
      await notification.show({
        title: 'Hello',
        body: `Hello, ${args?.name || 'World'}!`
      });
    }
  });
});

// onLoad 回调会自动执行
```

### UI 插件

适合需要复杂 UI 交互的场景。

```html
<!-- index.html -->
<!DOCTYPE html>
<html>
<head>
  <title>My Plugin</title>
</head>
<body>
  <h1>My Plugin</h1>
  <button onclick="doSomething()">执行操作</button>
  
  <script type="module">
    import { settings, notification } from 'baize-plugin-sdk';
    
    window.doSomething = async () => {
      const config = await settings.getAll();
      await notification.show({
        title: 'Success',
        body: 'Operation completed!'
      });
    };
  </script>
</body>
</html>
```

### 插件配置 (manifest.json)

```json
{
  "id": "com.example.myplugin",
  "name": "我的插件",
  "version": "1.0.0",
  "description": "示例插件",
  "entry": "index.js",
  "commands": [
    {
      "code": "hello",
      "name": "问候指令",
      "description": "向用户问候",
      "keywords": [
        {"name": "hello", "type": "text"},
        {"name": "你好", "type": "text"}
      ]
    }
  ],
  "permissions": {
    "http": {
      "enable": true,
      "allowUrls": ["https://api.example.com/*"]
    },
    "storage": {
      "enable": true
    },
    "notification": {
      "enable": true
    }
  }
}
```

更多插件开发文档：
- [插件生命周期系统](./PLUGIN_LIFECYCLE.md)
- [插件命令使用](./PLUGIN_COMMAND_USAGE.md)
- [插件 SDK 文档](./plugins-sdk/README.md)

## 项目结构

```
baize/
├── src/                    # 前端源码 (SvelteKit)
│   ├── lib/               # 组件和工具库
│   ├── routes/            # 页面路由
│   └── index.css          # 全局样式
├── src-tauri/             # Tauri 后端 (Rust)
│   ├── src/               # Rust 源码
│   └── Cargo.toml         # Rust 依赖配置
├── plugins-sdk/           # 插件 SDK
├── static/                # 静态资源
└── docs/                  # 文档
```

## 开发环境

推荐使用 [VS Code](https://code.visualstudio.com/) 并安装以下插件：
- [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 许可证

MIT License
