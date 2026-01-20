//! # 日期偏移计算引擎
//!
//! 支持的语法：
//! - 基础偏移：`+3d`, `-1w`, `+2m`, `+1y`
//! - 组合偏移：`+1m5d`, `-2w3d`
//! - 特定日期计算：`2026-12-25 -3d`
//! - 日期差值：`2026-12-25 - now`, `now - 2025-01-01`

use crate::extension::types::ExtensionResult;
use chrono::{Datelike, Duration, Local, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

// ============================================================================
// 正则表达式
// ============================================================================

/// 匹配纯偏移表达式: +3d, -1w, +1m5d, -2w3d
static OFFSET_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([+-])(\d+[dwmy])+$").unwrap());

/// 匹配单个偏移单元: 3d, 1w, 2m, 1y
static OFFSET_UNIT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+)([dwmy])").unwrap());

/// 匹配日期格式: YYYY-MM-DD
static DATE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4}-\d{2}-\d{2})$").unwrap());

/// 匹配日期+偏移: 2026-12-25 -3d, 2026-12-25 +1w
static DATE_OFFSET_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4}-\d{2}-\d{2})\s*([+-])(\d+[dwmy])+$").unwrap());

/// 匹配日期差值: 2026-12-25 - now, now - 2025-01-01, 2026-12-31 - 2026-01-01
static DATE_DIFF_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4}-\d{2}-\d{2}|now)\s*-\s*(\d{4}-\d{2}-\d{2}|now)$").unwrap()
});

// ============================================================================
// 表达式类型
// ============================================================================

#[derive(Debug)]
enum DateExprType {
    /// 纯偏移: +3d, -1w, +1m5d
    Offset,
    /// 日期+偏移: 2026-12-25 -3d
    DateOffset,
    /// 日期差值: 2026-12-25 - now
    DateDiff,
    /// 未知
    Unknown,
}

/// 识别表达式类型
fn detect_expr_type(input: &str) -> DateExprType {
    let trimmed = input.trim().to_lowercase();

    if OFFSET_PATTERN.is_match(&trimmed) {
        return DateExprType::Offset;
    }

    if DATE_OFFSET_PATTERN.is_match(&trimmed) {
        return DateExprType::DateOffset;
    }

    if DATE_DIFF_PATTERN.is_match(&trimmed) {
        return DateExprType::DateDiff;
    }

    DateExprType::Unknown
}

// ============================================================================
// 公共接口
// ============================================================================

/// 检查输入是否是日期表达式
pub fn matches(input: &str) -> bool {
    !matches!(detect_expr_type(input), DateExprType::Unknown)
}

/// 计算日期表达式
pub fn evaluate(input: &str) -> ExtensionResult {
    match detect_expr_type(input) {
        DateExprType::Offset => evaluate_offset(input),
        DateExprType::DateOffset => evaluate_date_offset(input),
        DateExprType::DateDiff => evaluate_date_diff(input),
        DateExprType::Unknown => ExtensionResult::error("无法解析日期表达式".to_string()),
    }
}

// ============================================================================
// 内部实现
// ============================================================================

/// 解析偏移单元并应用到日期
fn apply_offset(date: NaiveDate, offset_str: &str, positive: bool) -> NaiveDate {
    let mut result = date;

    for cap in OFFSET_UNIT_PATTERN.captures_iter(offset_str) {
        let value: i64 = cap[1].parse().unwrap_or(0);
        let unit = &cap[2];

        let value = if positive { value } else { -value };

        result = match unit {
            "d" => result + Duration::days(value),
            "w" => result + Duration::weeks(value),
            "m" => add_months(result, value as i32),
            "y" => add_years(result, value as i32),
            _ => result,
        };
    }

    result
}

/// 添加月份（处理月末边界情况）
fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total_months = date.year() * 12 + date.month() as i32 - 1 + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12 + 1) as u32;

    // 处理月末边界情况
    let max_day = days_in_month(new_year, new_month);
    let new_day = date.day().min(max_day);

    NaiveDate::from_ymd_opt(new_year, new_month, new_day).unwrap_or(date)
}

/// 添加年份
fn add_years(date: NaiveDate, years: i32) -> NaiveDate {
    let new_year = date.year() + years;
    let max_day = days_in_month(new_year, date.month());
    let new_day = date.day().min(max_day);

    NaiveDate::from_ymd_opt(new_year, date.month(), new_day).unwrap_or(date)
}

/// 获取某月的天数
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// 判断闰年
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 计算纯偏移: +3d, -1w, +1m5d
fn evaluate_offset(input: &str) -> ExtensionResult {
    let trimmed = input.trim().to_lowercase();
    let today = Local::now().date_naive();

    let positive = trimmed.starts_with('+');
    let offset_str = &trimmed[1..]; // 去掉符号

    let result_date = apply_offset(today, offset_str, positive);
    let formatted = format_date(result_date);

    ExtensionResult::datetime(formatted)
}

