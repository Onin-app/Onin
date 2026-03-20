# Onin Plugin SDK 完整演示

全面演示 Onin SDK 的所有 API 功能的示例项目。

## 功能

本项目涵盖 SDK 的 **10 个 API 模块**，共计 **60+ 个方法**：

### 核心 API
- **Lifecycle API** - 插件生命周期管理
  - `onLoad()`, `onUnload()`, `onWindowShow()`, `onWindowHide()`
- **Command API** - 命令注册、处理、匹配
  - `register()`, `handle()`, `remove()`
  - 支持 matches 匹配规则：text, file, folder, image, regexp
- **Settings API** - 插件设置定义
  - `useSettingsSchema()`, `getSettings()`
  - 支持所有字段类型：text, password, textarea, number, switch, slider, select, radio, color, date, time, datetime, button

### 数据 API
- **Storage API** - 持久化存储
  - `setItem()`, `getItem()`, `removeItem()`, `clear()`, `keys()`
  - `setItems()`, `getItems()`, `getAll()`, `setAll()`
- **File System API** - 文件系统操作
  - `readFile()`, `writeFile()`, `exists()`, `createDir()`, `listDir()`
  - `deleteFile()`, `deleteDir()`, `getFileInfo()`, `copyFile()`, `moveFile()`
- **Clipboard API** - 剪贴板操作
  - `readText()`, `writeText()`, `readImage()`, `writeImage()`
  - `clear()`, `hasText()`, `hasImage()`, `copy()`, `paste()`

### 交互 API
- **Notification API** - 系统通知
  - `show()`
- **Dialog API** - 对话框
  - `showMessage()`, `showConfirm()`, `showOpen()`, `showSave()`
  - `info()`, `warning()`, `error()`, `confirm()`
  - `selectFile()`, `selectFiles()`, `selectFolder()`, `save()`
- **HTTP API** - 网络请求
  - `get()`, `post()`, `put()`, `patch()`, `delete()`, `request()`
- **Scheduler API** - 定时任务
  - `schedule()`, `daily()`, `hourly()`, `weekly()`, `cancel()`, `list()`

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

## 目录结构

```
packages/plugin-demo/
├── index.html           # 主页面
├── package.json         # 包配置
├── README.md            # 本文档
├── tsconfig.json        # TypeScript 配置
├── vite.config.ts       # Vite 配置
└── src/
    ├── main.ts          # 入口文件
    ├── style.css        # 样式
    └── demos/           # 各 API 演示模块
        ├── index.ts           # 导出所有 demo
        ├── lifecycle.ts       # Lifecycle API 演示
        ├── command.ts         # Command API 演示
        ├── storage.ts         # Storage API 演示
        ├── notification.ts    # Notification API 演示
        ├── clipboard.ts       # Clipboard API 演示
        ├── http.ts            # HTTP API 演示
        ├── fs.ts              # File System API 演示
        ├── dialog.ts          # Dialog API 演示
        ├── settings.ts        # Settings API 演示
        └── scheduler.ts       # Scheduler API 演示
```

## 注意事项

此项目需要在 Onin 主应用的插件环境中运行才能正常工作。
