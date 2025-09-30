# 权限系统重构总结

## 重构目标

将原有的扁平化权限结构重构为更灵活、更直观的命名空间结构。

## 主要变更

### 1. 权限结构变更

**之前的结构：**
```json
{
  "permissions": {
    "network": ["https://api.example.com/*"]
  }
}
```

**新的结构：**
```json
{
  "permissions": {
    "http": {
      "enable": true,
      "allowUrls": ["https://api.example.com/*"],
      "timeout": 30000,
      "maxRetries": 3
    },
    "storage": {
      "enable": true,
      "local": true,
      "session": false,
      "maxSize": "10MB"
    },
    "notification": {
      "enable": true,
      "sound": true,
      "badge": false
    },
    "command": {
      "enable": true,
      "allowCommands": ["my-command-*"],
      "maxExecutionTime": 5000
    }
  }
}
```

### 2. 代码变更

#### Rust 后端 (src-tauri/src/plugin_manager.rs)
- 重构 `PluginPermissions` 结构体
- 新增 `HttpPermission`、`StoragePermission`、`NotificationPermission`、`CommandPermission` 结构体
- 支持更细粒度的权限控制

#### TypeScript SDK (plugins-sdk/src/)
- 更新错误信息，指向新的权限路径
- 新增类型定义文件 `types/permissions.ts`
- 导出权限相关的 TypeScript 类型

#### 权限检查逻辑 (src-tauri/src/plugin_api/request.rs)
- 更新权限检查逻辑，支持 `enable` 开关
- 更新错误信息，指向 `permissions.http.allowUrls`

### 3. 优势

1. **更直观** - 每个权限模块都有明确的开关和配置选项
2. **更灵活** - 可以为每个权限添加具体的配置参数
3. **更易扩展** - 新增权限选项不会破坏现有结构
4. **更好的类型安全** - TypeScript 提供完整的类型检查
5. **向后兼容** - 可以通过默认值保持兼容性

### 4. 示例文件

- `example-plugin-manifest.json` - 完整的权限配置示例
- `example-plugin.ts` - 插件代码使用示例
- `plugins-sdk/src/types/permissions.ts` - TypeScript 类型定义

### 5. 迁移指南

对于现有插件，需要将：
```json
"permissions": {
  "network": ["https://api.example.com"]
}
```

更新为：
```json
"permissions": {
  "http": {
    "enable": true,
    "allowUrls": ["https://api.example.com"]
  }
}
```

## 下一步

1. 实现其他权限模块的具体逻辑（storage、notification、command）
2. 添加权限验证的单元测试
3. 更新插件开发文档和示例
4. 考虑添加权限配置的验证工具