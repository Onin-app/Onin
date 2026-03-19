# 5 分钟快速开始

本教程带你从零写一个能在 Onin 中运行的插件。

## 准备工作

确保已安装：

- Node.js >= 18
- pnpm（推荐）或 npm

## 1. 用脚手架创建项目

推荐直接使用 `create-onin-plugin`：

```bash
npx create-onin-plugin my-onin-plugin
cd my-onin-plugin
pnpm install
```

如果你想直接创建 React 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework react
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Vue 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework vue
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Vanilla TypeScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework vanilla
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Vanilla JavaScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework vanilla --language js
cd my-onin-plugin
pnpm install
```

如果你想直接创建 React JavaScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework react --language js
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Vue JavaScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework vue --language js
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Svelte JavaScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework svelte --language js
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Solid JavaScript 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework solid --language js
cd my-onin-plugin
pnpm install
```

如果你想直接创建 Solid 插件项目：

```bash
npx create-onin-plugin my-onin-plugin --framework solid
cd my-onin-plugin
pnpm install
```

脚手架默认会生成一个可发布的 UI 插件模板，已经包含：

- `src/plugin.ts` 或 `src/plugin.js`
- `src/main.ts` 或 `src/main.js`
- `manifest.json`
- `vite.config.ts`
- `scripts/build.mjs`
- `pnpm build`
- `pnpm pack:plugin`

开发模式：

```bash
pnpm dev
```

发布产物：

```bash
pnpm pack:plugin
```

`pnpm pack:plugin` 会先构建插件，再生成一个适合插件市场上传的 `plugin.zip`。

## 2. 手动创建项目（备选）

如果你需要完全自定义项目结构，也可以手动搭建：

```bash
mkdir my-onin-plugin
cd my-onin-plugin
pnpm init
pnpm add onin-sdk
```

## 3. 创建 manifest.json

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

## 4. 创建入口文件

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
      import { notification, command } from 'onin-sdk';

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

## 5. 加载到 Onin

1. 打开 Onin，进入「设置」→「插件」
2. 点击「从本地导入」
3. 选择你的插件项目目录
4. 安装完成后，在主搜索框输入 `hello` 即可触发指令

## 6. 开发模式（热更新）

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

## 7. 给 UI 插件准备后台入口

如果你的 UI 插件要做以下任一事情，就不要只构建页面入口，还必须额外产出后台入口脚本：

- 注册插件设置页
- 注册指令处理器
- 做启动时初始化逻辑
- 使用 `run_at_startup`

推荐目录：

```text
my-onin-plugin/
├─ src/
│  ├─ plugin.ts
│  ├─ main.ts
│  └─ ui.ts
├─ index.html
├─ manifest.json
├─ vite.config.ts
└─ scripts/build.mjs
```

`src/plugin.ts` 示例：

```ts
import { command, definePlugin, settings } from 'onin-sdk';

export const setup = async () => {
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
};

export default definePlugin({
  setup,
  mount: async ({ target }) => {
    const { mountPluginUi } = await import('./ui');
    return mountPluginUi({ target });
  },
});
```

`scripts/build.mjs` 会统一完成 UI 和后台入口两次构建：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "node ./scripts/build.mjs"
  }
}
```

对 HTML UI 插件，Onin 会按固定约定查找 `dist/background.js`。只要这个文件存在，`setup` 里的设置、指令和启动初始化就会执行。

## 8. 发布前检查

发布到插件市场前，先直接检查 zip 根目录或解压目录，至少应包含：

- `manifest.json`
- `icon.png` 或其他图标文件
- `dist/index.html` 及其静态资源
- `dist/background.js`

最常见的问题是本地开发可用，但发布 zip 漏了后台入口文件。这样插件页面仍然能打开，但 `setup` 里的设置 schema、指令处理器、启动初始化都不会注册。

## 下一步

- 📖 了解 [manifest.json 所有字段](./manifest)
- 🪟 学习 [Inline 与 Window 两种显示模式](./display-modes)
- 🔐 配置 [权限系统](./permissions)
- 📦 探索 [SDK API 参考](/sdk/)
