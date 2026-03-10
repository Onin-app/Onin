//! # 插件桥接模块
//!
//! 负责插件与 Tauri 应用之间的通信桥接，包括：
//! - HTML 资源内联（CSS/JS）
//! - Tauri API 桥接脚本注入
//! - 窗口顶栏模板


// ============================================================================
// 模板常量
// ============================================================================

/// 插件窗口顶栏 HTML 模板
///
/// 在编译时从文件加载，提供自定义窗口控制栏
pub const PLUGIN_WINDOW_TOPBAR_TEMPLATE: &str =
    include_str!("../../templates/plugin-window-topbar.html");

/// 插件窗口控制脚本
///
/// 在编译时从文件加载，提供窗口最小化、最大化、关闭功能
pub const PLUGIN_WINDOW_CONTROLS_SCRIPT: &str =
    include_str!("../../templates/plugin-window-controls.js");

