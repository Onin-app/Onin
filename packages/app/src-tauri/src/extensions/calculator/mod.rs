//! - 日期偏移：+3d, -1w, +1m5d
//! - 货币转换：$100 ¥, 100 usd cny

mod currency;
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
    description: "数学、单位、日期与货币计算",
    icon: "calculator",
    commands: &[ExtensionCommand {
        code: "calculate",
        name: "计算",
        description: "数学表达式、单位转换、日期计算与货币转换",
        keywords: &[
            "calc", "计算", "=", "convert", "转换", "date", "日期", "currency", "货币",
        ],
        matches: Some(ExtensionMatch {
            // 匹配数学表达式、单位转换、日期偏移或货币转换
            pattern: r"^[\d\+\-\*\/\(\)\s\.\^%a-zA-Z$¥€£₩]+$",
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
    CurrencyConversion,
    DateTimeExpression,
    UnitConversion,
    MathExpression,
    Unknown,
}

/// 识别输入类型
fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();

    // 优先检查货币转换（$100 ¥, 100 usd cny 等）
    if currency::matches(trimmed) {
        return InputType::CurrencyConversion;
    }

    // 检查日期表达式（+3d, -1w, 2026-12-25 - now 等）
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
            InputType::CurrencyConversion => currency::convert(input),
            InputType::DateTimeExpression => datetime::evaluate(input),
            InputType::UnitConversion => units::convert(input),
            InputType::MathExpression => engine::evaluate(input),
            InputType::Unknown => ExtensionResult::error("无法识别的表达式".to_string()),
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let input_type = detect_input_type(input);

        // 货币转换时，如果缓存冷，先返回加载状态
        if matches!(input_type, InputType::CurrencyConversion) && !currency::is_cache_warm() {
            // 触发后台加载（会阻塞，但下次请求就有缓存了）
            let result = currency::convert(input);
            if !result.success {
                return None;
            }
            // 构建带 subtitle 的描述
            let description = if let Some(ref sub) = result.subtitle {
                format!("货币转换（{}）· 回车复制", sub)
            } else {
                "货币转换 · 回车复制".to_string()
            };
            return Some(ExtensionPreview {
                extension_id: "calculator".to_string(),
                command_code: "calculate".to_string(),
                title: format!("= {}", result.value.unwrap_or_default()),
                description,
                icon: "currencyDollar".to_string(),
                copyable: result.copyable.unwrap_or_default(),
            });
        }

        let result = match input_type {
            InputType::CurrencyConversion => currency::convert(input),
            InputType::DateTimeExpression => datetime::evaluate(input),
            InputType::UnitConversion => units::convert(input),
            InputType::MathExpression => engine::evaluate(input),
            InputType::Unknown => return None,
        };

        if !result.success {
            return None;
        }

        let (title, description, icon) = match input_type {
            InputType::CurrencyConversion => {
                let desc = if let Some(ref sub) = result.subtitle {
                    format!("货币转换（{}）· 回车复制", sub)
                } else {
                    "货币转换 · 回车复制".to_string()
                };
                (
                    format!("= {}", result.value.as_ref().unwrap_or(&String::new())),
                    desc,
                    "currencyDollar".to_string(),
                )
            }
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
