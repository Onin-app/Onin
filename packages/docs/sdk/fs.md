# fs

文件系统 API，提供对插件独立沙盒目录的读写能力。所有路径均相对于插件数据目录。

## 导入

```typescript
import { fs } from 'onin-plugin-sdk';
```

> ⚠️ **路径沙盒**：所有操作严格限制在插件的数据目录内，无法访问系统其他文件。

## API

| 方法                             | 说明                         |
| -------------------------------- | ---------------------------- |
| `fs.readFile(path)`              | 读取文本文件（UTF-8）        |
| `fs.writeFile(path, content)`    | 写入文本文件（不存在则创建） |
| `fs.exists(path)`                | 检查文件或目录是否存在       |
| `fs.createDir(path, recursive?)` | 创建目录，默认递归创建       |
| `fs.listDir(path)`               | 列出目录内容                 |
| `fs.deleteFile(path)`            | 删除文件                     |
| `fs.deleteDir(path, recursive?)` | 删除目录，默认不递归         |
| `fs.getFileInfo(path)`           | 获取文件元信息               |
| `fs.copyFile(src, dest)`         | 复制文件                     |
| `fs.moveFile(src, dest)`         | 移动/重命名文件              |

## FileInfo 结构

```typescript
interface FileInfo {
  name: string; // 文件名
  path: string; // 完整路径
  isFile: boolean;
  isDirectory: boolean;
  size: number; // 字节数
  modifiedTime: number; // Unix 时间戳（毫秒）
  createdTime: number;
}
```

## 示例

```typescript
import { fs } from 'onin-plugin-sdk';

// 读写 JSON 配置文件
const CONFIG_FILE = 'config.json';

async function loadConfig() {
  if (await fs.exists(CONFIG_FILE)) {
    const content = await fs.readFile(CONFIG_FILE);
    return JSON.parse(content);
  }
  return { version: 1 };
}

async function saveConfig(config: object) {
  await fs.writeFile(CONFIG_FILE, JSON.stringify(config, null, 2));
}

// 目录操作
await fs.createDir('logs');
const files = await fs.listDir('.');
for (const item of files) {
  console.log(item.name, item.isDirectory ? '[目录]' : `${item.size} bytes`);
}

// 文件管理
await fs.copyFile('data.json', 'backups/data.json');
await fs.moveFile('old.txt', 'new.txt');
await fs.deleteFile('temp.txt');
```
