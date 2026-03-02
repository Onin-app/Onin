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

## 下一步

- 📖 了解 [manifest.json 所有字段](./manifest)
- 🪟 学习 [Inline 与 Window 两种显示模式](./display-modes)
- 🔐 配置 [权限系统](./permissions)
- 📦 探索 [SDK API 参考](/sdk/)
