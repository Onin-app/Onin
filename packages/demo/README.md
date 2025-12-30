# Baize SDK Demo

这是一个用于测试 Baize SDK 的演示项目。

## 功能

- 实时测试 SDK 的所有 API
- 本地开发时直接引用 workspace 中的 SDK
- 可视化的测试界面

## 开发

```bash
# 在根目录运行
pnpm dev:demo

# 或者在当前目录
pnpm dev
```

## 测试的 API

- **Lifecycle API**: 插件生命周期管理
- **Window API**: 窗口控制
- **Storage API**: 数据存储
- **UI API**: 界面组件
- **Request API**: HTTP 请求

## 注意事项

此项目需要在 Baize 主应用的插件环境中运行才能正常工作。
