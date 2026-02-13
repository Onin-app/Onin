# Extension 匹配系统架构

## 概述

Extension（内置扩展）采用 **统一匹配引擎 + 可选自定义钩子** 的架构，与 Plugin 共享前端匹配逻辑，同时支持后端语义级自定义匹配。

## 核心概念

### 两种匹配方式

| 方式                              | 位置                   | 触发场景        | 示例                               |
| --------------------------------- | ---------------------- | --------------- | ---------------------------------- |
| **声明式匹配 (`CommandMatch`)**   | 前端 `matchCommand.ts` | 粘贴文本/文件时 | Translator: `{type:"text", min:1}` |
| **自定义匹配 (`custom_matches`)** | 后端 Rust              | 实时输入时      | Calculator: 检测数学表达式         |

### 数据流

```
用户输入 "hello"
    │
    ├─► 关键词匹配 (fuzzyMatch)
    │   └─► 匹配到 "Demo Hello" (keyword: hello)
    │
    ├─► Extension 自定义匹配 (custom_matches)
    │   ├─► Calculator: "hello" → 非数学表达式 → None/false
    │   └─► Translator: "hello" → 非空文本 → Some(true) → 显示预览 "翻译: hello"
    │
    └─► CommandMatch 匹配 (matchCommand.ts)
        ├─► Translator: text min:1 → ✅ 匹配
        │   └─► 但已有预览，去重过滤 ❌ 不显示
        ├─► Demo AI Ask: text 1-500 → ✅ 匹配 → 显示
        └─► Demo Search: text 1-100 → ✅ 匹配 → 显示
```

## 关键文件

### 后端 (Rust)

- **`extension/registry.rs`** — `Extension` trait 定义，包含 `custom_matches()` 钩子
- **`extension/types.rs`** — `StaticCommandMatch` 类型，用于 `static` 声明式匹配规则
- **`extension/mod.rs`** — `get_extension_commands()` 将 `StaticCommandMatch` 转为前端 `CommandMatch`
- **`command_manager/storage.rs`** — 命令持久化，合并时同步 `matches` 字段
- **`command_manager/generators/extension.rs`** — 生成 Extension 的 `Command` 对象

### 前端 (TypeScript/Svelte)

- **`lib/utils/matchCommand.ts`** — 通用匹配引擎，处理 text/image/file/folder 类型
- **`routes/+page.svelte`** — 搜索结果去重逻辑（Extension 预览与 CommandMatch 去重）
- **`lib/components/settings/CommandCard.svelte`** — 支持 `mode` prop 区分功能/匹配视图
- **`lib/components/settings/CommandSettings.svelte`** — 设置页匹配指令标签页

## Extension trait 接口

```rust
pub trait Extension: Send + Sync {
    fn manifest(&self) -> &ExtensionManifest;

    /// 自定义匹配钩子（可选）
    /// - 返回 Some(true): 匹配成功，生成预览
    /// - 返回 Some(false): 明确不匹配
    /// - 返回 None: 不参与自定义匹配，交由前端 CommandMatch 处理
    fn custom_matches(&self, _input: &str) -> Option<bool> {
        None // 默认不实现
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview>;
    fn execute(&self, input: &str) -> ExtensionResult;
}
```

## 各 Extension 匹配策略

| Extension      | `custom_matches`                 | `StaticCommandMatch`     | 说明                   |
| -------------- | -------------------------------- | ------------------------ | ---------------------- |
| **Calculator** | ✅ 检测数学/单位/货币/日期表达式 | `None`                   | 纯语义匹配，不走声明式 |
| **Translator** | ✅ 任何非空文本 → `Some(true)`   | `[{type:"text", min:1}]` | 双重匹配，前端去重     |
| **Emoji**      | ❌ 默认 `None`                   | `None`                   | 仅关键词触发           |
| **Clipboard**  | ❌ 默认 `None`                   | `None`                   | 仅关键词触发           |

## 去重机制

搜索结果中，当 Extension 已有预览条目（通过 `custom_matches` + `preview` 生成）时，从 `matchedCommands` 中过滤掉 `source === "Extension"` 的条目，避免同一 Extension 重复显示。Plugin 命令不受影响。

```typescript
// +page.svelte displayList 去重
const deduplicatedMatchedCommands = extensionPreviewItem
  ? matchedCommands.filter((cmd) => cmd.source !== "Extension")
  : matchedCommands;
```

## 设置页展示

CommandCard 组件通过 `mode` prop 控制展示内容：

- **`mode="function"`**（功能指令标签页）→ 显示关键词/别名，隐藏匹配规则
- **`mode="match"`**（匹配指令标签页）→ 显示匹配规则，隐藏关键词/别名
- **`mode="all"`**（默认）→ 全部显示

同一命令可同时出现在两个标签页（如 Translator 既有关键词也有匹配规则）。
