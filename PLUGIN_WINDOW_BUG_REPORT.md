# 插件窗口拖动和 Hover 失效 Bug 报告

## Bug 概述

在重构插件窗口实现后，出现了两个严重问题：
1. **窗口拖动完全失效** - 无法通过拖动标题栏移动窗口
2. **Resize 后 Hover 失效** - 窗口尺寸改变后，所有元素的 hover 效果失效，鼠标样式不变化

## 问题表现

### 初始状态
- ✅ 窗口可以正常显示
- ✅ 按钮可以点击（通过触摸板轻点直接触发）
- ✅ Hover 效果正常

### Resize 后
- ❌ 所有 hover 效果失效（包括按钮、iframe 内容）
- ❌ 鼠标样式不再变化
- ❌ 控制台疯狂打印日志，永不停止
- ⚠️ 只能通过"重点出现 hover 效果，再轻点触发"的方式操作，体验像移动端

### 拖动问题
- ❌ 无论在标题栏任何位置都无法拖动窗口
- ✅ 点击事件可以正常触发

## 根本原因

### 问题 1: Resize 后 Hover 失效 - `onResized` 无限循环

**原因：**
```typescript
// 错误的实现
await currentWindow.onResized(async () => {
  isMaximized = await currentWindow.isMaximized();
});
```

`onResized` 回调中更新了 `isMaximized` 状态，触发 Svelte 组件重新渲染（因为按钮中有 `{#if isMaximized}` 条件渲染），DOM 更新又触发了新的 resize 事件，形成**无限循环**。

**表现：**
- 控制台疯狂打印 "Window resized"
- 页面一直在重新渲染
- Hover 状态无法稳定，因为 DOM 一直在变化

**解决方案：**
完全移除 `onResized` 监听器，改为在按钮点击时手动切换状态：

```typescript
async function handleMaximize() {
  if (isMaximized) {
    await invoke("plugin_unmaximize_window", { label: windowLabel });
    isMaximized = false;  // 手动更新状态
  } else {
    await invoke("plugin_maximize_window", { label: windowLabel });
    isMaximized = true;   // 手动更新状态
  }
}
```

### 问题 2: 窗口拖动失效 - Tauri 权限配置缺失

**原因：**
Tauri v2 使用细粒度的权限系统，每个窗口需要明确授予权限才能使用特定的 API。插件窗口的 capabilities 配置中**缺少 `core:window:allow-start-dragging` 权限**。

**错误日志：**
```
window.start_dragging not allowed on window "plugin_com_translate_20251014"
allowed on: [windows: "main", URL: local]
referenced by: capability: default, permission: allow-start-dragging
```

**解决方案：**
在 `src-tauri/capabilities/plugin.json` 中添加拖动权限：

```json
{
  "permissions": [
    "core:default",
    "core:webview:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-is-maximized",
    "core:window:allow-set-focus",
    "core:window:allow-set-title",
    "core:window:allow-start-dragging"  // ← 添加这一行
  ]
}
```

## 调试过程

### 尝试 1: 使用 `data-tauri-drag-region` 属性
- ❌ 完全不工作
- 原因：权限问题，但当时不知道

### 尝试 2: 手动调用 `currentWindow.startDragging()`
- ❌ 报错但不清楚原因
- 原因：权限问题，但错误信息被 catch 吞掉了

### 尝试 3: 调整 CSS（pointer-events, backdrop-filter 等）
- ❌ 无效
- 原因：问题不在 CSS

### 尝试 4: 修改窗口配置（transparent, title_bar_style 等）
- ❌ 无效
- 原因：问题不在窗口配置

### 尝试 5: 添加调试日志
- ✅ 发现了两个关键信息：
  1. Resize 后控制台疯狂打印 → 发现无限循环
  2. 拖动时的错误日志 → 发现权限问题

## 最终解决方案

### 前端代码 (src/routes/plugin-window/+page.svelte)

```typescript
// 1. 移除 onResized 监听器
async function setupWindowListeners() {
  isMaximized = await currentWindow.isMaximized();
  window.addEventListener("message", handlePluginMessage);
}

// 2. 手动管理最大化状态
async function handleMaximize() {
  if (isMaximized) {
    await invoke("plugin_unmaximize_window", { label: windowLabel });
    isMaximized = false;
  } else {
    await invoke("plugin_maximize_window", { label: windowLabel });
    isMaximized = true;
  }
}

// 3. 使用 mousedown 事件触发拖动
function handleTitlebarMouseDown(event: MouseEvent) {
  if (event.button !== 0) return;
  
  const target = event.target as HTMLElement;
  if (target.closest('button') || target.closest('input') || target.closest('select')) {
    return;
  }
  
  currentWindow.startDragging();
}
```

```html
<!-- 在标题栏上绑定 mousedown 事件 -->
<div class="titlebar" onmousedown={handleTitlebarMouseDown}>
  <div class="titlebar-title">{pluginName || "Plugin"}</div>
  <div class="titlebar-controls">
    <!-- 按钮 -->
  </div>
</div>
```

