# Onin Plugin SDK Demo

用于验证 Onin 插件 SDK 核心能力和新插件入口模型的示例项目。

## 功能

本项目覆盖一组代表性的 SDK 能力：

- Setup / background entry
- Window events
- Command registration and handling
- Storage and settings
- Clipboard / file system / dialog / HTTP
- Scheduler
- AI availability and simple ask flow

## 开发

```bash
# 在根目录运行
pnpm dev:demo

# 或者在当前目录
pnpm dev
```

## 构建

```bash
pnpm build:demo
```

`plugin-demo` 现在采用和脚手架一致的单源码声明、双产物输出模型：

- `src/plugin.ts` 声明 `setup` 和 `mount`
- `src/main.ts` 通过 `mountPlugin` 托管 UI 生命周期
- `scripts/build.mjs` 从单主入口自动产出 `dist/index.html` 和 `dist/background.js`

## 目录结构

```
packages/plugin-demo/
├── index.html           # 主页面
├── package.json         # 包配置
├── README.md            # 本文档
├── manifest.json        # 插件清单（含后台入口）
├── scripts/
│   └── build.mjs        # 双产物构建脚本
├── tsconfig.json        # TypeScript 配置
├── vite.config.ts       # Vite 配置
└── src/
    ├── main.ts          # UI 入口薄包装
    ├── plugin.ts        # 插件声明
    ├── style.css        # 样式
    └── ui.ts            # Vanilla UI 挂载和交互逻辑
```

## 注意事项

此项目需要在 Onin 主应用的插件环境中运行才能正常工作。
