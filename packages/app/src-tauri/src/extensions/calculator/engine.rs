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
/// 支持百分比语法：
/// - `a + b%` → `a * (1 + b / 100)`
/// - `a - b%` → `a * (1 - b / 100)`
fn preprocess_expression(expr: &str) -> String {
    let mut result = expr.to_string();

    // 移除空格
    result = result.replace(' ', "");

    // 处理百分比语法：a+b% 或 a-b%
    // 匹配模式：数字 + 或 - 数字%
    result = preprocess_percentage(&result);

    result
}

/// 处理百分比语法
///
/// - `100+20%` → `100*(1+20/100)` = 120
/// - `100-20%` → `100*(1-20/100)` = 80
/// - `100+20%+20%` → 链式处理 = 144
fn preprocess_percentage(expr: &str) -> String {
    use regex::Regex;

    let mut result = expr.to_string();

    // 从右向左迭代处理百分比
    // 匹配模式：(表达式或数字)(+/-)(数字)%
    // 使用循环处理链式百分比
    loop {
        // 匹配最右边的 "X+N%" 或 "X-N%" 模式
        // X 可以是数字、括号表达式或之前处理的结果
        let re = Regex::new(r"([\d\)]+)([\+\-])(\d+\.?\d*)%").unwrap();

        if let Some(caps) = re.captures(&result) {
            let full_match = caps.get(0).unwrap();
            let base = &caps[1]; // 基数部分
            let op = &caps[2]; // 运算符
            let percent = &caps[3]; // 百分比

            // 转换为 (base) * (1 +/- percent/100)
            let replacement = format!("({}*(1{}{}/100))", base, op, percent);

            // 替换匹配的部分
            result = format!(
                "{}{}{}",
                &result[..full_match.start()],
                replacement,
                &result[full_match.end()..]
            );
        } else {
            // 没有更多百分比需要处理
            break;
        }
    }

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

    // ==================== 基础四则运算 ====================
    #[test]
    fn test_basic_addition() {
        assert_eq!(evaluate("1+1").value, Some("2".to_string()));
        assert_eq!(evaluate("0+0").value, Some("0".to_string()));
        assert_eq!(evaluate("999+1").value, Some("1000".to_string()));
    }

    #[test]
    fn test_basic_subtraction() {
        assert_eq!(evaluate("5-3").value, Some("2".to_string()));
        assert_eq!(evaluate("3-5").value, Some("-2".to_string()));
        assert_eq!(evaluate("0-0").value, Some("0".to_string()));
    }

    #[test]
    fn test_basic_multiplication() {
        assert_eq!(evaluate("2*3").value, Some("6".to_string()));
        assert_eq!(evaluate("0*100").value, Some("0".to_string()));
        assert_eq!(evaluate("7*7").value, Some("49".to_string()));
    }

    #[test]
    fn test_basic_division() {
        assert_eq!(evaluate("10/2").value, Some("5".to_string()));
        assert_eq!(evaluate("7/2").value, Some("3.5".to_string()));
        assert_eq!(evaluate("1/3").value, Some("0.3333333333".to_string()));
    }

    // ==================== 运算符优先级 ====================
    #[test]
    fn test_operator_precedence() {
        // 乘除优先于加减
        assert_eq!(evaluate("2+3*4").value, Some("14".to_string()));
        assert_eq!(evaluate("10-6/2").value, Some("7".to_string()));
        assert_eq!(evaluate("2*3+4*5").value, Some("26".to_string()));
    }

    // ==================== 括号 ====================
    #[test]
    fn test_parentheses() {
        assert_eq!(evaluate("(1+2)*3").value, Some("9".to_string()));
        assert_eq!(evaluate("2*(3+4)").value, Some("14".to_string()));
        assert_eq!(evaluate("(2+3)*(4+5)").value, Some("45".to_string()));
        assert_eq!(evaluate("((1+2)+3)*4").value, Some("24".to_string()));
    }

    #[test]
    fn test_nested_parentheses() {
        assert_eq!(evaluate("((2+3)*4)").value, Some("20".to_string()));
        assert_eq!(evaluate("(((1+1)))").value, Some("2".to_string()));
        assert_eq!(evaluate("2*((3+4)*2)").value, Some("28".to_string()));
    }

    // ==================== 幂运算 ====================
    #[test]
    fn test_power() {
        assert_eq!(evaluate("2^3").value, Some("8".to_string()));
        assert_eq!(evaluate("2^10").value, Some("1024".to_string()));
        assert_eq!(evaluate("10^0").value, Some("1".to_string()));
        assert_eq!(evaluate("2^0.5").value, Some("1.4142135624".to_string()));
    }

    // ==================== 小数 ====================
    #[test]
    fn test_decimal() {
        assert_eq!(evaluate("1.5+1.5").value, Some("3".to_string()));
        assert_eq!(evaluate("1.1+2.2").value, Some("3.3".to_string()));
        assert_eq!(evaluate("0.1+0.2").value, Some("0.3".to_string()));
        assert_eq!(evaluate("3.14159*2").value, Some("6.28318".to_string()));
    }

    // ==================== 负数 ====================
    #[test]
    fn test_negative_numbers() {
        assert_eq!(evaluate("-5+3").value, Some("-2".to_string()));
        assert_eq!(evaluate("5+-3").value, Some("2".to_string()));
        assert_eq!(evaluate("-5*-3").value, Some("15".to_string()));
        assert_eq!(evaluate("(-5)*3").value, Some("-15".to_string()));
    }

    // ==================== 空格处理 ====================
    #[test]
    fn test_spaces() {
        assert_eq!(evaluate("1 + 1").value, Some("2".to_string()));
        assert_eq!(evaluate(" 2 * 3 ").value, Some("6".to_string()));
        assert_eq!(evaluate("100 + 20%").value, Some("120".to_string()));
    }

    // ==================== 百分比语法 ====================
    #[test]
    fn test_percentage_basic() {
        // 100 + 20% = 100 * 1.2 = 120
        assert_eq!(evaluate("100+20%").value, Some("120".to_string()));
        // 100 - 20% = 100 * 0.8 = 80
        assert_eq!(evaluate("100-20%").value, Some("80".to_string()));
    }

    #[test]
    fn test_percentage_decimal() {
        // 小数百分比
        assert_eq!(evaluate("200+10.5%").value, Some("221".to_string()));
        assert_eq!(evaluate("100+0.5%").value, Some("100.5".to_string()));
    }

    #[test]
    fn test_percentage_chained() {
        // 链式百分比：100+20%+20% = 100*1.2*1.2 = 144
        assert_eq!(evaluate("100+20%+20%").value, Some("144".to_string()));
        // 100-10%-10% = 100*0.9*0.9 = 81
        assert_eq!(evaluate("100-10%-10%").value, Some("81".to_string()));
        // 混合：100+10%-5%
        assert_eq!(evaluate("100+10%-5%").value, Some("104.5".to_string()));
    }

    #[test]
    fn test_percentage_edge_cases() {
        // 50 - 50% = 25
        assert_eq!(evaluate("50-50%").value, Some("25".to_string()));
        // 100 + 100% = 200
        assert_eq!(evaluate("100+100%").value, Some("200".to_string()));
        // 100 - 100% = 0
        assert_eq!(evaluate("100-100%").value, Some("0".to_string()));
    }

    // ==================== 大数和小数 ====================
    #[test]
    fn test_large_numbers() {
        assert_eq!(
            evaluate("1000000*1000").value,
            Some("1000000000".to_string())
        );
        assert_eq!(evaluate("999999+1").value, Some("1000000".to_string()));
    }

    #[test]
    fn test_small_numbers() {
        assert_eq!(evaluate("0.001+0.001").value, Some("0.002".to_string()));
        assert_eq!(evaluate("0.1*0.1").value, Some("0.01".to_string()));
    }

    // ==================== 复杂表达式 ====================
    #[test]
    fn test_complex_expressions() {
        assert_eq!(evaluate("2+3*4-5/2").value, Some("11.5".to_string()));
        assert_eq!(
            evaluate("(1+2)*(3+4)/(5+6)").value,
            Some("1.9090909091".to_string())
        );
        assert_eq!(evaluate("2^3+4*5-6/2").value, Some("25".to_string()));
    }

    // ==================== 结果格式化 ====================
    #[test]
    fn test_format_result() {
        assert_eq!(format_result(42.0), "42");
        assert_eq!(format_result(3.14159), "3.14159");
        assert_eq!(format_result(1.0000000001), "1");
        assert_eq!(format_result(0.0), "0");
        assert_eq!(format_result(-42.5), "-42.5");
    }

    // ==================== 错误处理 ====================
    #[test]
    fn test_empty_input() {
        assert!(!evaluate("").success);
        assert!(!evaluate("   ").success);
    }

    #[test]
    fn test_invalid_expression() {
        assert!(!evaluate("abc").success);
        assert!(!evaluate("1++1").success);
        assert!(!evaluate("1+").success);
    }
}
