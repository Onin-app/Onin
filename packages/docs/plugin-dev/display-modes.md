# 显示模式

Onin 插件支持两种显示方式，通过 `manifest.json` 中的 `display_mode` 字段配置。

## Inline 模式（默认）

插件在 Onin 主窗口的内容区域内嵌展示，通过 iframe 加载你的 HTML 入口文件。

```json
{
  "display_mode": "inline"
}
```

**特点：**

- 与 Onin 主窗口融为一体，视觉上更统一
- 当 Onin 窗口关闭时，插件也随之隐藏
- 适合快速查询类工具（翻译、计算、搜索结果展示等）

**窗口事件：**

```typescript
import { pluginWindow } from 'onin-plugin-sdk';

pluginWindow.onShow(() => {
  // 每次 Onin 弹出且展示此插件时触发
  refreshData();
});

pluginWindow.onHide(() => {
  // Onin 窗口隐藏时触发
  pauseTimers();
});
```

## Window 模式

插件在独立的浮动窗口中打开，该窗口独立于 Onin 主窗口存在。

```json
{
  "display_mode": "window"
}
```

**特点：**

- 拥有独立的窗口，可以拖拽、调整大小
- 支持「始终置顶」
- Onin 主窗口关闭后，插件窗口仍然存在
- 适合需要长期使用的工具（番茄时钟、音乐播放器等）

## 动态切换（Inline → Window）

用户可以在 Inline 模式下通过悬浮按钮（FAB）将插件「弹出」到独立窗口。

插件无需特殊处理，Onin 会自动处理切换逻辑，并通过 `pluginWindow` 的事件通知你当前模式。

```typescript
import { pluginWindow } from 'onin-plugin-sdk';

// 获取当前运行模式
const mode = pluginWindow.getMode(); // 'inline' | 'window' | 'unknown'

// 异步获取（等待运行时初始化）
const mode = await pluginWindow.getModeAsync(); // 'inline' | 'window'
```

## auto_detach 自动分离

设置 `auto_detach: true` 后，当用户选中该插件的指令时，会自动以 Window 模式打开，而不是在 Inline 区域展示。

```json
{
  "auto_detach": true
}
```

适合那种"启动后就应该一直常驻"的工具类插件。

## 模式选择建议

| 场景                 | 推荐模式                        |
| -------------------- | ------------------------------- |
| 翻译、计算、快速搜索 | `inline`                        |
| 番茄时钟、音乐播放器 | `window` 或 `auto_detach: true` |
| 文件处理工具         | `inline`，需要时手动弹出        |
| 数据管理面板         | `window`                        |
