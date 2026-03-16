# storage

持久化键值存储 API，每个插件有独立隔离的存储空间，数据在应用重启后保持。

## 导入

```typescript
import { storage } from 'onin-sdk';
```

> **所需权限**：`"storage": { "enable": true }`

## API

### `storage.setItem(key, value)`

存储一个值。

```typescript
await storage.setItem('theme', 'dark');
await storage.setItem('user', { name: 'John', level: 10 });
```

### `storage.getItem<T>(key)`

读取一个值，不存在时返回 `null`。

```typescript
const theme = await storage.getItem<string>('theme'); // 'dark' | null
const user = await storage.getItem<{ name: string }>('user');
```

### `storage.removeItem(key)`

删除一个值。

```typescript
await storage.removeItem('sessionToken');
```

### `storage.clear()`

清空该插件的所有存储数据（不可恢复）。

```typescript
await storage.clear();
```

### `storage.keys()`

获取所有已存储的 key 列表。

```typescript
const allKeys = await storage.keys(); // string[]
```

### `storage.setItems(items)`

批量写入多个值，比循环调用 `setItem` 更高效。

```typescript
await storage.setItems({
  theme: 'dark',
  fontSize: 14,
  language: 'zh-CN',
});
```

### `storage.getItems<T>(keys)`

批量读取多个值。

```typescript
const result = await storage.getItems(['theme', 'fontSize']);
console.log(result.theme); // 'dark'
console.log(result.fontSize); // 14
```

### `storage.getAll()`

获取该插件的全量存储数据。

```typescript
const allData = await storage.getAll();
```

### `storage.setAll(data)`

用新数据替换全量存储（原子操作，先清空再写入）。

```typescript
await storage.setAll({ version: '2.0', migrated: true });
```

## 完整示例

```typescript
import { storage } from 'onin-sdk';

interface AppConfig {
  apiKey: string;
  theme: 'light' | 'dark';
  language: string;
}

// 读取配置，不存在时使用默认值
const config = (await storage.getItem<AppConfig>('config')) ?? {
  apiKey: '',
  theme: 'dark',
  language: 'zh-CN',
};

// 修改后写回
config.theme = 'light';
await storage.setItem('config', config);
```
