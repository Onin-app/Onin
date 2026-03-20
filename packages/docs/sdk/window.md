# pluginWindow

窗口事件监听 API，提供对插件窗口（Inline 或独立 Window）的显示/隐藏/焦点事件监听能力。

## 导入

```typescript
import { pluginWindow } from 'onin-sdk';
```

## API

### 事件监听

| 方法                             | 触发时机           |
| -------------------------------- | ------------------ |
| `pluginWindow.onShow(callback)`  | 窗口从隐藏变为可见 |
| `pluginWindow.onHide(callback)`  | 窗口从可见变为隐藏 |
| `pluginWindow.onFocus(callback)` | 窗口获得焦点       |
| `pluginWindow.onBlur(callback)`  | 窗口失去焦点       |

```typescript
pluginWindow.onShow(() => {
  // Inline 模式：Onin 展开且展示此插件时触发
  // Window 模式：独立窗口显示或从最小化恢复时触发
  refreshData();
});

pluginWindow.onHide(() => {
  // 暂停后台任务，节省资源
  pauseTimers();
});
```

### 获取运行模式

```typescript
// 同步获取（可能返回 'unknown'，若运行时信息尚未初始化）
const mode = pluginWindow.getMode(); // 'inline' | 'window' | 'unknown'

// 异步获取（等待运行时初始化完成）
const mode = await pluginWindow.getModeAsync(); // 'inline' | 'window'
```

### 获取插件 ID

```typescript
const pluginId = pluginWindow.getPluginId(); // string
```

## 示例

```typescript
import { pluginWindow, storage } from 'onin-sdk';

let data: any[] = [];

// 每次窗口显示时刷新数据
pluginWindow.onShow(async () => {
  data = await fetchLatestData();
  renderList(data);
});

// 窗口隐藏时保存状态
pluginWindow.onHide(async () => {
  await storage.setItem('lastState', { timestamp: Date.now() });
});

// 根据模式调整 UI
const mode = await pluginWindow.getModeAsync();
if (mode === 'window') {
  // 独立窗口模式，可以显示更多内容
  document.querySelector('.sidebar')?.removeAttribute('hidden');
}
```

## Inline vs Window 模式下的行为差异

| 事件      | Inline 模式           | Window 模式              |
| --------- | --------------------- | ------------------------ |
| `onShow`  | Onin 弹出并展示此插件 | 独立窗口显示或最小化恢复 |
| `onHide`  | Onin 窗口隐藏         | 独立窗口最小化或隐藏     |
| `onFocus` | 内嵌 Webview 获得焦点 | 独立窗口获得焦点         |
| `onBlur`  | 内嵌 Webview 失去焦点 | 独立窗口失去焦点         |
