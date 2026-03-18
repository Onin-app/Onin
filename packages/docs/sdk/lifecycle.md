# lifecycle

插件生命周期钩子，在插件加载和卸载时执行初始化与清理逻辑。

## 导入

```typescript
import { lifecycle } from 'onin-sdk';
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

## 后台入口脚本 (`lifecycle.js`)

对于 UI 插件，如果你希望在不打开界面的情况下执行后台逻辑（如定时同步数据、监听系统消息），可以结合 `manifest.json` 中的 `run_at_startup: true` 使用。

Onin 会在启动时自动加载 `manifest.lifecycle` 指定的后台入口文件（默认为 `lifecycle.js`），并由于该文件通常会导入 SDK 并注册 `onLoad`，你的初始化逻辑将在后台自动运行。

**manifest.json 示例：**

```json
{
  "run_at_startup": true,
  "lifecycle": "dist/lifecycle.js"
}
```

## 构建要求

对于 UI 插件，宿主需要一个可直接执行的后台入口文件。推荐做法是不再手写 `lifecycle.ts + vite.lifecycle.config.ts`，而是采用单源码声明模式：

- `src/plugin.ts` 导出 `background` 和 `ui`
- `src/background.ts` 作为薄包装，只注册 `background`
- `src/main.ts` 作为薄包装，只挂载 `ui`
- `scripts/build.mjs` 一次构建出 `dist/index.html` 和 `dist/lifecycle.js`

最小配置示例：

```json
{
  "scripts": {
    "build": "node ./scripts/build.mjs"
  }
}
```

```ts
import { command, definePlugin, settings } from 'onin-sdk';

export const background = async () => {
  await settings.useSettingsSchema([
    { key: 'apiKey', label: 'API 密钥', type: 'password', required: true },
  ]);

  await command.handle(async (code) => {
    if (code === 'my-command') {
      return { ok: true };
    }
    return null;
  });
};

export default definePlugin({
  background,
  ui: {
    mount: async ({ target }) => {
      const { mountPluginUi } = await import('./ui');
      return mountPluginUi({ target });
    },
  },
});
```

发布前检查：

- `manifest.lifecycle` 的路径和产物路径一致
- zip 里确实包含该文件
- 如果后台入口里注册了 settings 或 commands，本地解压后也能看到该文件

如果缺少这个构建步骤，插件 UI 仍然可能正常打开，但后台初始化不会执行，进而导致设置页、命令注册和启动初始化全部失效。