/// 计算日期+偏移: 2026-12-25 -3d
fn evaluate_date_offset(input: &str) -> ExtensionResult {
    let trimmed = input.trim().to_lowercase();

    if let Some(caps) = DATE_OFFSET_PATTERN.captures(&trimmed) {
        let date_str = &caps[1];
        let sign = &caps[2];
        let positive = sign == "+";

        // 解析基础日期
        let base_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return ExtensionResult::error("无效的日期格式".to_string()),
        };

        // 获取偏移部分
        let offset_start = caps.get(2).unwrap().start();
        let offset_str = &trimmed[offset_start..];

        let result_date = apply_offset(base_date, offset_str, positive);
        let formatted = format_date(result_date);

        return ExtensionResult::datetime(formatted);
    }

    ExtensionResult::error("无法解析日期偏移表达式".to_string())
}

/// 计算日期差值: 2026-12-25 - now
fn evaluate_date_diff(input: &str) -> ExtensionResult {
    let trimmed = input.trim().to_lowercase();

    if let Some(caps) = DATE_DIFF_PATTERN.captures(&trimmed) {
        let date1_str = &caps[1];
        let date2_str = &caps[2];

        let today = Local::now().date_naive();

        let date1 = if date1_str == "now" {
            today
        } else {
            match NaiveDate::parse_from_str(date1_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => return ExtensionResult::error("无效的日期格式".to_string()),
            }
        };

        let date2 = if date2_str == "now" {
            today
        } else {
            match NaiveDate::parse_from_str(date2_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => return ExtensionResult::error("无效的日期格式".to_string()),
            }
        };

        let diff = (date1 - date2).num_days();
        let formatted = format_days_diff(diff);

        return ExtensionResult::datetime(formatted);
    }

    ExtensionResult::error("无法解析日期差值表达式".to_string())
}

/// 格式化日期输出
fn format_date(date: NaiveDate) -> String {
    let today = Local::now().date_naive();

    if date.year() == today.year() {
        // 同年只显示月日
        format!("{}月{}日", date.month(), date.day())
    } else {
        // 不同年显示完整日期
        format!("{}年{}月{}日", date.year(), date.month(), date.day())
    }
}

/// 格式化天数差值
fn format_days_diff(days: i64) -> String {
    let abs_days = days.abs();

    if abs_days == 0 {
        "今天".to_string()
    } else if abs_days == 1 {
        if days > 0 {
            "1天后".to_string()
        } else {
            "1天前".to_string()
        }
    } else {
        format!("{}天", abs_days)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 匹配测试 ====================

    #[test]
    fn test_matches_offset() {
        assert!(matches("+3d"));
        assert!(matches("-1w"));
        assert!(matches("+2m"));
        assert!(matches("+1y"));
        assert!(matches("+1m5d"));
        assert!(matches("-2w3d"));
        assert!(matches("+1y2m3w4d"));
    }

    #[test]
    fn test_matches_date_offset() {
        assert!(matches("2026-12-25 -3d"));
        assert!(matches("2026-02-14 +1w"));
        assert!(matches("2026-01-01 +1m5d"));
    }

    #[test]
    fn test_matches_date_diff() {
        assert!(matches("2026-12-25 - now"));
        assert!(matches("now - 2025-01-01"));
        assert!(matches("2026-12-31 - 2026-01-01"));
    }

    #[test]
    fn test_matches_invalid() {
        assert!(!matches("hello"));
        assert!(!matches("1+1"));
        assert!(!matches("10km m"));
        assert!(!matches(""));
        assert!(!matches("3d")); // 缺少符号
    }

    // ==================== 偏移计算测试 ====================

    #[test]
    fn test_apply_offset_days() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert_eq!(
            apply_offset(date, "3d", true),
            NaiveDate::from_ymd_opt(2026, 1, 23).unwrap()
        );
        assert_eq!(
            apply_offset(date, "7d", false),
            NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()
        );
    }

    #[test]
    fn test_apply_offset_weeks() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert_eq!(
            apply_offset(date, "2w", true),
            NaiveDate::from_ymd_opt(2026, 2, 3).unwrap()
        );
    }

    #[test]
    fn test_apply_offset_months() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert_eq!(
            apply_offset(date, "1m", true),
            NaiveDate::from_ymd_opt(2026, 2, 20).unwrap()
        );

        // 月末边界情况
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        assert_eq!(
            apply_offset(date, "1m", true),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()
        );
    }

    #[test]
    fn test_apply_offset_years() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert_eq!(
            apply_offset(date, "1y", true),
            NaiveDate::from_ymd_opt(2027, 1, 20).unwrap()
        );

        // 闰年边界情况
        let date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        assert_eq!(
            apply_offset(date, "1y", true),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
        );
    }

    #[test]
    fn test_apply_offset_combined() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        assert_eq!(
            apply_offset(date, "1m5d", true),
            NaiveDate::from_ymd_opt(2026, 2, 25).unwrap()
        );
    }

    // ==================== 日期差值测试 ====================

    #[test]
    fn test_date_diff() {
        let result = evaluate("2026-12-31 - 2026-01-01");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "364天");
    }

    #[test]
    fn test_date_diff_same_day() {
        let result = evaluate("2026-01-20 - 2026-01-20");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "今天");
    }

    // ==================== 辅助函数测试 ====================

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29); // 闰年
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(2026));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2000));
    }
}
