# Onin Monorepo

Onin 插件化桌面应用 - Tauri + SvelteKit

## 快速开始

```bash
# 安装
pnpm install

# 开发
pnpm dev              # Web 开发 (http://localhost:1420)
pnpm tauri dev        # Tauri 桌面应用（首次需要 3-10 分钟编译 Rust）
pnpm dev:demo         # SDK demo (http://localhost:5174)

# 构建
pnpm build            # 构建所有包
pnpm build:sdk        # 只构建 SDK

# 测试
pnpm test:sdk         # 测试 SDK
```

## 项目结构

```
packages/
├── app/              # 主应用 (Tauri + SvelteKit)
│   └── docs/         # 主应用设计文档
├── sdk/              # 插件 SDK (发布为 onin-sdk)
│   ├── docs/         # SDK 设计文档
│   └── examples/     # SDK 使用示例
└── demo/             # SDK 测试项目
```

## SDK 开发流程

1. 修改 SDK: `packages/sdk/src/`
2. 构建: `pnpm build:sdk`
3. 测试: `pnpm dev:demo`

## 常见问题

### Tauri 启动失败

```bash
# 清理构建缓存
rm -rf packages/app/src-tauri/target
pnpm tauri dev
```

### SDK 改动后 Demo 没更新

```bash
# 重新构建 SDK
pnpm build:sdk
```

### 依赖问题

```bash
# 重新安装
rm -rf node_modules packages/*/node_modules pnpm-lock.yaml
pnpm install
```

## 文档

### 主应用

- [API 文档](packages/app/docs/API.md)
- [插件系统](packages/app/docs/PLUGIN_COMMAND_USAGE.md)
- [窗口管理](packages/app/docs/WINDOW_LIFECYCLE_FINAL.md)
- [更多...](packages/app/docs/)

### SDK

- [SDK README](packages/sdk/README.md)
- [API 设计](packages/sdk/docs/)
- [使用示例](packages/sdk/examples/)

## License

MIT
