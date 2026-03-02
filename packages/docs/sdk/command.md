# command

指令处理 API，提供注册指令处理器和动态管理指令的能力。

## 导入

```typescript
import { command } from 'onin-plugin-sdk';
```

## API

### `command.handle(handler)`

注册指令处理器，接收来自 Onin 的指令执行请求。**每个插件只需调用一次**。

```typescript
await command.handle(async (code: string, args: any) => {
  // 返回值会作为指令结果反馈给 Onin
  return result;
});
```

**参数：**

| 参数      | 类型                               | 说明                                               |
| --------- | ---------------------------------- | -------------------------------------------------- |
| `handler` | `(code: string, args: any) => any` | 指令处理函数。`code` 为指令标识，`args` 为执行参数 |

**示例 — 多指令路由：**

```typescript
await command.handle(async (code, args) => {
  switch (code) {
    case 'greet':
      return `Hello, ${args.name || 'World'}!`;

    case 'calculate':
      const { a, b } = args;
      return a + b;

    default:
      throw new Error(`Unknown command: ${code}`);
  }
});
```

---

### `command.register(definition)`

动态注册一条指令。适合根据用户数据（书签、联系人等）在运行时生成指令列表。

```typescript
await command.register({
  code: 'open-bookmark-1',
  name: 'My Favorite Site',
  keywords: [{ name: 'bookmark' }],
});
```

**参数 — `CommandDefinition`：**

| 字段          | 类型                       | 必填 | 说明         |
| ------------- | -------------------------- | ---- | ------------ |
| `code`        | `string`                   | ✅   | 指令唯一标识 |
| `name`        | `string`                   | ✅   | 指令显示名称 |
| `description` | `string`                   | ❌   | 指令描述     |
| `keywords`    | `CommandKeyword[]`         | ❌   | 触发关键词   |
| `matches`     | `CommandMatchDefinition[]` | ❌   | 内容匹配规则 |

---

### `command.remove(code)`

移除一条动态注册的指令。

```typescript
await command.remove('open-bookmark-1');
```

## 完整示例

```typescript
import { lifecycle, command, storage } from 'onin-plugin-sdk';

lifecycle.onLoad(async () => {
  // 从存储读取书签列表，动态注册指令
  const bookmarks = (await storage.getItem<string[]>('bookmarks')) ?? [];
  for (const [index, url] of bookmarks.entries()) {
    await command.register({
      code: `bookmark-${index}`,
      name: url,
      keywords: [{ name: 'bookmark' }],
    });
  }

  // 统一处理所有指令
  await command.handle(async (code, args) => {
    if (code.startsWith('bookmark-')) {
      const index = parseInt(code.split('-')[1]);
      const url = bookmarks[index];
      // 打开对应 URL（需要结合 command.run 或直接 window.open）
      return { url };
    }
  });
});
```
