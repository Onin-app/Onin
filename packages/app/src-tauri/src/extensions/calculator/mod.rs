//! # 计算器扩展
//!
//! 提供计算功能，支持：
//! - 基础运算：加减乘除
//! - 括号嵌套、幂运算
//! - 百分比语法：100+20% → 120
//! - 单位转换：10 km to m → 10000 m
//! - 日期偏移：+3d, -1w, +1m5d

mod datetime;
mod engine;
mod units;

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
    description: "数学计算、单位转换与日期计算",
    icon: "calculator",
    commands: &[ExtensionCommand {
        code: "calculate",
        name: "计算",
        description: "计算数学表达式、转换单位或计算日期",
        keywords: &["calc", "计算", "=", "convert", "转换", "date", "日期"],
        matches: Some(ExtensionMatch {
            // 匹配数学表达式、单位转换或日期偏移
            pattern: r"^[\d\+\-\*\/\(\)\s\.\^%a-zA-Z]+$",
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
    // 支持以数字、括号或负号开头
    Regex::new(r"^[\d\(\-][\d\s\+\-\*\/\(\)\.\^%]*[\+\-\*\/\^%][\d\s\+\-\*\/\(\)\.\^%]*$").unwrap()
});

/// 输入类型
enum InputType {
    DateTimeExpression,
    UnitConversion,
    MathExpression,
    Unknown,
}

/// 识别输入类型
fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();

    // 优先检查日期表达式（+3d, -1w, 2026-12-25 - now 等）
    if datetime::matches(trimmed) {
        return InputType::DateTimeExpression;
    }

    // 检查单位转换（包含 to/in/= 关键词）
    if units::matches(trimmed) {
        return InputType::UnitConversion;
    }

    // 检查数学表达式
    if MATH_PATTERN.is_match(trimmed) {
        return InputType::MathExpression;
    }

    InputType::Unknown
}

impl Extension for CalculatorExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &CALCULATOR_MANIFEST
    }

    fn matches(&self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        !matches!(detect_input_type(trimmed), InputType::Unknown)
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        match detect_input_type(input) {
            InputType::DateTimeExpression => datetime::evaluate(input),
            InputType::UnitConversion => units::convert(input),
            InputType::MathExpression => engine::evaluate(input),
            InputType::Unknown => ExtensionResult::error("无法识别的表达式".to_string()),
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let input_type = detect_input_type(input);

        let result = match input_type {
            InputType::DateTimeExpression => datetime::evaluate(input),
            InputType::UnitConversion => units::convert(input),
            InputType::MathExpression => engine::evaluate(input),
            InputType::Unknown => return None,
        };

        if !result.success {
            return None;
        }

        let (title, description, icon) = match input_type {
            InputType::DateTimeExpression => (
                format!("= {}", result.value.as_ref().unwrap_or(&String::new())),
                "日期计算 · 回车复制".to_string(),
                "calendar".to_string(),
            ),
            InputType::UnitConversion => (
                format!("= {}", result.value.as_ref().unwrap_or(&String::new())),
                "单位转换 · 回车复制".to_string(),
                "ruler".to_string(),
            ),
            InputType::MathExpression => (
                format!("= {}", result.value.as_ref().unwrap_or(&String::new())),
                "计算结果 · 回车复制".to_string(),
                "calculator".to_string(),
            ),
            InputType::Unknown => return None,
        };

        Some(ExtensionPreview {
            extension_id: "calculator".to_string(),
            command_code: "calculate".to_string(),
            title,
            description,
            icon,
            copyable: result.copyable.unwrap_or_default(),
        })
    }
}
