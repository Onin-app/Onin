# clipboard

剪贴板读写 API，支持文本、图片、文件等多种内容类型。

## 导入

```typescript
import { clipboard } from 'onin-plugin-sdk';
```

## API

### 文本操作

```typescript
// 读取剪贴板文本
const text = await clipboard.readText(); // string | null

// 写入文本到剪贴板
await clipboard.writeText('Hello, World!');
```

### 图片操作

```typescript
// 读取剪贴板图片（返回 base64 字符串）
const imageBase64 = await clipboard.readImage(); // string | null

// 写入图片到剪贴板（传入 base64）
await clipboard.writeImage(base64String);
```

### 文件操作

```typescript
// 读取剪贴板中的文件列表
const files = await clipboard.readFiles(); // string[] | null（文件路径列表）
```

### 元信息

```typescript
// 读取剪贴板元信息（判断当前内容类型）
const meta = await clipboard.readMeta();
// {
//   contentType: 'text' | 'image' | 'files' | 'unknown',
//   hasText: boolean,
//   hasImage: boolean,
//   hasFiles: boolean,
// }
```

## 完整示例

```typescript
import { clipboard, notification } from 'onin-plugin-sdk';

// 处理任意类型的剪贴板内容
async function processClipboard() {
  const meta = await clipboard.readMeta();

  if (meta.hasText) {
    const text = await clipboard.readText();
    console.log('剪贴板文本:', text);
    return text;
  }

  if (meta.hasImage) {
    const image = await clipboard.readImage();
    console.log('剪贴板图片 (base64):', image?.substring(0, 50) + '...');
    return image;
  }

  if (meta.hasFiles) {
    const files = await clipboard.readFiles();
    console.log('剪贴板文件:', files);
    return files;
  }

  console.log('剪贴板为空');
  return null;
}
```
