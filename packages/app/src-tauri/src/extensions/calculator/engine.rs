//! # 计算引擎
//!
//! 使用 meval crate 进行数学表达式解析和求值

use crate::extension::types::ExtensionResult;

/// 计算数学表达式
///
/// # 参数
/// - `input`: 用户输入的表达式，如 "1+1", "2*3+4", "(1+2)*3"
///
/// # 返回
/// - `ExtensionResult`: 计算结果或错误信息
pub fn evaluate(input: &str) -> ExtensionResult {
    let expr = input.trim();

    if expr.is_empty() {
        return ExtensionResult::error("表达式为空".to_string());
    }

    // 预处理表达式
    let processed = preprocess_expression(expr);

    // 使用 meval 计算
    match meval::eval_str(&processed) {
        Ok(result) => {
            // 格式化结果
            let formatted = format_result(result);
            ExtensionResult::calculation(formatted)
        }
        Err(e) => {
            // 不返回错误，让匹配失败（避免显示错误预览）
            ExtensionResult::error(format!("计算错误: {}", e))
        }
    }
}

/// 预处理表达式
///
/// 将用户输入转换为 meval 可识别的格式
fn preprocess_expression(expr: &str) -> String {
    let mut result = expr.to_string();

    // 移除空格（可选，meval 可以处理空格）
    result = result.replace(' ', "");

    // 将 ^ 转换为 meval 的幂运算符（meval 使用 ^ 作为幂运算）
    // 已经是 ^，无需转换

    // 处理百分比：5% -> 0.05
    // 简单处理：将 N% 转换为 (N/100)
    // 这里暂不实现复杂的百分比逻辑

    result
}

/// 格式化计算结果
///
/// - 整数结果不显示小数点
/// - 浮点结果保留合理精度
fn format_result(value: f64) -> String {
    // 检查是否为整数
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }

    // 检查是否接近整数（处理浮点精度问题）
    let rounded = value.round();
    if (value - rounded).abs() < 1e-10 && rounded.abs() < 1e15 {
        return format!("{}", rounded as i64);
    }

    // 保留最多 10 位有效数字
    let formatted = format!("{:.10}", value);

    // 移除尾部的零
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate("1+1").value, Some("2".to_string()));
        assert_eq!(evaluate("2*3").value, Some("6".to_string()));
        assert_eq!(evaluate("10/2").value, Some("5".to_string()));
        assert_eq!(evaluate("5-3").value, Some("2".to_string()));
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("(1+2)*3").value, Some("9".to_string()));
        assert_eq!(evaluate("2*(3+4)").value, Some("14".to_string()));
    }

    #[test]
    fn test_power() {
        assert_eq!(evaluate("2^3").value, Some("8".to_string()));
        assert_eq!(evaluate("2^10").value, Some("1024".to_string()));
    }

    #[test]
    fn test_decimal() {
        assert_eq!(evaluate("1.5+1.5").value, Some("3".to_string()));
        assert_eq!(evaluate("1.1+2.2").value, Some("3.3".to_string()));
    }

    #[test]
    fn test_format_result() {
        assert_eq!(format_result(42.0), "42");
        assert_eq!(format_result(3.14159), "3.14159");
        assert_eq!(format_result(1.0000000001), "1");
    }
}
