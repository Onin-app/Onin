# Onin Marketplace API 文档

> 版本：v1.0.0  
> Base URL：`http://localhost:3001/api/v1`

## 📋 目录

- [认证](#认证)
- [响应格式](#响应格式)
- [错误码](#错误码)
- [接口列表](#接口列表)
  - [获取插件列表](#获取插件列表)
  - [获取插件详情](#获取插件详情)

---

## 认证

当前版本无需认证（后续版本会添加 API Key）。

---

## 响应格式

### 成功响应

所有成功的响应遵循以下格式：

```json
{
  "data": [...],
  "meta": {
    "total": 100,
    "page": 1,
    "limit": 20,
    "totalPages": 5
  }
}
```

### 错误响应

所有错误响应遵循以下格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "status": 400
  }
}
```

---

## 错误码

| 错误码 | HTTP 状态码 | 说明 |
|--------|------------|------|
| `INVALID_PARAMS` | 400 | 请求参数无效 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

---

## 接口列表

### 获取插件列表

获取所有可用的插件列表，支持搜索、分类筛选和分页。

**请求**

```http
GET /api/v1/plugins
```

**查询参数**

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `page` | number | 否 | 1 | 页码（>= 1） |
| `limit` | number | 否 | 20 | 每页数量（1-100） |
| `keyword` | string | 否 | - | 搜索关键词（匹配标题、描述、关键词） |
| `category` | string | 否 | - | 分类筛选 |

**分类列表**

- `productivity` - 效率工具
- `utility` - 实用工具
- `entertainment` - 娱乐
- `development` - 开发工具

**请求示例**

```bash
# 获取所有插件（第一页）
curl "http://localhost:3001/api/v1/plugins"

# 搜索包含"翻译"的插件
curl "http://localhost:3001/api/v1/plugins?keyword=翻译"

# 获取效率工具分类
curl "http://localhost:3001/api/v1/plugins?category=productivity"

# 分页查询
curl "http://localhost:3001/api/v1/plugins?page=2&limit=10"

# 组合查询
curl "http://localhost:3001/api/v1/plugins?category=productivity&keyword=翻译&page=1&limit=20"
```

**成功响应**

```json
{
  "data": [
    {
      "id": "translate",
      "name": "Translate",
      "description": "A aggregated translate plugin running on Onin",
      "author": "b-yp",
      "repository": "https://github.com/b-yp/onin-web-translate",
      "icon": "https://raw.githubusercontent.com/Onin-app/marketplace/master/packages/translate/icon.svg",
      "category": "productivity",
      "keywords": ["翻译", "工具", "效率"],
      "addedAt": 1765358330852,
      "downloads": 1234,
      "stars": 56,
      "downloadUrl": "https://github.com/b-yp/onin-web-translate/releases/download/v1.0.0/onin-web-translate-1.0.0.zip",
      "version": "v1.0.0",
      "size": 1048576,
      "checksum": "",
      "releaseDate": "2024-12-11T10:00:00Z"
    }
  ],
  "meta": {
    "total": 1,
    "page": 1,
    "limit": 20,
    "totalPages": 1
  }
}
```

**字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 插件唯一标识 |
| `name` | string | 插件名称 |
| `description` | string | 插件描述 |
| `author` | string | 作者 |
| `repository` | string | GitHub 仓库地址 |
| `icon` | string | 插件图标 URL |
| `category` | string | 分类 |
| `keywords` | string[] | 关键词列表 |
| `addedAt` | number | 添加时间（Unix 时间戳） |
| `downloads` | number | 下载次数（从 GitHub Releases 统计，无数据时为 0） |
| `stars` | number | GitHub Star 数（无数据时为 0） |
| `downloadUrl` | string | 下载链接（无 release 时为空字符串） |
| `version` | string | 版本号（无 release 时为 "N/A"） |
| `size` | number | 文件大小（字节，无 release 时为 0） |
| `checksum` | string | 文件校验和（SHA256，暂时为空） |
| `releaseDate` | string | 发布日期（ISO 8601 格式，无 release 时为空字符串） |

**错误响应**

```json
{
  "error": {
    "code": "INVALID_PARAMS",
    "message": "Invalid page or limit parameter",
    "status": 400
  }
}
```

**状态码**

- `200` - 成功
- `400` - 参数错误
- `500` - 服务器错误

---

### 获取插件详情

获取指定插件的详细信息。

**请求**

```http
GET /api/v1/plugins/:id
```

**路径参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 插件 ID |

**请求示例**

```bash
# 获取插件详情
curl "http://localhost:3001/api/v1/plugins/translate"
```

**成功响应**

```json
{
  "data": {
    "id": "translate",
    "folder": "onin-web-translate",
    "name": "Translate",
    "description": "A aggregated translate plugin running on Onin",
    "author": "b-yp",
    "repository": "https://github.com/b-yp/onin-web-translate",
    "icon": "https://raw.githubusercontent.com/Onin-app/marketplace/master/packages/onin-web-translate/icon.svg",
    "category": "productivity",
    "keywords": ["翻译", "工具", "效率"],
    "addedAt": 1765358330852,
    "license": "MIT",
    "downloads": 1234,
    "stars": 56,
    "downloadUrl": "https://github.com/b-yp/onin-web-translate/releases/download/v1.0.0/onin-web-translate-1.0.0.zip",
    "version": "v1.0.0",
    "size": 1048576,
    "checksum": "",
    "releaseDate": "2024-12-11T10:00:00Z",
    "releaseNotes": "## What's Changed\n- Initial release\n- Add translation feature"
  }
}
```

**字段说明**

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 插件唯一标识 |
| `folder` | string | 插件文件夹名称 |
| `name` | string | 插件名称 |
| `description` | string | 插件描述 |
| `author` | string | 作者 |
| `repository` | string | GitHub 仓库地址 |
| `icon` | string | 插件图标 URL |
| `category` | string | 分类 |
| `keywords` | string[] | 关键词列表 |
| `addedAt` | number | 添加时间（Unix 时间戳） |
| `license` | string | 开源协议 |
| `downloads` | number | 下载次数（从 GitHub Releases 统计，无数据时为 0） |
| `stars` | number | GitHub Star 数（无数据时为 0） |
| `downloadUrl` | string | 下载链接 |
| `version` | string | 版本号 |
| `size` | number | 文件大小（字节） |
| `checksum` | string | 文件校验和（SHA256，暂时为空） |
| `releaseDate` | string | 发布日期（ISO 8601 格式） |
| `releaseNotes` | string | 更新说明（Markdown 格式） |

**错误响应**

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Plugin with id 'xxx' not found",
    "status": 404
  }
}
```

**状态码**

- `200` - 成功
- `404` - 插件不存在
- `500` - 服务器错误

---

## 更新日志

### v1.0.0 (2024-12-11)

- ✅ 插件列表接口
- ✅ 插件详情接口
- ✅ 搜索功能
- ✅ 分类筛选
- ✅ 分页支持
- ✅ RESTful 响应格式

---

## 联系方式

- GitHub: [Onin-app/marketplace](https://github.com/Onin-app/marketplace)
- Issues: [提交问题](https://github.com/Onin-app/marketplace/issues)
