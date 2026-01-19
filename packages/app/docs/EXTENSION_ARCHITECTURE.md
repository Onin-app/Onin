# Extension 架构设计

> Extension（扩展）是 onin 的内置功能模块，与 Plugin（第三方插件）相对应。

## 核心概念

| 概念         | Extension（扩展） | Plugin（插件）     |
| ------------ | ----------------- | ------------------ |
| **定位**     | 系统内置功能      | 第三方功能         |
| **来源**     | 随 App 发布       | 用户安装           |
| **执行层**   | Rust 原生         | JavaScript/WebView |
| **通信方式** | 直接函数调用      | HTTP/IPC           |
| **可卸载**   | ❌                | ✅                 |

## 架构图

```
┌────────────────────────────────────────────────┐
│                  onin Core                      │
├─────────────────┬──────────────────────────────┤
│ Command Manager │ 统一命令入口                  │
├─────────────────┼──────────────────────────────┤
│ Extension       │ Plugin Manager               │
│ Manager         │                              │
├─────────────────┼──────────────────────────────┤
│ Calculator      │ Third-party plugins          │
│ Clipboard...    │ User plugins                 │
└─────────────────┴──────────────────────────────┘
      ↑ Rust 原生            ↑ JS/WebView
```

## 目录结构

```
src-tauri/src/
├── extension/                    # Extension 系统
│   ├── mod.rs                    # Extension 管理器
│   ├── types.rs                  # 类型定义
│   └── registry.rs               # 内置扩展注册
│
├── extensions/                   # 内置扩展实现
│   ├── mod.rs                    # 扩展入口
│   └── calculator/               # 计算器扩展
│       ├── mod.rs
│       └── engine.rs             # 计算引擎
```

## Extension Manifest

Extension 使用 Rust 静态结构体定义清单，**编译时确定**：

```rust
/// Extension 清单
pub struct ExtensionManifest {
    pub id: &'static str,           // "calculator"
    pub name: &'static str,         // "计算器"
    pub description: &'static str,  // "数学计算"
    pub icon: &'static str,         // "calculator"
    pub commands: &'static [ExtensionCommand],
}

/// Extension 命令
pub struct ExtensionCommand {
    pub code: &'static str,         // "calculate"
    pub name: &'static str,         // "计算"
    pub keywords: &'static [&'static str],
    pub matches: Option<ExtensionMatch>,
}

/// 匹配规则
pub struct ExtensionMatch {
    pub pattern: &'static str,      // 正则表达式
    pub min_length: Option<usize>,
}
```

## 调用流程

```
用户输入 "1+1"
    ↓
CommandManager 接收输入
    ↓
ExtensionManager.match_input("1+1")
    ↓
Calculator.evaluate("1+1") → Result(2.0)
    ↓
返回 ExtensionResult 给前端
    ↓
搜索结果显示 "= 2"
```

## 与 Plugin 的对比

| 特性          | Extension             | Plugin             |
| ------------- | --------------------- | ------------------ |
| Manifest 格式 | Rust struct（编译时） | JSON（运行时）     |
| 执行环境      | Rust 函数             | WebView + JS       |
| 调用方式      | 直接调用              | HTTP/onin-protocol |
| UI 渲染       | 主窗口内嵌            | 独立窗口/内联      |
| 权限系统      | 完全信任              | 沙箱 + 权限声明    |
| 分发方式      | App 内置              | 插件市场           |

## 内置扩展列表

| 扩展              | 状态   | 描述               |
| ----------------- | ------ | ------------------ |
| Calculator        | 开发中 | 数学计算、单位转换 |
| Clipboard History | 计划中 | 剪贴板历史管理     |
