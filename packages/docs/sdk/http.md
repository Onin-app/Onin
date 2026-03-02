# http

HTTP 网络请求 API，支持 GET、POST、PUT、PATCH、DELETE 等标准方法。

## 导入

```typescript
import { http } from 'onin-plugin-sdk';
```

> **所需权限**：`"http": { "enable": true, "allowUrls": ["https://api.example.com"] }`

## API

### `http.get<T>(url, options?)`

发送 GET 请求。

```typescript
const { body } = await http.get<{ title: string }>(
  'https://api.example.com/posts/1',
);
console.log(body.title);
```

### `http.post<T>(url, body?, options?)`

发送 POST 请求，`body` 支持对象（自动序列化为 JSON）。

```typescript
const { body } = await http.post('https://api.example.com/users', {
  name: 'John',
  email: 'john@example.com',
});
```

### `http.put<T>(url, body?, options?)`

发送 PUT 请求。

### `http.patch<T>(url, body?, options?)`

发送 PATCH 请求。

### `http.delete<T>(url, options?)`

发送 DELETE 请求。

### `http.request<T>(options)`

通用请求方法，完整控制所有参数。

```typescript
const response = await http.request<Blob>({
  url: 'https://example.com/file.pdf',
  method: 'GET',
  responseType: 'arraybuffer',
  timeout: 30000,
  headers: {
    Authorization: `Bearer ${token}`,
  },
});
```

**`RequestOptions` 字段：**

| 字段           | 类型                                | 说明                      |
| -------------- | ----------------------------------- | ------------------------- |
| `url`          | `string`                            | 请求 URL                  |
| `method`       | `HttpMethod`                        | HTTP 方法，默认 `'GET'`   |
| `headers`      | `Record<string, string>`            | 请求头                    |
| `body`         | `string \| object \| ArrayBuffer`   | 请求体                    |
| `timeout`      | `number`                            | 超时时间（毫秒）          |
| `responseType` | `'json' \| 'text' \| 'arraybuffer'` | 响应体格式，默认 `'json'` |

**`Response<T>` 结构：**

```typescript
interface Response<T> {
  status: number; // HTTP 状态码
  statusText: string; // 状态文本
  headers: Record<string, string>;
  body: T; // 响应体
}
```

## 错误处理

```typescript
import { http } from 'onin-plugin-sdk';

try {
  const response = await http.get('https://api.example.com/data');
  console.log(response.body);
} catch (error: any) {
  switch (error.code) {
    case 'HTTP_NETWORK_ERROR':
      console.error('网络连接失败');
      break;
    case 'HTTP_TIMEOUT':
      console.error('请求超时');
      break;
    case 'HTTP_HTTP_ERROR':
      console.error(`HTTP 错误: ${error.context?.status}`);
      break;
    case 'PERMISSION_DENIED':
      console.error('没有网络请求权限或域名不在白名单');
      break;
  }
}
```
