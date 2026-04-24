//! # Extension 类型定义模块
//!
//! 定义 Extension 系统使用的所有数据结构，包括：
//! - Extension 清单
//! - Extension 命令
//! - Extension 匹配规则
//! - Extension 执行结果

use serde::{Deserialize, Serialize};

// ============================================================================
// Extension 清单类型
// ============================================================================

/// Extension 清单
///
/// 使用静态字符串，编译时确定，无需运行时解析
#[derive(Debug, Clone)]
pub struct ExtensionManifest {
    /// 扩展唯一标识符，如 "calculator"
    pub id: &'static str,
    /// 扩展显示名称，如 "计算器"
    pub name: &'static str,
    /// 扩展描述
    pub description: &'static str,
    /// 扩展图标（Iconfont 名称）
    pub icon: &'static str,
    /// 扩展提供的命令列表
    pub commands: &'static [ExtensionCommand],
}

/// Extension 命令定义
#[derive(Debug, Clone)]
pub struct ExtensionCommand {
    /// 命令代码，如 "calculate"
    pub code: &'static str,
    /// 命令显示名称
    pub name: &'static str,
    /// 命令描述（可选）
    pub description: Option<&'static str>,
    /// 命令图标（可选，默认使用 manifest.icon）
    pub icon: Option<&'static str>,
    /// 触发关键词
    pub keywords: &'static [&'static str],
    /// 匹配规则（可选），使用统一的 StaticCommandMatch 格式
    pub matches: Option<&'static [StaticCommandMatch]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionCommandInfo {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub keywords: Vec<String>,
    pub has_matches: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub enabled: bool,
    pub commands: Vec<ExtensionCommandInfo>,
}

/// 编译时静态的匹配规则
///
/// 与 `CommandMatch`（shared_types.rs）语义一致，但使用 `&'static str`
/// 以便在 Extension 的 `static` 声明中使用。
/// 在 `generators/extension.rs` 中会转换为运行时的 `CommandMatch`。
#[derive(Debug, Clone)]
pub struct StaticCommandMatch {
    /// 匹配类型: "text" | "image" | "file" | "folder"
    pub match_type: &'static str,
    /// 匹配规则名称
    pub name: &'static str,
    /// 匹配规则描述
    pub description: &'static str,
    /// 正则表达式（仅 type="text" 时使用）
    pub regexp: Option<&'static str>,
    /// 最小数量（text: 字符数, file/image/folder: 文件数量）
    pub min: Option<u32>,
    /// 最大数量
    pub max: Option<u32>,
}

// ============================================================================
// Extension 执行结果
// ============================================================================

/// Extension 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionResult {
    /// 是否成功
    pub success: bool,
    /// 结果值（用于显示）
    pub value: Option<String>,
    /// 结果类型
    pub result_type: ExtensionResultType,
    /// 可复制到剪贴板的文本
    pub copyable: Option<String>,
    /// 副标题/元数据（如汇率更新日期）
    pub subtitle: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// Extension 结果类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionResultType {
    /// 计算结果
    Calculation,
    /// 转换结果
    Conversion,
    /// 日期时间结果
    DateTime,
    /// 货币转换结果
    Currency,
    /// 错误
    Error,
}

impl ExtensionResult {
    /// 创建成功的计算结果
    pub fn calculation(value: String) -> Self {
        Self {
            success: true,
            value: Some(value.clone()),
            result_type: ExtensionResultType::Calculation,
            copyable: Some(value),
            subtitle: None,
            error: None,
        }
    }

    /// 创建成功的转换结果
    pub fn conversion(value: String) -> Self {
        Self {
            success: true,
            value: Some(value.clone()),
            result_type: ExtensionResultType::Conversion,
            copyable: Some(value),
            subtitle: None,
            error: None,
        }
    }

    /// 创建成功的日期时间结果
    pub fn datetime(value: String) -> Self {
        Self {
            success: true,
            value: Some(value.clone()),
            result_type: ExtensionResultType::DateTime,
            copyable: Some(value),
            subtitle: None,
            error: None,
        }
    }

    /// 创建成功的货币转换结果
    pub fn currency(value: String, rate_date: Option<String>) -> Self {
        Self {
            success: true,
            value: Some(value.clone()),
            result_type: ExtensionResultType::Currency,
            copyable: Some(value),
            subtitle: rate_date,
            error: None,
        }
    }

    /// 创建错误结果
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            value: None,
            result_type: ExtensionResultType::Error,
            copyable: None,
            subtitle: None,
            error: Some(message),
        }
    }
}

// ============================================================================
// Extension 实时预览
// ============================================================================

/// 预览视图类型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PreviewViewType {
    /// 单项预览（如计算器结果）
    #[default]
    Single,
    /// Grid 网格视图（如 emoji 选择器）
    Grid,
}

/// Emoji Grid 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiGridData {
    /// 分类分组
    pub groups: Vec<EmojiGroup>,
}

/// Emoji 分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiGroup {
    /// 分类名称
    pub name: String,
    /// 分类 slug
    pub slug: String,
    /// emoji 列表
    pub emojis: Vec<EmojiItem>,
}

/// 单个 Emoji 项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiItem {
    /// emoji 字符
    pub emoji: String,
    /// 名称（用于搜索和显示）
    pub name: String,
    /// 搜索关键词（不序列化到前端）
    #[serde(default, skip_serializing)]
    pub tags: Vec<String>,
}

/// Extension 预览结果（用于搜索列表实时显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPreview {
    /// 扩展 ID
    pub extension_id: String,
    /// 命令代码
    pub command_code: String,
    /// 预览标题（如 "= 42"）
    pub title: String,
    /// 预览描述（如 "计算结果"）
    pub description: String,
    /// 图标
    pub icon: String,
    /// 可复制的值
    pub copyable: String,
    /// 视图类型
    #[serde(default)]
    pub view_type: PreviewViewType,
    /// Grid 数据（仅 Grid 视图使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_data: Option<EmojiGridData>,
}
