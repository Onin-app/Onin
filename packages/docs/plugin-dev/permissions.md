# 权限配置

Onin 采用最小权限原则，插件只能使用在 `manifest.json` 中显式声明的能力。

## 配置结构

在 `manifest.json` 的 `permissions` 字段中声明权限：

```json
{
  "permissions": {
    "http": {
      "enable": true,
      "allowUrls": ["https://api.example.com"]
    },
    "storage": {
      "enable": true
    },
    "notification": {
      "enable": true
    },
    "command": {
      "enable": true
    },
    "scheduler": {
      "enable": true
    }
  }
}
```

## 权限说明

### http — 网络请求

允许插件通过 `http` API 发起网络请求。

```json
{
  "http": {
    "enable": true,
    "allowUrls": ["https://api.example.com", "https://api.other.com"],
    "timeout": 10000,
    "maxRetries": 3
  }
}
```

| 字段         | 类型       | 说明                                              |
| ------------ | ---------- | ------------------------------------------------- |
| `enable`     | `boolean`  | 是否启用                                          |
| `allowUrls`  | `string[]` | 允许访问的 URL 前缀列表，不在列表中的域名会被拒绝 |
| `timeout`    | `number`   | 请求超时时间（毫秒），默认 30000                  |
| `maxRetries` | `number`   | 最大重试次数                                      |

### storage — 持久化存储

允许插件使用 `storage` API 读写本地数据。

```json
{
  "storage": {
    "enable": true,
    "local": true,
    "maxSize": "10MB"
  }
}
```

| 字段      | 类型      | 说明                      |
| --------- | --------- | ------------------------- |
| `enable`  | `boolean` | 是否启用                  |
| `local`   | `boolean` | 允许本地持久化存储        |
| `maxSize` | `string`  | 最大存储容量，如 `"10MB"` |

### notification — 系统通知

允许插件使用 `notification` API 发送系统通知。

```json
{
  "notification": {
    "enable": true,
    "sound": true,
    "badge": false
  }
}
```

### command — 系统命令

允许插件使用 `command` API 执行系统命令（如 shell 命令）。

```json
{
  "command": {
    "enable": true,
    "allowCommands": ["open", "curl"],
    "maxExecutionTime": 30000
  }
}
```

> ⚠️ **注意**：此权限较为敏感，请仅在必要时申请，并严格限制 `allowCommands` 列表。

### scheduler — 定时任务

允许插件使用 `scheduler` API 注册定时任务。

```json
{
  "scheduler": {
    "enable": true,
    "maxTasks": 5
  }
}
```

| 字段       | 类型      | 说明                 |
| ---------- | --------- | -------------------- |
| `enable`   | `boolean` | 是否启用             |
| `maxTasks` | `number`  | 允许注册的最大任务数 |

## 权限被拒时的处理

当插件调用未声明的权限时，SDK 会抛出 `PERMISSION_DENIED` 错误：

```typescript
import { http } from 'onin-plugin-sdk';

try {
  const response = await http.get('https://api.example.com/data');
} catch (error) {
  if (error.code === 'PERMISSION_DENIED') {
    console.error(
      '没有网络请求权限，请检查 manifest.json 中的 permissions.http 配置',
    );
  }
}
```
