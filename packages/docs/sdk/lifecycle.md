# lifecycle

插件生命周期钩子，在插件加载和卸载时执行初始化与清理逻辑。

## 导入

```typescript
import { lifecycle } from 'onin-plugin-sdk';
```

## API

### `lifecycle.onLoad(callback)`

注册插件加载时的回调。插件被系统加载时自动触发，在任何用户交互之前。

**推荐在此处完成所有初始化工作：**

- 注册设置页
- 注册指令处理器
- 初始化默认数据

```typescript
lifecycle.onLoad(async () => {
  // 注册设置
  await settings.useSettingsSchema([
    { key: 'apiKey', label: 'API 密钥', type: 'password', required: true },
  ]);

  // 注册指令处理器
  await command.handle(async (code, args) => {
    if (code === 'my-command') return handleCommand(args);
  });

  // 初始化数据
  const firstRun = await storage.getItem('firstRun');
  if (firstRun === null) {
    await storage.setItem('firstRun', false);
    await storage.setItem('installTime', new Date().toISOString());
  }
});
```

> 💡 可以多次调用 `onLoad`，所有回调会在同一个生命周期内按顺序执行。

### `lifecycle.onUnload(callback)`

注册插件卸载时的回调，在插件被禁用、卸载或应用关闭时触发。

**推荐在此处完成清理工作：**

- 取消定时任务
- 保存状态
- 关闭连接

```typescript
let syncInterval: number;

lifecycle.onLoad(() => {
  syncInterval = setInterval(() => syncData(), 60000);
});

lifecycle.onUnload(async () => {
  clearInterval(syncInterval);
  await storage.setItem('lastUnload', new Date().toISOString());
});
```

## 与 pluginWindow 的区别

`lifecycle` 处理的是插件的**安装级别**生命周期（加载/卸载），而 `pluginWindow` 处理的是**窗口级别**生命周期（显示/隐藏/焦点）。

| 事件          | 使用                            |
| ------------- | ------------------------------- |
| 插件初始化    | `lifecycle.onLoad`              |
| 插件卸载清理  | `lifecycle.onUnload`            |
| 窗口显示/隐藏 | `pluginWindow.onShow / onHide`  |
| 窗口焦点变化  | `pluginWindow.onFocus / onBlur` |

## 后台生命周期脚本 (lifecycle.js)

对于 UI 插件，如果你希望在不打开界面的情况下执行后台逻辑（如定时同步数据、监听系统消息），可以结合 `manifest.json` 中的 `run_at_startup: true` 使用。

Onin 会在启动时自动加载 `lifecycle` 指定的文件（默认为 `lifecycle.js`），并由于该文件通常会导入 SDK 并注册 `onLoad`，你的初始化逻辑将在后台自动运行。

**manifest.json 示例：**

```json
{
  "run_at_startup": true,
  "lifecycle": "dist/lifecycle.js"
}
```
