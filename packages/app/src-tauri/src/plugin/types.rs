//! # 插件类型定义模块
//!
//! 定义插件系统使用的所有数据结构，包括：
//! - 插件清单 (Manifest) 相关类型
//! - 插件权限配置
//! - 插件设置模式
//! - 运行时状态类型

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// 应用级状态类型
// ============================================================================

/// 插件存储
///
/// 使用 Mutex 保护的 HashMap，存储所有已加载的插件
/// Key: 插件目录名 (dir_name)
/// Value: 加载的插件信息
pub struct PluginStore(pub Mutex<HashMap<String, LoadedPlugin>>);

/// 当前活跃的插件窗口
///
/// 用于跟踪哪个插件窗口当前处于焦点状态
pub struct ActivePluginWindow(pub Mutex<Option<String>>);

/// 正在创建的插件窗口集合
///
/// 用于防止同一插件窗口被重复创建
pub struct PluginWindowCreating(pub Mutex<std::collections::HashSet<String>>);

/// 插件 HTTP 服务器端口
///
/// 存储插件服务器运行的端口号
pub struct PluginServerPort(pub Mutex<Option<u16>>);

/// 窗口切换防抖状态
///
/// 记录每个窗口最后一次切换的时间戳，防止短时间内重复切换
pub struct PluginWindowToggleDebounce(pub Mutex<HashMap<String, std::time::Instant>>);

// ============================================================================
// 插件命令相关类型
// ============================================================================

/// 插件命令清单
///
/// 定义插件提供的命令，包括名称、描述、关键词等
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandManifest {
    /// 命令代码（唯一标识符）
    pub code: String,
    /// 命令显示名称
    pub name: String,
    /// 命令描述
    pub description: String,
    /// 触发关键词列表
    #[serde(default)]
    pub keywords: Vec<PluginCommandKeyword>,
    /// 匹配规则列表
    #[serde(default)]
    pub matches: Vec<PluginCommandMatch>,
}

/// 插件命令关键词
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandKeyword {
    /// 关键词名称
    pub name: String,
    /// 关键词类型
    #[serde(rename = "type")]
    pub keyword_type: String,
}

/// 插件命令匹配配置
///
/// 三层优雅降级模型：
/// 1. 开发者层：只需配置 extensions（如 [".png", ".jpg"]）
/// 2. 系统层：自动将 extensions 映射为内部 MIME 类型
/// 3. 运行层：优先使用 MIME 类型判断，fallback 到 extensions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginCommandMatch {
    /// 匹配类型: "text" | "image" | "file" | "folder"
    #[serde(rename = "type")]
    pub match_type: String,
    /// 匹配规则名称
    pub name: String,
    /// 匹配规则描述
    pub description: String,
    /// 正则表达式（仅 type=text 时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>,
    /// 最小数量（text: 字符数, file/image/folder: 文件数量）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
    /// 最大数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    /// 文件扩展名数组（如 [".png", ".jpg"]）
    #[serde(default)]
    pub extensions: Vec<String>,
}

// ============================================================================
// 插件权限相关类型
// ============================================================================

/// HTTP 请求权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpPermission {
    /// 是否启用
    #[serde(default)]
    pub enable: bool,
    /// 允许访问的 URL 列表
    #[serde(default, rename = "allowUrls")]
    pub allow_urls: Vec<String>,
    /// 请求超时时间（毫秒）
    #[serde(default)]
    pub timeout: Option<u64>,
    /// 最大重试次数
    #[serde(default, rename = "maxRetries")]
    pub max_retries: Option<u32>,
}

/// 存储权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoragePermission {
    /// 是否启用
    #[serde(default)]
    pub enable: bool,
    /// 允许本地存储
    #[serde(default)]
    pub local: bool,
    /// 允许会话存储
    #[serde(default)]
    pub session: bool,
    /// 最大存储大小（如 "10MB"）
    #[serde(default, rename = "maxSize")]
    pub max_size: Option<String>,
}

