# AI API 开发文档

> 为 Onin 插件提供强大的 AI 能力

## 📖 目录

- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [API 参考](#api-参考)
- [多模态支持](#多模态支持)
- [最佳实践](#最佳实践)
- [常见问题](#常见问题)

---

## 快速开始

### 安装

AI API 已内置在 `@onin/sdk` 中,无需额外安装。

```typescript
import { ai } from "@onin/sdk";
```

### 第一个 AI 调用

```typescript
// 简单问答
const answer = await ai.ask("什么是 Rust?");
console.log(answer);
```

就这么简单!AI 会使用用户配置的 provider 和模型自动处理请求。

---

## 核心概念

### 1. Provider (提供商)

用户在 Onin 设置中配置 AI provider,如:

- OpenAI (GPT-4, GPT-3.5)
- Anthropic Claude
- 本地 Ollama
- 其他 OpenAI 兼容 API

**作为插件开发者,你不需要关心具体是哪个 provider**,API 会自动路由到用户配置的服务。

### 2. 两种输出模式

| 模式       | 方法       | 适用场景                |
| ---------- | ---------- | ----------------------- |
| **非流式** | `ask()`    | 大多数场景,等待完整响应 |
| **流式**   | `stream()` | 聊天界面,长文本生成     |

### 3. 多轮对话

使用 `Conversation` 类管理上下文:

```typescript
const conv = ai.createConversation();
await conv.ask("什么是所有权?");
await conv.ask("能举个例子吗?"); // AI 会记住上文
```

---

## API 参考

### `ai.ask(prompt, options?)`

发送问题并等待完整回答。

**参数:**

- `prompt`: `string | ChatMessage[]` - 问题或消息数组
- `options?`: `Partial<ChatRequest>` - 可选配置

**返回:** `Promise<string>` - AI 的完整回答

**示例:**

```typescript
// 简单问答
const answer = await ai.ask("解释一下闭包");

// 带配置
const answer = await ai.ask("写一首诗", {
  temperature: 0.9,
  max_tokens: 200,
});
```

---

### `ai.stream(prompt, onChunk, options?)`

流式接收 AI 回答,适合实时显示。

**参数:**

- `prompt`: `string | ChatMessage[]` - 问题
- `onChunk`: `(chunk: string) => void` - 接收每个文本片段的回调
- `options?`: `Partial<ChatRequest>` - 可选配置

**返回:** `Promise<void>` - 完成时 resolve

**示例:**

```typescript
await ai.stream("写一个故事", (chunk) => {
  // 实时显示每个片段
  outputElement.textContent += chunk;
});
```

---

### `ai.isAvailable()`

检查 AI 功能是否可用(用户是否已配置 provider)。

**返回:** `Promise<boolean>`

**示例:**

```typescript
if (await ai.isAvailable()) {
  // 可以使用 AI
} else {
  showMessage("请先在设置中配置 AI provider");
}
```

---

### `ai.getCapabilities()`

获取当前 provider 的能力信息。

**返回:** `Promise<AICapabilities | null>`

**AICapabilities 接口:**

```typescript
interface AICapabilities {
  supports_images: boolean; // 是否支持图片输入
  supports_streaming: boolean; // 是否支持流式输出
  supports_function_calling: boolean; // 是否支持函数调用
  max_context_tokens?: number; // 最大上下文长度
  max_images_per_message?: number; // 每条消息最多图片数
}
```

**示例:**

```typescript
const caps = await ai.getCapabilities();
if (caps?.supports_images) {
  // 可以发送图片
}
```

---

### `ai.listModels()`

列出当前 provider 支持的所有模型。

**返回:** `Promise<ModelInfo[]>`

**示例:**

```typescript
const models = await ai.listModels();
models.forEach((m) => console.log(m.id, m.name));
```

---

### `ai.createConversation(systemPrompt?)`

创建一个对话管理器,自动维护上下文。

**参数:**

- `systemPrompt?`: `string` - 系统提示词(可选)

**返回:** `Conversation` 实例

**Conversation 方法:**

- `ask(prompt, options?)` - 发送消息并记录到历史
- `stream(prompt, onChunk, options?)` - 流式发送并记录
- `getHistory()` - 获取完整对话历史
- `clear()` - 清空对话(保留 system prompt)
- `getLastResponse()` - 获取最后一次 AI 回复

**示例:**

```typescript
const conv = ai.createConversation("你是一个 Rust 专家");

const ans1 = await conv.ask("什么是生命周期?");
const ans2 = await conv.ask("能举个例子吗?"); // 保持上下文

console.log(conv.getHistory()); // 查看完整对话
```

---

## 多模态支持

### 检查图片支持

```typescript
const caps = await ai.getCapabilities();
if (!caps?.supports_images) {
  throw new Error("当前 AI 不支持图片");
}
```

### 发送图片

使用 `createImageMessage` 辅助函数:

```typescript
import { ai, createImageMessage } from "@onin/sdk";

// 发送图片 URL
const response = await ai.ask([
  createImageMessage("这张图里有什么?", ["https://example.com/image.jpg"]),
]);

// 发送多张图片
const response = await ai.ask([
  createImageMessage("比较这两张图", [
    "https://example.com/img1.jpg",
    "https://example.com/img2.jpg",
  ]),
]);

// 发送 Base64 图片
const base64 = "iVBORw0KGgoAAAANSUhEUgAA...";
const response = await ai.ask([createImageMessage("分析这张截图", [base64])]);

// 指定图片细节级别
const response = await ai.ask([
  createImageMessage("详细分析", [
    { url: "https://example.com/img.jpg", detail: "high" },
  ]),
]);
```

### 在对话中使用图片

```typescript
const conv = ai.createConversation("你是 UI 设计专家");

await conv.ask(createImageMessage("分析这个界面", ["https://..."]));

await conv.ask("有什么改进建议?"); // 继续讨论
```

---

## 最佳实践

### 1. 始终检查可用性

```typescript
if (!(await ai.isAvailable())) {
  showError("请先配置 AI provider");
  return;
}
```

### 2. 使用能力查询

```typescript
const caps = await ai.getCapabilities();

// 根据能力调整功能
if (caps?.supports_images) {
  enableImageUpload();
}

// 检查限制
if (caps?.max_images_per_message) {
  setImageLimit(caps.max_images_per_message);
}
```

### 3. 错误处理

```typescript
try {
  const answer = await ai.ask("你好");
} catch (error) {
  if (error.message.includes("No active AI provider")) {
    showError("请先配置 AI");
  } else if (error.message.includes("API Error")) {
    showError("AI 服务出错,请稍后重试");
  } else {
    showError("未知错误: " + error.message);
  }
}
```

### 4. 对话管理

```typescript
// ✅ 好的做法 - 使用 Conversation
const conv = ai.createConversation();
await conv.ask('问题1');
await conv.ask('问题2');  // 自动保持上下文

// ❌ 不好的做法 - 手动管理历史
const history = [];
history.push({ role: 'user', content: [...] });
const ans1 = await ai.ask(history);
history.push({ role: 'assistant', content: [...] });
// 容易出错!
```

### 5. 流式输出的正确用法

```typescript
let fullText = "";

await ai.stream("写一个故事", (chunk) => {
  fullText += chunk;
  updateUI(fullText); // 实时更新 UI
});

// 流式完成后使用完整文本
saveToDatabase(fullText);
```

---

## 常见问题

### Q: 如何指定使用特定模型?

A: 在 options 中指定 `model`:

```typescript
const answer = await ai.ask("你好", {
  model: "gpt-4",
});
```

如果不指定,会使用用户配置的默认模型。

---

### Q: 流式和非流式有什么区别?

A:

- **非流式** (`ask`): 等待完整响应,一次性返回,简单易用
- **流式** (`stream`): 实时接收片段,用户体验更好,适合长文本

大多数情况下用 `ask` 就够了。

---

### Q: Conversation 失败时会怎样?

A: 如果 AI 调用失败,`Conversation` 会自动回滚,移除刚添加的用户消息,保持历史一致性:

```typescript
const conv = ai.createConversation();
await conv.ask("问题1"); // 成功

try {
  await conv.ask("问题2"); // 失败
} catch (error) {
  // 历史中不会包含"问题2",保持干净
  console.log(conv.getHistory());
}
```

---

### Q: 如何限制 AI 回复长度?

A: 使用 `max_tokens`:

```typescript
const answer = await ai.ask("解释量子力学", {
  max_tokens: 100, // 限制在 100 tokens
});
```

---

### Q: 如何让 AI 更有创意/更保守?

A: 使用 `temperature`:

```typescript
// 更有创意 (0.7-1.0)
const creative = await ai.ask("写一首诗", {
  temperature: 0.9,
});

// 更保守/准确 (0.0-0.3)
const precise = await ai.ask("计算结果", {
  temperature: 0.1,
});
```

---

### Q: 支持哪些图片格式?

A: 大多数 AI provider 支持:

- PNG, JPEG, WebP, GIF (常见格式)
- 图片 URL 或 Base64 编码

具体支持情况取决于用户配置的 provider。

---

### Q: 如何处理超长对话?

A: 注意 token 限制:

```typescript
const conv = ai.createConversation();

// 对话太长时清空历史
if (conv.getHistory().length > 20) {
  conv.clear(); // 保留 system prompt
}
```

或者使用总结策略:

```typescript
// 定期总结对话
if (conv.getHistory().length > 10) {
  const summary = await ai.ask(
    `总结以下对话:\n${JSON.stringify(conv.getHistory())}`,
  );

  conv.clear();
  conv.addMessage(createTextMessage("system", `之前的对话总结: ${summary}`));
}
```

---

## 完整示例

### 示例 1: 代码审查助手

```typescript
import { ai } from "@onin/sdk";

async function reviewCode(code: string, language: string) {
  const conv = ai.createConversation("你是一个资深代码审查专家");

  const review = await conv.ask(
    `请审查这段 ${language} 代码:\n\`\`\`${language}\n${code}\n\`\`\``,
  );

  console.log("审查结果:", review);

  // 继续讨论
  const suggestions = await conv.ask("有什么具体的改进建议?");

  return { review, suggestions };
}
```

### 示例 2: 截图分析工具

```typescript
import { ai, createImageMessage } from "@onin/sdk";

async function analyzeScreenshot(imageUrl: string) {
  // 检查能力
  const caps = await ai.getCapabilities();
  if (!caps?.supports_images) {
    throw new Error("当前 AI 不支持图片分析");
  }

  const conv = ai.createConversation("你是 UI/UX 设计专家");

  // 分析界面
  const analysis = await conv.ask(
    createImageMessage("分析这个界面的设计", [imageUrl]),
  );

  // 获取建议
  const suggestions = await conv.ask("从可访问性角度有什么建议?");

  return { analysis, suggestions };
}
```

### 示例 3: 智能聊天界面

```typescript
import { ai } from "@onin/sdk";

class ChatInterface {
  private conversation = ai.createConversation();

  async sendMessage(message: string) {
    const outputDiv = document.getElementById("output");

    // 显示用户消息
    this.appendMessage("user", message);

    // 流式显示 AI 回复
    const aiMessageDiv = this.createMessageDiv("assistant");

    await this.conversation.stream(message, (chunk) => {
      aiMessageDiv.textContent += chunk;
    });
  }

  private appendMessage(role: string, text: string) {
    const div = this.createMessageDiv(role);
    div.textContent = text;
  }

  private createMessageDiv(role: string): HTMLDivElement {
    const div = document.createElement("div");
    div.className = `message ${role}`;
    document.getElementById("output")?.appendChild(div);
    return div;
  }
}
```

---

## 总结

AI API 提供了简单而强大的接口:

- ✅ **简单** - `ai.ask()` 一行代码搞定
- ✅ **灵活** - 支持流式、多轮对话、多模态
- ✅ **可靠** - 自动错误处理和回滚
- ✅ **透明** - 能力查询让你知道能做什么

开始使用吧!🚀
