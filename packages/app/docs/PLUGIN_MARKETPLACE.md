# 插件市场功能文档

## 功能概述

实现了完整的插件市场功能，支持浏览、下载、安装插件，并区分本地开发版本和市场版本。

## 核心功能

### 1. 插件安装来源分离

**目录命名规则**：
```
plugins/
├── web-translate.20251202@local/     # 本地导入（符号链接）
└── web-translate.20251202@market/    # 市场下载（实际文件）
```

**数据结构**：
```rust
pub enum InstallSource {
    Local,
    Marketplace,
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub dir_name: String,  // 包含后缀
    pub install_source: InstallSource,
    // ...
}
```

**关键特性**：
- 同一插件的本地版本和市场版本可以共存
- 同时只能启用一个版本
- 设置文件共享（使用不带后缀的 plugin_id）

### 2. 下载和安装

**后端 Command**: `download_and_install_plugin(download_url, plugin_id)`

**安装流程**：
1. 从 URL 下载 ZIP 文件到临时目录
2. 解压并验证 manifest.json
3. 复制到 `plugins/{plugin_id}@market/`
4. 加载并初始化插件

**ZIP 文件要求**：
- 必须包含 manifest.json
- 支持直接包含文件或带顶层目录两种结构

**前端 API**：
```typescript
import { downloadAndInstallPlugin } from '$lib/api/marketplace';

await downloadAndInstallPlugin(downloadUrl, pluginId);
```

### 3. UI 功能

**两种安装方式**：
- 列表直接安装：插件卡片右下角的"安装"按钮
- 详情页安装：查看完整信息后安装

**状态显示**：
- 未安装：蓝色"安装"按钮
- 安装中：灰色"安装中..."按钮（禁用）
- 已安装：灰色"已安装"按钮（带勾选图标）

## 后端接口要求

```typescript
interface MarketplacePlugin {
  id: string;              // manifest 中的真实 ID
  name: string;
  description: string;
  author: string;
  icon: string;            // HTTP URL 或相对路径
  category: string;
  keywords: string[];
  downloads: number;
  stars: number;
  
  // 安装必需字段
  downloadUrl: string;     // ZIP 下载链接
  version?: string;
  size?: number;           // 字节
  releaseDate?: string;
}
```

## 调试指南

### 检查 Icon 不显示

1. 打开浏览器控制台，检查 `icon` 字段值
2. 相对路径会自动转换为 `plugin://{dir_name}/{icon}`
3. HTTP URL 直接显示

### 检查已安装状态

在控制台执行：
```javascript
// 查看市场插件 ID
console.log('Market plugins:', plugins.map(p => p.id));

// 查看已安装插件 ID
console.log('Installed IDs:', Array.from(installedPluginIds));

// 检查匹配
const isInstalled = installedPluginIds.has(plugins[0]?.id);
console.log('Is installed:', isInstalled);
```

**关键**：市场接口返回的 `id` 必须与 manifest 中的 `id` 一致。

### 检查安装按钮

1. 确认后端返回 `downloadUrl` 字段
2. 查看 Network 标签中的 `/api/v1/plugins` 响应
3. 检查控制台是否有错误信息

## 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 看不到安装按钮 | `downloadUrl` 为空 | 检查后端返回数据 |
| 已安装插件未置灰 | ID 不匹配 | 确保市场和本地 ID 一致 |
| 安装失败 | ZIP 格式错误 | 检查 manifest.json 是否存在 |
| 提示"插件已存在" | 已安装市场版本 | 先卸载再安装 |

## 安全特性

- 临时文件自动清理
- 路径遍历攻击防护
- manifest.json 格式验证
- 插件 ID 格式验证（禁止 `..`, `/`, `\`）

## 依赖

```toml
# Cargo.toml
zip = "2.2"
tempfile = "3.13"
reqwest = { version = "0.11", features = ["json"] }
```

## 后续优化

- [ ] 下载进度显示
- [ ] 版本更新检测
- [ ] 批量安装
- [ ] 校验和验证
- [ ] 断点续传