/// 通知权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationPermission {
    /// 是否启用
    #[serde(default)]
    pub enable: bool,
    /// 允许播放声音
    #[serde(default)]
    pub sound: bool,
    /// 允许显示角标
    #[serde(default)]
    pub badge: bool,
}

/// 命令执行权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandPermission {
    /// 是否启用
    #[serde(default)]
    pub enable: bool,
    /// 允许执行的命令列表
    #[serde(default, rename = "allowCommands")]
    pub allow_commands: Vec<String>,
    /// 最大执行时间（毫秒）
    #[serde(default, rename = "maxExecutionTime")]
    pub max_execution_time: Option<u64>,
}

/// 调度器权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchedulerPermission {
    /// 是否启用
    #[serde(default)]
    pub enable: bool,
    /// 最大任务数量
    #[serde(default, rename = "maxTasks")]
    pub max_tasks: Option<usize>,
}

/// 插件权限配置
///
/// 包含所有类型的权限设置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginPermissions {
    /// HTTP 请求权限
    #[serde(default)]
    pub http: Option<HttpPermission>,
    /// 存储权限
    #[serde(default)]
    pub storage: Option<StoragePermission>,
    /// 通知权限
    #[serde(default)]
    pub notification: Option<NotificationPermission>,
    /// 命令执行权限
    #[serde(default)]
    pub command: Option<CommandPermission>,
    /// 调度器权限
    #[serde(default)]
    pub scheduler: Option<SchedulerPermission>,
}

// ============================================================================
// 插件清单相关类型
// ============================================================================

/// 插件清单
///
/// 对应 manifest.json 文件的结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    /// 插件唯一标识符
    pub id: String,
    /// 插件显示名称
    pub name: String,
    /// 插件版本号
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 入口文件路径
    pub entry: String,
    /// 图标文件路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// 插件类型（如 "ui", "script" 等）
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
    /// 插件提供的命令列表
    #[serde(default)]
    pub commands: Vec<PluginCommandManifest>,
    /// 权限配置
    pub permissions: Option<PluginPermissions>,
    /// 显示模式: "inline"（默认）或 "window"
    /// - "inline": 在主窗口列表区域显示
    /// - "window": 在新的 webview 窗口中打开
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    /// 自动分离到独立窗口
    /// 如果为 true，HTML 插件将始终在独立窗口中打开
    #[serde(default)]
    pub auto_detach: bool,
    /// 退出到后台立即结束运行
    #[serde(default)]
    pub terminate_on_bg: bool,
    /// 跟随主程序同时启动运行
    #[serde(default)]
    pub run_at_startup: bool,
    /// 开发模式标志
    /// 如果为 true，插件将从 devServer 加载而非本地文件
    #[serde(default, rename = "devMode")]
    pub dev_mode: bool,
    /// 开发服务器 URL
    /// 仅在 devMode 为 true 时使用（例如 "http://localhost:5172"）
    #[serde(skip_serializing_if = "Option::is_none", rename = "devServer")]
    pub dev_server: Option<String>,
}

/// 默认显示模式
fn default_display_mode() -> String {
    "inline".to_string()
}

// ============================================================================
// 插件设置相关类型
// ============================================================================

/// 设置选项
///
/// 用于 select/radio 等选择型控件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingOption {
    /// 显示标签
    pub label: String,
    /// 选项值
    pub value: JsonValue,
}

/// 设置字段
///
/// 定义插件设置页面的表单字段
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingField {
    /// 字段唯一键
    pub key: String,
    /// 显示标签
    pub label: String,
    /// 字段类型（如 "text", "number", "select" 等）
    #[serde(rename = "type")]
    pub field_type: String,
    /// 字段描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 占位符文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// 默认值
    #[serde(skip_serializing_if = "Option::is_none", rename = "defaultValue")]
    pub default_value: Option<JsonValue>,
    /// 是否必填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// 选项列表（用于 select/radio）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SettingOption>>,
    /// 最大长度
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxLength")]
    pub max_length: Option<u32>,
    /// 最小长度
    #[serde(skip_serializing_if = "Option::is_none", rename = "minLength")]
    pub min_length: Option<u32>,
    /// 最小值（数字类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// 最大值（数字类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// 步进值（数字类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// 是否允许多选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple: Option<bool>,
    /// 按钮文本（按钮类型）
    #[serde(skip_serializing_if = "Option::is_none", rename = "buttonText")]
    pub button_text: Option<String>,
}

