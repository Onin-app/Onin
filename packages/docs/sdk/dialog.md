# dialog

系统对话框 API，提供文件选择、目录选择、保存对话框和确认框。

## 导入

```typescript
import { dialog } from 'onin-sdk';
```

## API

### `dialog.open(options?)`

打开文件/目录选择对话框。

```typescript
// 选择单个文件
const filePath = await dialog.open();
// string | null

// 选择多个文件，并过滤格式
const files = await dialog.open({
  multiple: true,
  filters: [
    { name: '图片', extensions: ['png', 'jpg', 'webp'] },
    { name: '所有文件', extensions: ['*'] },
  ],
});
// string[] | null

// 选择目录
const dir = await dialog.open({ directory: true });
// string | null
```

**`OpenDialogOptions` 字段：**

| 字段          | 类型             | 说明                       |
| ------------- | ---------------- | -------------------------- |
| `multiple`    | `boolean`        | 是否允许多选，默认 `false` |
| `directory`   | `boolean`        | 是否选择目录，默认 `false` |
| `filters`     | `DialogFilter[]` | 文件类型过滤器             |
| `defaultPath` | `string`         | 默认打开路径               |
| `title`       | `string`         | 对话框标题                 |

### `dialog.save(options?)`

打开文件保存对话框。

```typescript
const savePath = await dialog.save({
  defaultPath: 'output.json',
  filters: [{ name: 'JSON', extensions: ['json'] }],
});

if (savePath) {
  await fs.writeFile(savePath, JSON.stringify(data));
}
```

### `dialog.confirm(options)`

显示确认对话框，返回用户是否点击了「确认」。

```typescript
const confirmed = await dialog.confirm({
  title: '删除确认',
  message: '确定要删除所有数据吗？此操作不可撤销。',
});

if (confirmed) {
  await storage.clear();
}
```

## 完整示例

```typescript
import { dialog, fs, notification } from 'onin-sdk';

// 导入文件流程
async function importFile() {
  const path = await dialog.open({
    title: '选择要导入的文件',
    filters: [{ name: 'JSON 文件', extensions: ['json'] }],
  });

  if (!path) return; // 用户取消了

  const content = await fs.readFile(path as string);
  const data = JSON.parse(content);

  // 处理数据...
  await notification.show({ title: '导入成功', body: `已导入 ${path}` });
}
```
