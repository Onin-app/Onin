//! # 计算器扩展
//!
//! 提供数学表达式计算功能，支持：
//! - 基础运算：加减乘除
//! - 括号嵌套
//! - 幂运算
//! - 取余运算

mod engine;

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionMatch, ExtensionPreview, ExtensionResult,
};
use regex::Regex;
use std::sync::LazyLock;

// ============================================================================
// Calculator 清单定义
// ============================================================================

/// Calculator 扩展清单
pub static CALCULATOR_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "calculator",
    name: "计算器",
    description: "数学计算",
    icon: "calculator",
    commands: &[ExtensionCommand {
        code: "calculate",
        name: "计算",
        description: "计算数学表达式",
        keywords: &["calc", "计算", "="],
        matches: Some(ExtensionMatch {
            // 匹配数学表达式：数字、运算符、括号、小数点、空格
            pattern: r"^[\d\+\-\*\/\(\)\s\.\^%]+$",
            min_length: Some(1),
            max_length: None,
        }),
    }],
};

// ============================================================================
// Calculator Extension 实现
// ============================================================================

/// Calculator 扩展实例
pub struct CalculatorExtension;

/// 全局静态实例
pub static CALCULATOR_EXTENSION: CalculatorExtension = CalculatorExtension;

/// 编译后的正则表达式（延迟初始化）
static MATH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // 匹配至少包含一个运算符的表达式
    Regex::new(r"^[\d\s]*[\+\-\*\/\^%][\d\s\+\-\*\/\(\)\.\^%]*$").unwrap()
});

impl Extension for CalculatorExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &CALCULATOR_MANIFEST
    }

    fn matches(&self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        // 必须包含至少一个运算符
        MATH_PATTERN.is_match(trimmed)
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        engine::evaluate(input)
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        if !self.matches(input) {
            return None;
        }

        match engine::evaluate(input) {
            result if result.success => Some(ExtensionPreview {
                extension_id: "calculator".to_string(),
                command_code: "calculate".to_string(),
                title: format!("= {}", result.value.as_ref().unwrap_or(&String::new())),
                description: "计算结果 · 回车复制".to_string(),
                icon: "calculator".to_string(),
                copyable: result.copyable.unwrap_or_default(),
            }),
            _ => None, // 计算失败时不显示预览
        }
    }
}