### 后端配置 (src-tauri/capabilities/plugin.json)

```json
{
  "identifier": "plugin-capability",
  "windows": ["plugin_*"],
  "permissions": [
    "core:window:allow-start-dragging"  // 添加拖动权限
  ]
}
```

## 经验教训

### 1. Tauri v2 权限系统
- **细粒度权限控制**：每个窗口需要明确授予权限
- **调试技巧**：查看完整的错误日志，不要用 `.catch()` 吞掉错误
- **文档重要性**：Tauri v2 的权限系统是新特性，需要仔细阅读文档

### 2. Svelte 响应式陷阱
- **避免在事件监听器中更新状态**：可能导致无限循环
- **条件渲染的副作用**：`{#if}` 会触发 DOM 更新
- **调试技巧**：观察控制台是否有重复日志

### 3. 调试策略
- **逐步排除法**：从简单到复杂
- **添加日志**：关键位置添加 console.log
- **不要吞掉错误**：`.catch()` 要打印错误信息
- **查看完整错误**：错误信息往往包含解决方案

### 4. macOS 特殊性
- `data-tauri-drag-region` 在 macOS 上对 CSS 敏感
- `pointer-events: none` 会导致拖动失效
- 但本次问题的根本原因是权限，不是 macOS 特性

## 相关文件

- `src/routes/plugin-window/+page.svelte` - 插件窗口前端实现
- `src-tauri/src/plugin_manager.rs` - 插件窗口创建逻辑
- `src-tauri/capabilities/plugin.json` - 插件窗口权限配置

## 测试验证

修复后验证以下功能：
- ✅ 窗口可以通过拖动标题栏移动
- ✅ Resize 后 hover 效果正常
- ✅ 按钮可以正常点击
- ✅ 最大化/还原按钮图标正确切换
- ✅ 控制台无异常日志

## 总结

这是一个由**两个独立问题**组成的复合 bug：
1. **前端问题**：`onResized` 无限循环导致 hover 失效
2. **配置问题**：Tauri 权限缺失导致拖动失效

两个问题互相干扰，增加了调试难度。最终通过添加详细日志，发现了问题的根本原因。

**关键启示**：在 Tauri v2 中，任何窗口 API 调用失败时，首先检查 capabilities 配置！

---

## 修复应用过程（Stash Apply 冲突解决）

### 背景

Bug 修复是在历史 commit `dd9cb2b892fde7e69bf52593656b1c4d631ca444` 上完成的，修复后将代码 stash 保存。当基于最新代码执行 `git stash apply` 时，遇到了多个文件的合并冲突。

### 冲突文件

1. `src-tauri/capabilities/plugin.json` - 权限配置冲突
2. `src-tauri/src/lib.rs` - 无实际冲突（自动合并）
3. `src-tauri/src/plugin_manager.rs` - 窗口创建逻辑冲突
4. `src/routes/plugin-window/+page.svelte` - 前端实现冲突（最复杂）

### 解决策略

**核心原则**：保留所有最新功能，同时应用关键的 bug 修复。

### 具体解决方案

#### 1. plugin.json - 简单合并

**冲突内容：**
- Bug 修复：添加了 `"core:window:allow-start-dragging"` 权限
- 最新代码：添加了 `remote.urls` 配置

**解决方案：** 两者都保留

```json
{
  "permissions": [
    // ... 其他权限
    "core:window:allow-start-dragging"  // Bug 修复
  ],
  "remote": {
    "urls": ["http://localhost:*"]      // 最新功能
  }
}
```

#### 2. plugin_manager.rs - 功能合并

**冲突内容：**
- Bug 修复：无改动
- 最新代码：
  - 添加了 `get_plugin_detail` 命令（读取 README）
  - 添加了开发模式支持（devMode/devServer）
  - 添加了窗口状态保存/恢复功能
  - 添加了窗口菜单创建

**解决方案：** 保留所有最新功能

关键合并点：
```rust
// 保留开发模式打印
if plugin.manifest.dev_mode {
    if let Some(dev_server) = &plugin.manifest.dev_server {
        println!("[plugin_manager] Plugin {} is in dev mode", plugin.manifest.id);
    }
}

// 保留窗口状态恢复
let window_states = load_plugin_window_states(&app);
let saved_bounds = window_states.get(&plugin.manifest.id);

// 保留菜单创建
let menu_result = (|| -> Result<Menu<tauri::Wry>, tauri::Error> {
    let back_to_inline = MenuItemBuilder::with_id("back_to_inline", "切换到主窗口模式").build(&app)?;
    Menu::with_items(&app, &[&back_to_inline])
})();

// 合并窗口构建器逻辑
let mut builder = WebviewWindowBuilder::new(/* ... */);

// 应用保存的窗口位置
if let Some(bounds) = saved_bounds {
    builder = builder.position(bounds.x, bounds.y).inner_size(bounds.width, bounds.height);
} else {
    builder = builder.inner_size(800.0, 600.0);
}

// 添加菜单
let builder = if let Ok(menu) = menu_result {
    builder.menu(menu)
} else {
    builder
};
```

