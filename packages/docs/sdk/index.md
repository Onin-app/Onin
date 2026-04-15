# SDK API 概览

`onin-sdk` 是 Onin 插件的官方开发工具包，提供与 Onin 主程序交互的所有能力。

## 安装

```bash
npm install onin-sdk
# 或
pnpm add onin-sdk
```

## 快速导入

```typescript
import {
  command,
  storage,
  http,
  fs,
  clipboard,
  dialog,
  notification,
  scheduler,
  lifecycle,
  settings,
  pluginWindow,
  ai,
  toast,
} from 'onin-sdk';
```

## API 模块总览

| 模块                             | 说明                            | 需要权限       |
| -------------------------------- | ------------------------------- | -------------- |
| [`command`](./command)           | 注册指令处理器、动态注册指令    | —              |
| [`storage`](./storage)           | 持久化键值存储                  | `storage`      |
| [`http`](./http)                 | HTTP 网络请求                   | `http`         |
| [`fs`](./fs)                     | 文件系统读写                    | —              |
| [`clipboard`](./clipboard)       | 读写剪贴板                      | —              |
| [`dialog`](./dialog)             | 系统对话框（文件选择、确认框）  | —              |
| [`notification`](./notification) | 系统通知                        | `notification` |
| [`scheduler`](./scheduler)       | 基于 cron 的定时任务            | `scheduler`    |
| [`lifecycle`](./lifecycle)       | 插件加载/卸载/后台生命周期回调 | —              |
| [`settings`](./settings)         | 插件设置页面配置                | —              |
| [`pluginWindow`](./window)       | 窗口事件监听（show/hide/focus） | —              |
| [`ai`](./ai)                     | 调用用户配置的 AI 能力          | —              |
| [`toast`](./toast)               | 窗口内提示信息（Success/Error） | —              |

## 典型用法模式

一个完整的插件通常按以下顺序初始化：

```typescript
import { lifecycle, settings, command, notification } from 'onin-sdk';

// 1. 在 onLoad 中完成所有初始化
lifecycle.onLoad(async () => {
  // 2. 注册设置页
  await settings.useSettingsSchema([
    { key: 'apiKey', label: 'API 密钥', type: 'password', required: true },
  ]);

  // 3. 注册指令处理器
  await command.handle(async (code, args) => {
    switch (code) {
      case 'my-command':
        return handleMyCommand(args);
      default:
        throw new Error(`Unknown command: ${code}`);
    }
  });
});

async function handleMyCommand(args: any) {
  await notification.show({ title: '执行成功', body: '指令已完成' });
  return { status: 'ok' };
}
```
