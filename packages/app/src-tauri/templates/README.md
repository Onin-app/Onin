# 插件窗口模板

这个目录包含插件窗口的 HTML/CSS/JS 模板文件，用于外部化 Rust 代码中的前端资源。

## 文件说明

### plugin-window-topbar.html
包含插件窗口的自定义顶栏 UI，包括：
- 样式定义（支持亮色/暗色主题）
- 窗口标题栏
- 控制按钮（切换到主窗口、最小化、最大化、关闭）
- 调试信息显示区域

### plugin-window-controls.js
包含插件窗口控制逻辑，包括：
- Tauri API 桥接初始化
- 窗口控制按钮事件处理
- 窗口状态管理

## 使用方式

这些模板文件在编译时通过 `include_str!` 宏嵌入到 Rust 二进制文件中：

```rust
const PLUGIN_WINDOW_TOPBAR_TEMPLATE: &str = include_str!("../templates/plugin-window-topbar.html");
const PLUGIN_WINDOW_CONTROLS_SCRIPT: &str = include_str!("../templates/plugin-window-controls.js");
```

然后在运行时动态注入到插件窗口的 HTML 中。

## 优势

1. **代码分离**：前端代码与 Rust 代码分离，更易维护
2. **语法高亮**：编辑器可以正确识别 HTML/CSS/JS 语法
3. **零运行时开销**：编译时嵌入，无需运行时读取文件
4. **易于修改**：修改模板无需在 Rust 字符串中处理转义

## 修改模板

修改模板文件后，需要重新编译 Rust 项目才能生效：

```bash
cd src-tauri
cargo build
```