#### 3. +page.svelte - 复杂合并（关键）

**冲突内容：**

Bug 修复（Stashed changes）：
- ❌ 移除了 `onResized` 监听器
- ✅ 使用 `currentWindow.startDragging()`
- ✅ 手动管理 `isMaximized` 状态
- ❌ 移除了 `ArrowsIn` 图标

最新代码（Updated upstream）：
- ❌ 保留了 `onResized` 监听器（bug 根源！）
- ❌ 使用 `invoke("plugin_start_dragging")`
- ✅ 添加了开发模式支持（devMode/devServer）
- ✅ 添加了"切换到主窗口"功能（`handleBackToInline`）
- ✅ 添加了 iframe load 事件处理（`handleIframeLoad`）
- ✅ 添加了窗口可见性事件监听
- ✅ 添加了 `backdrop-filter` 样式

**解决方案：** 保留所有新功能 + 应用 bug 修复

关键改动点：

1. **保留 ArrowsIn 图标导入**（最新功能需要）
```typescript
import { Minus, Square, CornersIn, X, ArrowsIn } from "phosphor-svelte";
```

2. **保留开发模式支持**
```typescript
// 根据开发模式决定使用哪个 URL
if (plugin.devMode && plugin.devServer) {
  const url = new URL(plugin.devServer);
  url.searchParams.set("plugin_id", pluginId);
  pluginUrl = url.toString();
} else {
  const port = await invoke<number>("get_plugin_server_port");
  pluginUrl = `http://127.0.0.1:${port}/plugin/${plugin.dir_name}/${plugin.entry}?mode=window&plugin_id=${pluginId}`;
}
```

3. **移除 onResized 监听器**（Bug 修复）
```typescript
// ❌ 删除这段代码（会导致无限循环）
// const unlisten = await currentWindow.onResized(async () => {
//   isMaximized = await currentWindow.isMaximized();
// });

// ✅ 只检查初始状态
isMaximized = await currentWindow.isMaximized();
```

4. **手动管理 isMaximized 状态**（Bug 修复）
```typescript
const handleMaximize = async () => {
  if (isMaximized) {
    await invoke("plugin_unmaximize_window", { label: windowLabel });
    isMaximized = false;  // ← 手动更新
  } else {
    await invoke("plugin_maximize_window", { label: windowLabel });
    isMaximized = true;   // ← 手动更新
  }
};
```

5. **使用 currentWindow.startDragging()**（Bug 修复）
```typescript
// ❌ 旧实现（需要额外的 invoke 命令）
// await invoke("plugin_start_dragging");

// ✅ 新实现（直接使用 Tauri API）
function handleTitlebarMouseDown(event: MouseEvent) {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement;
  if (target.closest('button') || target.closest('input') || target.closest('select')) {
    return;
  }
  currentWindow.startDragging();  // ← 直接调用 API
}
```

6. **保留所有新功能**
```typescript
// 保留 iframe load 处理
const handleIframeLoad = () => { /* ... */ };

// 保留窗口可见性监听
const unlistenVisibility = await listen<boolean>("window_visibility", /* ... */);

// 保留切换到主窗口功能
const handleBackToInline = async () => { /* ... */ };
```

7. **保留新增的 UI 元素**
```html
<!-- 保留"切换到主窗口"按钮 -->
<button onclick={handleBackToInline} class="titlebar-button titlebar-button-inline">
  <ArrowsIn size={16} weight="bold" />
</button>

<div class="titlebar-separator"></div>
```

8. **保留新增的样式**
```css
.titlebar {
  backdrop-filter: blur(10px);  /* 保留 */
}

.titlebar-button-inline:hover { /* 保留 */ }
.titlebar-separator { /* 保留 */ }
```

### 验证清单

合并完成后，确保以下功能都正常：

- ✅ **Bug 修复验证**
  - 窗口可以拖动（通过标题栏）
  - Resize 后 hover 效果正常
  - 控制台无无限循环日志
  - 最大化/还原按钮状态正确

- ✅ **新功能验证**
  - 开发模式可以加载 devServer
  - "切换到主窗口"按钮可用
  - iframe 可以接收 plugin_id
  - 窗口状态可以保存和恢复
  - 窗口菜单正常显示

### 关键经验

1. **理解冲突的本质**
   - Bug 修复：移除了导致问题的代码
   - 新功能：添加了新的功能代码
   - 目标：保留新功能，同时不引入旧 bug

2. **识别关键修复点**
   - `onResized` 监听器必须移除（核心 bug）
   - `currentWindow.startDragging()` 必须使用（权限已添加）
   - 手动状态管理必须保留（避免无限循环）

3. **保留所有新功能**
   - 不要因为解决冲突而删除新功能
   - 仔细阅读两边的代码，理解每个改动的目的
   - 测试确保新功能和 bug 修复都生效

4. **测试驱动验证**
   - 先列出所有需要验证的功能点
   - 逐一测试确保没有遗漏
   - 特别关注 bug 修复的核心问题是否真正解决
