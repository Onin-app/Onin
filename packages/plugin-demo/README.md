# Onin Plugin SDK Demo

用于验证 Onin 插件 SDK 核心能力和新插件入口模型的示例项目。

## 功能

本项目覆盖一组代表性的 SDK 能力：

- Lifecycle / background entry
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

- `src/plugin.ts` 声明后台入口和 UI 挂载
- `src/background.ts` 注册后台入口
- `src/main.ts` 挂载 UI
- `scripts/build.mjs` 一次产出 `dist/index.html` 和 `dist/lifecycle.js`

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
    ├── background.ts    # 后台入口薄包装
    ├── main.ts          # UI 入口薄包装
    ├── plugin.ts        # 插件声明
    ├── style.css        # 样式
    └── ui.ts            # Vanilla UI 挂载和交互逻辑
```

## 注意事项

此项目需要在 Onin 主应用的插件环境中运行才能正常工作。
