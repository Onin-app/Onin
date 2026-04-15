# toast

用于在插件窗口内显示非阻塞的瞬时提示信息。

## 导入

```typescript
import { toast } from 'onin-sdk';
```

> **所需权限**：此 API 目前无需特殊权限即可使用。

## API

### `toast.show(message, options?)`

显示一条基础提示。

```typescript
await toast.show('操作完成');
```

**参数：**

| 字段       | 类型     | 必填 | 说明                                                                 |
| ---------- | -------- | ---- | -------------------------------------------------------------------- |
| `message`  | `string` | ✅   | 提示正文                                                             |
| `options`  | `object` | ❌   | 配置项                                                               |
| `options.kind` | `string` | ❌   | 提示类型：`'default' \| 'success' \| 'error' \| 'warning' \| 'info'` |
| `options.duration` | `number` | ❌   | 持续时间（毫秒），默认 4000ms                                        |

### `toast.success(message, options?)`

显示成功提示。

```typescript
await toast.success('保存成功！');
```

### `toast.error(message, options?)`

显示错误提示。

```typescript
await toast.error('网络连接失败', { duration: 5000 });
```

### `toast.warning(message, options?)`

显示警告提示。

```typescript
await toast.warning('存储空间即将存满');
```

### `toast.info(message, options?)`

显示通知提示。

```typescript
await toast.info('已进入开发者模式');
```

## 示例

```typescript
import { toast, http } from 'onin-sdk';

async function uploadData() {
  try {
    await toast.info('正在同步...', { duration: 2000 });
    const response = await http.post('/sync', { data: '...' });
    
    if (response.status === 200) {
      await toast.success('同步成功！');
    } else {
      await toast.error(`同步失败: ${response.status}`);
    }
  } catch (e) {
    await toast.error('连接服务器异常');
  }
}
```
