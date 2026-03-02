# scheduler

定时任务 API，基于 cron 表达式，支持持久化和应用重启后自动恢复。

## 导入

```typescript
import { scheduler } from 'onin-plugin-sdk';
```

> **所需权限**：`"scheduler": { "enable": true }`

## API

### `scheduler.schedule(id, cron, callback)`

注册一个基于 cron 表达式的定时任务。

```typescript
// cron 格式: "分钟 小时 日期 月份 星期"
// 每天早上 8 点
await scheduler.schedule('daily-reminder', '0 8 * * *', async () => {
  await notification.show({ title: '早安', body: '开始新的一天！' });
});
```

**Cron 格式说明：**

```
┌─────── 分钟 (0-59)
│ ┌───── 小时 (0-23)
│ │ ┌─── 日期 (1-31)
│ │ │ ┌─ 月份 (1-12)
│ │ │ │ ┌ 星期 (0-6, 0=周日)
│ │ │ │ │
* * * * *
```

**常用示例：**

| cron 表达式     | 说明           |
| --------------- | -------------- |
| `0 8 * * *`     | 每天 8:00      |
| `0 * * * *`     | 每小时整点     |
| `30 12 * * 1-5` | 工作日 12:30   |
| `0 9 * * 1`     | 每周一 9:00    |
| `0 0 1 * *`     | 每月 1 日 0:00 |

### `scheduler.daily(id, time, callback)`

简化的每日定时，格式为 `HH:MM`。

```typescript
await scheduler.daily('morning', '08:30', async () => {
  await notification.show({ title: '早安提醒', body: '该起床了！' });
});
```

### `scheduler.hourly(id, minute, callback)`

每小时在指定分钟执行。

```typescript
// 每小时的第 30 分钟
await scheduler.hourly('sync', 30, () => syncData());
```

### `scheduler.weekly(id, weekday, time, callback)`

每周在指定星期几+时间执行。`weekday`: 0=周日, 1=周一, ..., 6=周六。

```typescript
await scheduler.weekly('report', 1, '09:00', () => generateReport());
```

### `scheduler.cancel(id)`

取消一个定时任务。

```typescript
await scheduler.cancel('morning');
```

### `scheduler.list()`

获取当前插件的所有定时任务列表。

```typescript
const tasks = await scheduler.list();
console.log('已注册任务:', tasks);
```
