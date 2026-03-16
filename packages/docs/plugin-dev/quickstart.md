# 5 分钟快速开始

本教程带你从零写一个能在 Onin 中运行的插件。

## 准备工作

确保已安装：

- Node.js >= 18
- pnpm（推荐）或 npm

## 1. 创建项目

```bash
mkdir my-onin-plugin
cd my-onin-plugin
pnpm init
pnpm add onin-plugin-sdk
```

## 2. 创建 manifest.json

这是插件的身份证，必须放在项目根目录：

```json
{
  "id": "my-first-plugin",
  "name": "我的第一个插件",
  "version": "0.1.0",
  "description": "一个简单的示例插件",
  "entry": "index.html",
  "type": "ui",
  "commands": [
    {
      "code": "hello",
      "name": "打个招呼",
      "description": "显示一个问好消息",
      "keywords": [{ "name": "hello" }, { "name": "你好" }]
    }
  ],
  "permissions": {
    "notification": {
      "enable": true
    }
  }
}
```

## 3. 创建入口文件

创建 `index.html`：

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <title>我的插件</title>
    <style>
      body {
        font-family: system-ui, sans-serif;
        padding: 24px;
        margin: 0;
      }
      button {
        padding: 8px 16px;
        border-radius: 6px;
        border: none;
        background: #3b82f6;
        color: white;
        cursor: pointer;
        font-size: 14px;
      }
    </style>
  </head>
  <body>
    <h2>👋 欢迎使用我的插件</h2>
    <p>点击按钮发送一条系统通知。</p>
    <button id="btn">发送通知</button>

    <script type="module">
      import { notification, command } from 'onin-plugin-sdk';

      // 注册指令处理器
      await command.handle(async (code, args) => {
        if (code === 'hello') {
          return { message: '你好，世界！' };
        }
      });

      // 按钮点击发送通知
      document.getElementById('btn').addEventListener('click', async () => {
        await notification.show({
          title: '来自我的插件',
          body: '这是一条测试通知！',
        });
      });
    </script>
  </body>
</html>
```

## 4. 加载到 Onin

1. 打开 Onin，进入「设置」→「插件」
2. 点击「从本地导入」
3. 选择你的插件项目目录
4. 安装完成后，在主搜索框输入 `hello` 即可触发指令

## 5. 开发模式（热更新）

开发时推荐启用 devMode，无需每次重新导入：

在 `manifest.json` 中添加：

```json
{
  "devMode": true,
  "devServer": "http://localhost:5173"
}
```

然后启动你的开发服务器：

```bash
pnpm dev
```

Onin 会直接加载开发服务器的内容，修改代码后自动刷新。

## 6. 给 UI 插件补上 lifecycle 构建

如果你的 UI 插件要做以下任一事情，就不要只构建页面入口，还必须额外构建 `lifecycle.js`：

- 注册插件设置页
- 注册指令处理器
- 做启动时初始化逻辑
- 使用 `run_at_startup`

推荐目录：

```text
my-onin-plugin/
├─ src/
│  ├─ main.ts
│  └─ lifecycle.ts
├─ index.html
├─ manifest.json
├─ vite.config.ts
└─ vite.lifecycle.config.ts
```

`src/lifecycle.ts` 示例：

```ts
import { lifecycle, settings, command } from 'onin-plugin-sdk';

lifecycle.onLoad(async () => {
  await settings.useSettingsSchema([
    {
      key: 'apiKey',
      label: 'API Key',
      type: 'password',
    },
  ]);

  await command.handle(async (code) => {
    if (code === 'hello') {
      return { ok: true };
    }
  });
});
```

`vite.lifecycle.config.ts` 示例：

```ts
import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    outDir: '.',
    emptyOutDir: false,
    lib: {
      entry: resolve(__dirname, 'src/lifecycle.ts'),
      formats: ['es'],
      fileName: () => 'lifecycle.js',
    },
    rollupOptions: {
      external: [],
      output: {
        inlineDynamicImports: true,
      },
    },
  },
});
```

`package.json` 至少要有：

```json
{
  "scripts": {
    "dev": "vite",
    "build:index": "vite build",
    "build:lifecycle": "vite build --config vite.lifecycle.config.ts",
    "build": "npm run build:index && npm run build:lifecycle"
  }
}
```

`manifest.json` 要和产物路径保持一致：

```json
{
  "entry": "dist/index.html",
  "lifecycle": "lifecycle.js"
}
```

如果你把生命周期文件输出到 `dist/`，那就把 manifest 改成 `"lifecycle": "dist/lifecycle.js"`。两边只要有一边不一致，Onin 就不会执行生命周期脚本，设置按钮和指令注册都会失效。

## 7. 发布前检查

发布到插件市场前，先直接检查 zip 根目录或解压目录，至少应包含：

- `manifest.json`
- `icon.png` 或其他图标文件
- `dist/index.html` 及其静态资源
- `lifecycle.js` 或 `manifest.lifecycle` 指向的实际文件

最常见的问题是本地开发可用，但发布 zip 漏了 `lifecycle.js`。这会导致插件页面能打开，但设置 schema、指令处理器、启动初始化都不会注册。

## 下一步

- 📖 了解 [manifest.json 所有字段](./manifest)
- 🪟 学习 [Inline 与 Window 两种显示模式](./display-modes)
- 🔐 配置 [权限系统](./permissions)
- 📦 探索 [SDK API 参考](/sdk/)
