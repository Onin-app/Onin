# 插件下载和安装功能

## 概述

实现了从插件市场直接下载和安装插件的完整功能。

## 功能特性

### 后端 (Rust)

**新增 Tauri Command**: `download_and_install_plugin`

功能：
1. 从指定 URL 下载插件 ZIP 文件
2. 解压到临时目录
3. 验证 manifest.json
4. 复制到插件目录
5. 加载并初始化插件

**依赖**:
- `zip = "2.2"` - ZIP 文件解压
- `tempfile = "3.13"` - 临时目录管理
- `reqwest` - HTTP 下载（已有）

### 前端 (TypeScript/Svelte)

**API 函数**: `downloadAndInstallPlugin(downloadUrl, pluginId)`

**UI 组件**: `PluginCard.svelte`
- 显示插件信息
- 一键安装按钮
- 安装状态反馈（安装中/已安装）

## 使用方法

### 1. 后端 API 提供下载信息

插件详情接口需要返回：

```json
{
  "data": {
    "id": "translate",
    "name": "Translate",
    "downloadUrl": "https://github.com/b-yp/onin-web-translate/releases/download/v1.0.0/onin-web-translate-1.0.0.zip",
    "version": "v1.0.0",
    "size": 1048576,
    // ... 其他字段
  }
}
```

### 2. 前端调用安装

```typescript
import { downloadAndInstallPlugin } from '$lib/api/marketplace';

// 安装插件
await downloadAndInstallPlugin(
  'https://github.com/.../plugin.zip',
  'plugin-id'
);
```

### 3. UI 组件使用

```svelte
<PluginCard 
  plugin={plugin} 
  onclick={() => showDetail(plugin)}
  oninstall={() => refreshPluginList()}
/>
```

## ZIP 文件结构要求

插件 ZIP 文件必须包含 `manifest.json`，支持两种结构：

**结构 1: 直接包含文件**
```
plugin.zip
├── manifest.json
├── icon.svg
├── package.json
└── dist/
    ├── index.html
    └── assets/
```

**结构 2: 带顶层目录**
```
plugin.zip
└── plugin-name/
    ├── manifest.json
    ├── icon.svg
    ├── package.json
    └── dist/
        ├── index.html
        └── assets/
```

系统会自动识别并提取正确的插件根目录。

## 安装流程

1. **下载**: 从 `downloadUrl` 下载 ZIP 文件到临时目录
2. **解压**: 解压 ZIP 文件
3. **验证**: 检查 manifest.json 是否存在且格式正确
4. **ID 验证**: 确保 manifest 中的 ID 与请求的 ID 一致
5. **复制**: 将插件文件复制到 `{app_data}/plugins/{plugin_id}/`
6. **加载**: 加载插件到内存并初始化生命周期
7. **完成**: 插件可立即使用

## 错误处理

- 下载失败：显示网络错误
- ZIP 格式错误：提示文件损坏
- manifest.json 缺失：提示无效插件
- ID 不匹配：提示插件信息不一致
- 插件已存在：提示先卸载旧版本

## 安全特性

- 临时文件自动清理
- 路径遍历攻击防护
- manifest.json 格式验证
- 插件 ID 格式验证（禁止 `..`, `/`, `\`）

## 示例

### 完整安装流程

```typescript
// 1. 获取插件详情
const plugin = await fetchPluginDetail('translate');

// 2. 检查是否有下载链接
if (plugin.downloadUrl) {
  try {
    // 3. 下载并安装
    await downloadAndInstallPlugin(plugin.downloadUrl, plugin.id);
    
    // 4. 刷新插件列表
    await refreshPlugins();
    
    console.log('插件安装成功！');
  } catch (error) {
    console.error('安装失败:', error);
  }
}
```

## 注意事项

1. **网络要求**: 需要能访问 GitHub 或插件托管服务器
2. **磁盘空间**: 确保有足够空间存储插件
3. **权限**: Windows 可能需要开发者模式或管理员权限（用于符号链接）
4. **版本管理**: 暂不支持自动更新，需先卸载再安装新版本

## 后续优化

- [ ] 下载进度显示
- [ ] 断点续传
- [ ] 版本更新检测
- [ ] 批量安装
- [ ] 安装队列管理
- [ ] 校验和验证（checksum）
