//! # 插件桥接模块
//!
//! 负责插件与 Tauri 应用之间的通信桥接，包括：
//! - HTML 资源内联（CSS/JS）
//! - Tauri API 桥接脚本注入
//! - 窗口顶栏模板

use std::path::Path;

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

/// 修复 HTML 中的绝对路径为相对路径
///
/// Vite 构建的插件使用绝对路径（如 /assets/...），但在我们的插件服务器中：
/// - 插件 HTML 的 URL 是：http://127.0.0.1:3457/plugin/plugin-id/dist/index.html
/// - 如果 HTML 中引用 /assets/style.css，浏览器会解析为 http://127.0.0.1:3457/assets/style.css（错误）
/// - 实际文件路径应该是：http://127.0.0.1:3457/plugin/plugin-id/dist/assets/style.css
///
/// 因此需要将绝对路径 / 转换为相对路径 ./，让浏览器相对于 HTML 文件所在目录解析资源
pub fn fix_asset_paths(html: &str) -> String {
    html.replace("=\"/", "=\"./").replace("='/", "='./")
}
