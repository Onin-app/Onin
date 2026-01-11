# Onin Plugin SDK

Onin 插件开发 SDK，提供完整的 API 支持。

## 安装

```bash
npm install onin-plugin-sdk
```

## 快速开始

```typescript
import { command, notification, storage } from 'onin-plugin-sdk';

// 注册动态命令
await command.register({
  code: 'hello-world',
  name: 'Hello World',
  keywords: [{ name: 'hello' }],
});

// 处理命令执行
await command.handle((code, args) => {
  if (code === 'hello-world') {
    notification.show({
      title: 'Hello!',
      body: 'Welcome to Onin Plugin SDK',
    });
  }
});
```

## API 概览

### Command API

```typescript
// 动态注册命令
await command.register({
  code: 'my-command',
  name: '我的命令',
  keywords: [{ name: '关键词' }],
  matches: [{ type: 'text', name: '文本匹配', min: 1 }],
});

// 处理命令执行
await command.handle((code, args) => {
  console.log('收到命令:', code, args);
});

// 移除动态命令
await command.remove('my-command');
```

### Notification API

```typescript
await notification.show({
  title: '标题',
  body: '内容',
});
```

### Storage API

```typescript
await storage.setItem('key', { data: 'value' });
const value = await storage.getItem('key');
await storage.removeItem('key');
```

### HTTP API

```typescript
const response = await http.get('https://api.example.com/data');
const data = await http.post('https://api.example.com/create', {
  body: JSON.stringify({ name: 'test' }),
});
```

### File System API

```typescript
await fs.writeFile('data.json', JSON.stringify(data));
const content = await fs.readFile('data.json');
const exists = await fs.exists('data.json');
```

### Dialog API

```typescript
const result = await dialog.confirm({
  title: '确认',
  message: '是否继续？',
});

const files = await dialog.open({
  multiple: true,
  filters: [{ name: 'Images', extensions: ['png', 'jpg'] }],
});
```

### Clipboard API

```typescript
const text = await clipboard.readText();
await clipboard.writeText('Hello');
```

## 类型定义

所有 API 都有完整的 TypeScript 类型定义，编辑器会自动提供智能提示。

## 更多文档

查看 [完整 API 文档](./docs) 获取更多信息。