/// 插件设置模式
///
/// 定义插件的设置页面结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginSettingsSchema {
    /// 设置字段列表
    pub fields: Vec<SettingField>,
}

// ============================================================================
// 插件安装来源
// ============================================================================

/// 插件安装来源
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstallSource {
    /// 本地导入
    #[serde(rename = "local")]
    Local,
    /// 从市场下载
    #[serde(rename = "marketplace")]
    Marketplace,
}

// ============================================================================
// 已加载插件
// ============================================================================

/// 已加载的插件
///
/// 包含插件的完整信息，包括清单、状态和设置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadedPlugin {
    /// 插件清单（扁平化到此结构）
    #[serde(flatten)]
    pub manifest: PluginManifest,
    /// 插件目录名
    pub dir_name: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 动态注册的设置模式（非来自 manifest）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<PluginSettingsSchema>,
    /// 安装来源
    #[serde(default = "default_install_source")]
    pub install_source: InstallSource,
}

/// 默认启用状态
fn default_enabled() -> bool {
    true
}

/// 默认安装来源
fn default_install_source() -> InstallSource {
    InstallSource::Local
}

// ============================================================================
// 插件状态持久化类型
// ============================================================================

/// 插件状态
///
/// 用于持久化保存插件的启用状态和其他配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginState {
    /// 是否启用
    pub enabled: bool,
    /// 是否自动分离到独立窗口
    #[serde(default)]
    pub auto_detach: bool,
    /// 退出到后台立即结束运行
    #[serde(default)]
    pub terminate_on_bg: bool,
    /// 跟随主程序同时启动运行
    #[serde(default)]
    pub run_at_startup: bool,
}

/// 插件状态集合
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginStates {
    /// 状态映射: plugin_id -> state
    pub states: HashMap<String, PluginState>,
}

// ============================================================================
// 窗口状态相关类型
// ============================================================================

/// 默认窗口宽度
pub const DEFAULT_WINDOW_WIDTH: u32 = 1000;
/// 默认窗口高度
pub const DEFAULT_WINDOW_HEIGHT: u32 = 700;
/// 最小窗口宽度
pub const MIN_WINDOW_WIDTH: u32 = 400;
/// 最小窗口高度
pub const MIN_WINDOW_HEIGHT: u32 = 300;
/// 最大窗口宽度（支持 4K 显示器）
pub const MAX_WINDOW_WIDTH: u32 = 4096;
/// 最大窗口高度
pub const MAX_WINDOW_HEIGHT: u32 = 2160;

/// 窗口边界
///
/// 存储窗口的位置和大小信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowBounds {
    /// X 坐标
    pub x: i32,
    /// Y 坐标
    pub y: i32,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否最大化
    #[serde(default)]
    pub is_maximized: bool,
}

