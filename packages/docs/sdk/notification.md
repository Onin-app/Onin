# notification

系统通知 API，调用操作系统的原生通知能力。

## 导入

```typescript
import { notification } from 'onin-sdk';
```

> **所需权限**：`"notification": { "enable": true }`

## API

### `notification.show(options)`

发送一条系统通知。

```typescript
await notification.show({
  title: '任务完成',
  body: '数据同步已完成，共处理 100 条记录。',
});
```

**参数：**

| 字段    | 类型     | 必填 | 说明           |
| ------- | -------- | ---- | -------------- |
| `title` | `string` | ✅   | 通知标题       |
| `body`  | `string` | ❌   | 通知正文       |
| `icon`  | `string` | ❌   | 自定义图标路径 |

## 示例

```typescript
import { notification, scheduler } from 'onin-sdk';

// 基础通知
await notification.show({
  title: '提醒',
  body: '你的番茄时钟结束了，休息一下吧！',
});

// 定时通知（配合 scheduler）
await scheduler.daily('morning-reminder', '08:30', async () => {
  await notification.show({
    title: '早安 ☀️',
    body: '新的一天开始了，今天也要加油！',
  });
});
```
