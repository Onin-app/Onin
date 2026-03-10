# 窗口生命周期 API - 最终简化方案

## 设计理念

**简单就是美**。使用浏览器原生 API 而不是复杂的自定义事件系统。

## 实现方案

### 前端（plugins-sdk）

使用浏览器标准 API：

- `document.visibilitychange` - 检测文档可见性变化
- `window.focus` / `window.blur` - 检测窗口焦点变化

```typescript
// 独立窗口模式
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    // 窗口隐藏
    executeWindowHideCallbacks();
  } else {
    // 窗口显示
    executeWindowShowCallbacks();
  }
});

window.addEventListener("focus", () => {
  if (!document.hidden) {
    executeWindowShowCallbacks();
  }
});

window.addEventListener("blur", () => {
  executeWindowHideCallbacks();
});
```

### 后端（src-tauri）

**不需要任何特殊处理**！

- 不需要注入自定义事件系统
- 不需要 `eval` 触发事件
- 不需要 `initialization_script`

只需要正常创建窗口即可：

```rust
let builder = WebviewWindowBuilder::new(
    &app,
    window_label,
    tauri::WebviewUrl::External(plugin_url.parse().unwrap()),
)
.title(plugin.manifest.name.clone())
.inner_size(800.0, 600.0)
.resizable(true)
.decorations(false)
.transparent(false);
```

## 使用方法

### 插件开发者

```typescript
// lifecycle.ts
import { lifecycle } from "onin-plugin-sdk";

lifecycle.onWindowShow(() => {
  console.log("窗口显示了");
  // 刷新数据、恢复定时器等
});

lifecycle.onWindowHide(() => {
  console.log("窗口隐藏了");
  // 暂停任务、保存状态等
});
```

### 确保 lifecycle.ts 被加载

```html
<!-- index.html -->
<!DOCTYPE html>
<html>
  <body>
    <!-- 你的内容 -->

    <script type="module" src="./lifecycle.ts"></script>
    <script type="module" src="./ui.ts"></script>
  </body>
</html>
```

## 优点

1. **简单**：使用标准浏览器 API，不需要复杂的初始化
2. **可靠**：浏览器原生 API 经过充分测试，兼容性好
3. **高性能**：不需要 Rust 和 JavaScript 之间的通信
4. **易维护**：代码量少，逻辑清晰
5. **零依赖**：不依赖 Tauri 的事件系统

## 防抖机制

SDK 内置 100ms 防抖，防止短时间内多次触发：

```typescript
let lastShowTime = 0;
let lastHideTime = 0;
const DEBOUNCE_MS = 100;

async function executeWindowShowCallbacks(): Promise<void> {
  const now = Date.now();
  if (now - lastShowTime < DEBOUNCE_MS) {
    return; // 忽略重复触发
  }
  lastShowTime = now;

  // 执行回调...
}
```

## 支持的模式

### 1. 独立窗口模式（display_mode: "window"）

使用浏览器原生 API，完美支持。

### 2. 旧 iframe 模式（display_mode: "inline"，已由 native inline webview 替代）

这是历史实现，使用 `postMessage` 通信：

```typescript
// 父窗口向 iframe 发送消息
iframe.contentWindow.postMessage(
  {
    type: "plugin-lifecycle-event",
    event: "show", // 或 'hide'
  },
  "*",
);

// iframe 内监听消息
window.addEventListener("message", (event) => {
  if (event.data.type === "plugin-lifecycle-event") {
    if (event.data.event === "show") {
      executeWindowShowCallbacks();
    } else if (event.data.event === "hide") {
      executeWindowHideCallbacks();
    }
  }
});
```

## 测试

1. 打开插件窗口
2. 按快捷键隐藏窗口 → 应该触发 `onWindowHide`
3. 按快捷键显示窗口 → 应该触发 `onWindowShow`
4. 切换到其他应用 → 应该触发 `onWindowHide`
5. 切换回来 → 应该触发 `onWindowShow`

预期日志：

```
[Lifecycle] Starting window events initialization
[Lifecycle] Environment check - inline webview mode
[Lifecycle] Initializing window events for standalone window mode
[Lifecycle] Using native visibilitychange API
[Lifecycle] ✅ Native browser event listeners registered

// 隐藏窗口时
[Lifecycle] ✅ Window blurred
[Lifecycle] Executing window hide callbacks, count: 1
窗口隐藏了

// 显示窗口时
[Lifecycle] ✅ Document visible (visibilitychange)
[Lifecycle] Executing window show callbacks, count: 1
窗口显示了
```

## 与之前方案的对比

| 特性     | 之前的方案              | 新方案               |
| -------- | ----------------------- | -------------------- |
| 后端代码 | 需要注入 100+ 行 JS     | 不需要任何特殊代码   |
| 前端代码 | 复杂的重试逻辑          | 简单的事件监听       |
| 依赖     | 依赖 Tauri 事件系统     | 只依赖浏览器标准 API |
| 可靠性   | 需要等待 Tauri API 加载 | 立即可用             |
| 性能     | 需要 Rust ↔ JS 通信    | 纯前端，零开销       |
| 维护成本 | 高                      | 低                   |

## 总结

通过使用浏览器原生 API，我们实现了一个：

- ✅ 更简单
- ✅ 更可靠
- ✅ 更高效
- ✅ 更易维护

的窗口生命周期系统。

**记住：简单的方案往往是最好的方案。**
