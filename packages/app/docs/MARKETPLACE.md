# Onin 插件市场

> 插件市场完整指南：架构设计 + 开发者提交指南

## 📋 目录

- [架构设计](#架构设计)
- [开发者提交指南](#开发者提交指南)
- [实现状态](#实现状态)

---

## 架构设计

### 整体流程

```
开发者提交 PR → GitHub Registry → Webhook → 后端 API → Onin 应用
```

### manifest.json 格式

开发者只需提供最基本信息：

```json
{
  "title": "快速翻译",
  "description": "选中文本即可快速翻译",
  "author": "张三",
  "repo": "zhangsan/translator-plugin",
  "icon": "icon.svg",
  "category": "productivity",
  "keywords": ["翻译", "工具"]
}
```

其他信息（版本号、下载链接、License 等）后端自动从 GitHub API 获取。

### 后端 API

已实现接口：

- `GET /api/v1/plugins` - 插件列表（支持搜索、分类、分页）

返回格式：

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

---

## 开发者提交指南

### 1. 准备插件仓库

- ✅ 完整的 README.md
- ✅ LICENSE 文件
- ✅ 至少一个 Release（包含 `plugin.zip`）

### 2. 提交到 marketplace

```bash
# Fork onin-launcher/marketplace 仓库
git clone https://github.com/YOUR_USERNAME/marketplace.git
cd marketplace

# 创建插件目录
mkdir -p plugins/my-plugin

# 添加 manifest.json 和 icon.svg
# ...

# 提交 PR
git add plugins/my-plugin/
git commit -m "Add my-plugin"
git push origin main
```

### 3. 等待审核

- GitHub Actions 自动验证格式
- 维护者人工审核代码
- 合并后自动同步到插件市场

---

## 实现状态

### ✅ 已完成

**前端**:

- 插件市场 UI（列表、搜索、筛选）
- API 客户端（已适配后端格式）

**后端**:

- 插件列表 API
- 搜索和分类筛选

### 🚧 待实现

**前端**:

- 插件详情页
- 下载安装功能

**后端**:

- 插件详情 API
- GitHub Webhook 自动同步
- 下载统计

**GitHub Registry**:

- 创建 marketplace 仓库
- 配置 GitHub Actions 验证
- 配置 Webhook

---

## 快速测试

1. 启动后端（已完成）

   ```bash
   # 后端运行在 http://localhost:3001
   ```

2. 配置前端

   ```bash
   # .env
   VITE_MARKETPLACE_API_URL=http://localhost:3001
   ```

3. 启动前端

   ```bash
   npm run dev
   ```

4. 查看效果
   - 打开 Onin → 插件管理 → 插件市场

---

**相关文档**:

- 后端 API 文档: `BACKEND_API_MVP.md`
- 后端接口文档: `API.md`
