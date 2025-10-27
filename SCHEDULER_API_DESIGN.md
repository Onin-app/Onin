# Scheduler API 设计文档

## 概述

为插件系统添加定时任务功能，支持基于 cron 表达式的任务调度。

## 设计决策

### 为什么需要 Scheduler API？

1. **持久化** - 应用重启后任务自动恢复
2. **统一管理** - 在插件管理界面查看所有定时任务
3. **权限控制** - 限制插件可创建的任务数量
4. **简化开发** - 插件作者无需自己实现定时逻辑

### 为什么不让插件自己用 setInterval？

- ❌ 应用关闭后任务丢失
- ❌ 无法跨会话持久化
- ❌ 难以管理和调试
- ❌ 每个插件都要写重复代码

## API 设计

### TypeScript SDK

```typescript
// 基础 API
await scheduler.schedule(id, cron, callback);

// 便捷方法
await scheduler.daily(id, time, callback);
await scheduler.hourly(id, minute, callback);
await scheduler.weekly(id, weekday, time, callback);

// 管理
await scheduler.cancel(id);
await scheduler.list();
```

### Cron 表达式格式

```
分 时 日 月 周
│ │ │ │ │
│ │ │ │ └─ 星期几 (0-6, 0=周日)
│ │ │ └─── 月份 (1-12)
│ │ └───── 日期 (1-31)
│ └─────── 小时 (0-23)
└───────── 分钟 (0-59)
```

### 示例

```typescript
// 每天早上 8 点
'0 8 * * *'

// 每小时的第 30 分钟
'30 * * * *'

// 每周一早上 9 点
'0 9 * * 1'

// 每月 1 号中午 12 点
'0 12 1 * *'
```

## 权限配置

在 `manifest.json` 中配置：

```json
{
  "permissions": {
    "scheduler": {
      "enable": true,
      "maxTasks": 5
    }
  }
}
```

- `enable`: 是否启用定时任务功能（必需）
- `maxTasks`: 插件最多可创建的任务数量（可选，默认 10）
```

## 实现细节

### Rust 后端

- `SchedulerState` - 存储所有任务
- `schedule_task` - 注册任务
- `cancel_task` - 取消任务
- `list_tasks` - 列出任务

### TypeScript SDK

- 任务回调存储在 `taskCallbacks` Map 中
- 提供便捷方法简化常见场景
- 自动处理错误

## 使用示例

### 每日一句插件

```typescript
import baize from '@baize/sdk';

async function fetchDailyQuote() {
  const response = await baize.http.get('https://v1.hitokoto.cn/');
  await baize.notification.show({
    title: '每日一句',
    body: response.body.hitokoto
  });
}

// 每天早上 8 点执行
baize.scheduler.daily('morning-quote', '08:00', fetchDailyQuote);
```

## 已实现的功能

✅ **核心调度逻辑** - 使用 `tokio-cron-scheduler` 实现真实的任务调度  
✅ **跨端通信** - 通过 Tauri 事件系统触发前端回调  
✅ **持久化存储** - 使用 `tauri-plugin-store` 保存任务，应用重启后自动恢复  
✅ **参数验证** - TypeScript 端验证时间格式和 cron 表达式  
✅ **完整的 Cron 支持** - 支持标准 5 字段 cron 表达式  

## 技术实现

### 调度执行流程

1. **注册任务** - 插件调用 `scheduler.schedule()`
2. **创建 Job** - Rust 后端使用 `tokio-cron-scheduler` 创建定时任务
3. **触发执行** - 到达时间时，后端发送 `scheduler:execute-task` 事件
4. **执行回调** - 前端监听事件，调用对应的回调函数
5. **持久化** - 任务信息保存到 `scheduler.json`

### 持久化机制

- 使用 `tauri-plugin-store` 存储任务配置
- 应用启动时自动加载并重新注册所有任务
- 任务变更时自动保存

## 代码质量改进

✅ **权限控制** - 从插件 manifest 读取 maxTasks 配置，限制每个插件的任务数量  
✅ **错误处理** - 单个任务恢复失败时继续处理其他任务，添加详细日志  
✅ **严格的 cron 验证** - 验证字段范围（分钟 0-59，小时 0-23 等），支持通配符、范围、列表、步长  
✅ **资源管理** - 取消任务时即使调度器移除失败也会清理任务列表，避免内存泄漏  
✅ **监听器初始化** - 模块加载时立即初始化监听器，无需等待首次调度  
✅ **异步操作一致性** - cancel 函数先等待后端成功再清理本地回调，保证状态一致  

## 后续优化

1. **任务历史** - 记录任务执行历史和结果
2. **错误重试** - 任务失败时自动重试
3. **时区支持** - 支持不同时区的任务调度
4. **UI 管理** - 在插件管理界面显示和管理定时任务

## 文件清单

### 后端
- `src-tauri/src/plugin_api/scheduler.rs` - Rust 实现
- `src-tauri/src/plugin_api/mod.rs` - 模块导出
- `src-tauri/src/lib.rs` - 命令注册

### SDK
- `plugins-sdk/src/api/scheduler.ts` - TypeScript API
- `plugins-sdk/src/index.ts` - 导出

### 示例插件
- `daily-quote-plugin/` - 每日一句示例
- `my-first-plugin/` - 番茄钟示例（不使用 scheduler）
