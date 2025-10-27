# Scheduler API 测试指南

## 修复的问题清单

### ✅ 1. 核心功能缺失
**问题**: 调度器系统没有实际执行定时任务的逻辑  
**修复**: 
- 集成 `tokio-cron-scheduler` 库
- 实现完整的任务调度和执行逻辑
- 在 `schedule_task` 中创建真实的 Job

### ✅ 2. 跨端通信问题
**问题**: 缺少后端触发前端任务执行的机制  
**修复**:
- 后端通过 `app_handle.emit("scheduler:execute-task")` 发送事件
- 前端通过 `listen()` 监听事件并执行回调
- 在 `setupTaskListener()` 中建立监听

### ✅ 3. 数据持久化问题
**问题**: 任务仅存储在内存中，应用重启后会丢失  
**修复**:
- 使用 `tauri-plugin-store` 持久化任务
- 实现 `load_from_store()` 和 `save_to_store()`
- 应用启动时自动恢复所有任务

### ✅ 4. 参数验证缺失
**问题**: TypeScript 端缺少对时间格式和 cron 表达式的验证  
**修复**:
- 添加 `validateTimeFormat()` 验证 HH:MM 格式
- 添加 `validateCronFormat()` 验证 cron 表达式
- 在 `daily()`, `weekly()` 等方法中进行参数验证

### ✅ 5. Cron 解析不完整
**问题**: Rust 端的 cron 解析函数仅做格式检查，无实际调度逻辑  
**修复**:
- 使用 `tokio-cron-scheduler` 的 `Job::new_async()` 处理 cron
- 库内部完整支持标准 cron 表达式
- 添加 `validate_cron()` 进行基本格式验证

## 测试步骤

### 1. 编译测试

```bash
cd src-tauri
cargo build
```

### 2. 功能测试

#### 测试 1: 基本任务注册
```typescript
// 在插件中
await scheduler.daily('test-task', '14:30', async () => {
  console.log('Task executed at 14:30');
});
```

#### 测试 2: 立即触发（用于测试）
```typescript
// 使用 cron 表达式设置为下一分钟
const now = new Date();
const nextMinute = now.getMinutes() + 1;
await scheduler.schedule('test-now', `${nextMinute} * * * *`, async () => {
  await notification.show({
    title: '测试',
    body: '任务执行成功！'
  });
});
```

#### 测试 3: 任务取消
```typescript
await scheduler.cancel('test-task');
```

#### 测试 4: 列出任务
```typescript
const tasks = await scheduler.list();
console.log('当前任务:', tasks);
```

#### 测试 5: 应用重启恢复
1. 注册一个任务
2. 关闭应用
3. 重新打开应用
4. 检查任务是否仍然存在并正常执行

### 3. 每日一句插件测试

```bash
cd daily-quote-plugin
pnpm install
# 打包为 zip
tar -a -c -f daily-quote.zip *
```

然后在应用中导入并测试：
1. 导入插件
2. 手动触发 `quote-now` 命令验证功能
3. 等待第二天早上 8 点验证自动执行
4. 或修改时间为当前时间+1分钟快速测试

### 4. 验证持久化

检查文件是否创建：
```
%APPDATA%/com.baize.dev/scheduler.json
```

内容应该类似：
```json
{
  "tasks": [
    {
      "id": "morning-quote",
      "plugin_id": "com.myapp.daily-quote",
      "cron": "0 8 * * *",
      "enabled": true
    }
  ]
}
```

## 常见问题

### Q: 任务没有执行？
A: 检查：
1. 控制台是否有错误信息
2. cron 表达式是否正确
3. 回调函数是否正确注册
4. 事件监听器是否设置

### Q: 应用重启后任务丢失？
A: 检查：
1. `scheduler.json` 文件是否存在
2. 控制台是否有加载错误
3. `init_scheduler` 是否被调用

### Q: 如何快速测试定时任务？
A: 使用当前时间+1分钟的 cron 表达式：
```typescript
const now = new Date();
const minute = (now.getMinutes() + 1) % 60;
const hour = minute === 0 ? (now.getHours() + 1) % 24 : now.getHours();
await scheduler.schedule('quick-test', `${minute} ${hour} * * *`, callback);
```

## 性能考虑

- 每个任务占用约 1KB 内存
- 建议每个插件最多 5-10 个任务
- 调度器使用异步执行，不会阻塞主线程
- 任务回调应该快速完成，避免长时间运行

## 安全考虑

- 任务回调在插件沙箱中执行
- 遵循插件的权限限制
- 建议在 manifest.json 中限制 `maxTasks`