impl WindowBounds {
    /// 验证并修正窗口尺寸，确保在合理范围内
    pub fn validate_and_fix(&mut self) {
        // 验证宽度
        if self.width < MIN_WINDOW_WIDTH || self.width > MAX_WINDOW_WIDTH {
            eprintln!(
                "[plugin] 窗口宽度无效: {}，使用默认值: {}",
                self.width, DEFAULT_WINDOW_WIDTH
            );
            self.width = DEFAULT_WINDOW_WIDTH;
        }

        // 验证高度
        if self.height < MIN_WINDOW_HEIGHT || self.height > MAX_WINDOW_HEIGHT {
            eprintln!(
                "[plugin] 窗口高度无效: {}，使用默认值: {}",
                self.height, DEFAULT_WINDOW_HEIGHT
            );
            self.height = DEFAULT_WINDOW_HEIGHT;
        }

        // 验证位置（确保窗口至少部分可见）
        const MAX_OFFSET: i32 = 10000;
        if self.x < -MAX_OFFSET || self.x > MAX_OFFSET {
            eprintln!("[plugin] 窗口 X 坐标无效: {}，重置为 100", self.x);
            self.x = 100;
        }

        if self.y < -MAX_OFFSET || self.y > MAX_OFFSET {
            eprintln!("[plugin] 窗口 Y 坐标无效: {}，重置为 100", self.y);
            self.y = 100;
        }
    }

    /// 创建默认窗口边界
    #[allow(dead_code)]
    pub fn new_default() -> Self {
        WindowBounds {
            x: 100,
            y: 100,
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            is_maximized: false,
        }
    }
}

/// 插件窗口状态集合
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginWindowStates {
    /// 窗口状态映射: plugin_id -> window bounds
    pub windows: HashMap<String, WindowBounds>,
}

// ============================================================================
// 插件详情响应
// ============================================================================

/// 插件详情响应
///
/// 包含插件信息和 README 内容
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginDetail {
    /// 插件信息
    #[serde(flatten)]
    pub plugin: LoadedPlugin,
    /// README 内容（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 从目录名解析插件 ID 和安装来源
///
/// 现在逻辑如下：
/// - "plugin-id@local" -> ("plugin-id", InstallSource::Local)
/// - "plugin-id" -> ("plugin-id", InstallSource::Marketplace)
/// - 其他情况（如不识别的后缀）一律视为 Marketplace，且名字即 ID
pub fn parse_plugin_dir_name(dir_name: &str) -> (String, InstallSource) {
    if let Some(at_pos) = dir_name.rfind('@') {
        let suffix = &dir_name[at_pos + 1..];

        if suffix == "local" {
            let plugin_id = dir_name[..at_pos].to_string();
            return (plugin_id, InstallSource::Local);
        }
    }

    // 其他情况（无后缀或后缀非 local）一律视为 Marketplace，且名字即 ID
    (dir_name.to_string(), InstallSource::Marketplace)
}

/// 生成带后缀的目录名
pub fn make_plugin_dir_name(plugin_id: &str, source: InstallSource) -> String {
    match source {
        InstallSource::Local => format!("{}@local", plugin_id),
        InstallSource::Marketplace => plugin_id.to_string(),
    }
}

/// 通过 plugin_id 查找插件（返回第一个匹配的）
pub fn find_plugin_by_id<'a>(
    store: &'a HashMap<String, LoadedPlugin>,
    plugin_id: &str,
) -> Option<&'a LoadedPlugin> {
    // 先尝试直接匹配 dir_name（兼容旧版本）
    if let Some(plugin) = store.get(plugin_id) {
        return Some(plugin);
    }

    // 查找 manifest.id 匹配的插件
    store.values().find(|p| p.manifest.id == plugin_id)
}

/// 通过 plugin_id 查找插件（可变引用）
pub fn find_plugin_by_id_mut<'a>(
    store: &'a mut HashMap<String, LoadedPlugin>,
    plugin_id: &str,
) -> Option<&'a mut LoadedPlugin> {
    // 先尝试直接匹配 dir_name
    if store.contains_key(plugin_id) {
        return store.get_mut(plugin_id);
    }

    // 查找 manifest.id 匹配的插件
    store.values_mut().find(|p| p.manifest.id == plugin_id)
}

/// 获取同一 plugin_id 的所有版本
#[allow(dead_code)]
pub fn find_all_versions(store: &HashMap<String, LoadedPlugin>, plugin_id: &str) -> Vec<String> {
    store
        .iter()
        .filter(|(_, p)| p.manifest.id == plugin_id)
        .map(|(dir_name, _)| dir_name.clone())
        .collect()
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
