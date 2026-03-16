# ai

AI 能力 API，调用用户在 Onin 中配置的 AI 大模型（OpenAI 兼容协议）。

## 导入

```typescript
import { ai } from 'onin-sdk';
```

> **前提**：用户需要在 Onin 设置 → AI 设置中配置好 AI 提供商。

## API

### `ai.ask(prompt, options?)`

发送一条消息，获取 AI 回复（非流式）。

```typescript
// 简单字符串提问
const reply = await ai.ask('用一句话解释什么是量子纠缠');
console.log(reply); // 'AI 的回答...'

// 指定模型
const reply = await ai.ask('写个快速排序', { model: 'gpt-4o' });
```

### `ai.stream(prompt, onChunk, options?)`

流式输出 AI 回复，逐块接收内容。

```typescript
let fullText = '';

await ai.stream('写一首关于编程的诗', (chunk) => {
  fullText += chunk;
  // 实时更新 UI
  document.getElementById('output')!.textContent = fullText;
});
```

### `ai.isAvailable()`

检查 AI 是否已配置可用。

```typescript
const available = await ai.isAvailable();
if (!available) {
  // 提示用户去配置 AI
}
```

### `ai.listModels()`

获取当前提供商支持的模型列表。

```typescript
const models = await ai.listModels();
// [{ id: 'gpt-4o', name: 'GPT-4o', context_window: 128000 }, ...]
```

### `ai.createConversation(systemPrompt?)`

创建多轮对话管理器，自动维护对话历史。

```typescript
const conv = ai.createConversation('你是一个专业的代码审查助手。');

const reply1 = await conv.ask('帮我看看这段代码：const x = 1 + "2"');
const reply2 = await conv.ask('那这样写有什么问题？'); // 上下文保留

// 流式多轮对话
await conv.stream('继续解释一下...', (chunk) => {
  process.stdout.write(chunk);
});

// 获取历史
const history = conv.getHistory();

// 清除历史（保留 system prompt）
conv.clear();
```

## 完整示例

```typescript
import { ai, command, notification } from 'onin-sdk';

await command.handle(async (code, args) => {
  if (code === 'ai-summarize') {
    const { text } = args;

    if (!(await ai.isAvailable())) {
      return { error: '请先在 Onin 设置中配置 AI 提供商' };
    }

    const summary = await ai.ask(`请用 3 句话总结以下内容：\n\n${text}`, {
      model: 'gpt-4o-mini',
    });

    return { summary };
  }
});
```
