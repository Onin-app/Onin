# Monorepo 实施方案

## 1. 目录结构调整

```
baize/
├── packages/
│   └── sdk/          # SDK核心包
│       ├── src/      # 源代码
│       ├── tests/    # 单元测试
│       └── package.json
├── src/             # 主应用代码
├── package.json     # 根workspace配置
└── pnpm-workspace.yaml
```

## 2. 关键配置

### pnpm-workspace.yaml
```yaml
packages:
  - 'packages/*'
  - 'src'
```

### 根package.json
```json
{
  "private": true,
  "workspaces": [
    "packages/*",
    "src"
  ]
}
```

### SDK包配置示例
```json
{
  "name": "@baize/plugin-sdk",
  "version": "1.0.0-alpha.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsup src/index.ts --format esm,cjs --dts",
    "dev": "tsup src/index.ts --format esm,cjs --dts --watch"
  }
}
```

## 3. 开发工作流

1. **依赖管理**：
   - 共享依赖提升到根目录
   - 包专用依赖在各自package.json声明

2. **构建系统**：
   - 使用Tsup进行SDK打包
   - Vite负责主应用构建
   - 共享tsconfig.base.json

3. **版本管理**：
   - Changesets管理多包版本
   - 统一发布流程

## 4. 优势

- 代码共享方便
- 依赖管理清晰
- 构建隔离但可协同
- 独立版本控制