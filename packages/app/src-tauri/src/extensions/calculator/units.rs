//! # 单位转换引擎
//!
//! 支持的语法：
//! - `10km m` → 10000 m
//! - `1h m` → 60 min (上下文感知：h是时间，所以m是分钟)
//! - `5 kg lb` → 11.02 lb

use crate::extension::types::ExtensionResult;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// 单位定义
// ============================================================================

/// 单位类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitCategory {
    Length,
    Weight,
    Time,
    Data,
    Volume,
}

/// 单位信息
struct UnitInfo {
    category: UnitCategory,
    to_base: f64,
    display: &'static str,
}

/// 歧义单位 - 需要根据上下文判断
/// key: 单位字符串, value: [(类别, 基准倍数, 显示名)]
/// 歧义单位仅用于需要上下文判断的情况
static AMBIGUOUS_UNITS: LazyLock<HashMap<&'static str, Vec<(UnitCategory, f64, &'static str)>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // "m" 可以是米或分钟（不用于数据单位MB，太歧义）
        m.insert(
            "m",
            vec![
                (UnitCategory::Length, 1.0, "m"),  // 米
                (UnitCategory::Time, 60.0, "min"), // 分钟
            ],
        );

        m
    });

/// 单位映射表
static UNITS: LazyLock<HashMap<&'static str, UnitInfo>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // ==================== 长度单位 ====================
    // 基准单位：米 (m)
    m.insert(
        "km",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 1000.0,
            display: "km",
        },
    );
    m.insert(
        "m",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 1.0,
            display: "m",
        },
    );
    m.insert(
        "cm",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.01,
            display: "cm",
        },
    );
    m.insert(
        "mm",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.001,
            display: "mm",
        },
    );
    m.insert(
        "mile",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 1609.344,
            display: "mile",
        },
    );
    m.insert(
        "miles",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 1609.344,
            display: "miles",
        },
    );
    m.insert(
        "yard",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.9144,
            display: "yard",
        },
    );
    m.insert(
        "yards",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.9144,
            display: "yards",
        },
    );
    m.insert(
        "yd",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.9144,
            display: "yd",
        },
    );
    m.insert(
        "ft",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.3048,
            display: "ft",
        },
    );
    m.insert(
        "foot",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.3048,
            display: "foot",
        },
    );
    m.insert(
        "feet",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.3048,
            display: "feet",
        },
    );
    m.insert(
        "inch",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254,
            display: "inch",
        },
    );
    m.insert(
        "inches",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254,
            display: "inches",
        },
    );
    m.insert(
        "in",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254,
            display: "in",
        },
    );
    // 印刷/设计单位
    m.insert(
        "pt",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254 / 72.0, // 1 pt = 1/72 inch
            display: "pt",
        },
    );
    m.insert(
        "point",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254 / 72.0,
            display: "pt",
        },
    );
    m.insert(
        "px",
        UnitInfo {
            category: UnitCategory::Length,
            to_base: 0.0254 / 96.0, // 1 px = 1/96 inch (CSS standard 96 PPI)
            display: "px",
        },
    );

    // ==================== 重量单位 ====================
    // 基准单位：千克 (kg)
    m.insert(
        "kg",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 1.0,
            display: "kg",
        },
    );
    m.insert(
        "g",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 0.001,
            display: "g",
        },
    );
    m.insert(
        "mg",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 0.000001,
            display: "mg",
        },
    );
    m.insert(
        "lb",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 0.453592,
            display: "lb",
        },
    );
    m.insert(
        "lbs",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 0.453592,
            display: "lbs",
        },
    );
    m.insert(
        "oz",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 0.0283495,
            display: "oz",
        },
    );
    m.insert(
        "t",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 1000.0,
            display: "t",
        },
    );
    m.insert(
        "ton",
        UnitInfo {
            category: UnitCategory::Weight,
            to_base: 1000.0,
            display: "ton",
        },
    );

    // ==================== 时间单位 ====================
    // 基准单位：秒 (s)
    m.insert(
        "year",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 31557600.0,
            display: "year",
        },
    );
    m.insert(
        "years",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 31557600.0,
            display: "years",
        },
    );
    m.insert(
        "yr",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 31557600.0,
            display: "yr",
        },
    );
    m.insert(
        "y",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 31557600.0,
            display: "y",
        },
    );
    m.insert(
        "month",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 2629800.0,
            display: "month",
        },
    );
    m.insert(
        "months",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 2629800.0,
            display: "months",
        },
    );
    m.insert(
        "mo",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 2629800.0,
            display: "mo",
        },
    );
    m.insert(
        "week",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 604800.0,
            display: "week",
        },
    );
    m.insert(
        "weeks",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 604800.0,
            display: "weeks",
        },
    );
    m.insert(
        "wk",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 604800.0,
            display: "wk",
        },
    );
    m.insert(
        "w",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 604800.0,
            display: "w",
        },
    );
    m.insert(
        "day",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 86400.0,
            display: "day",
        },
    );
    m.insert(
        "days",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 86400.0,
            display: "days",
        },
    );
    m.insert(
        "d",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 86400.0,
            display: "d",
        },
    );
    m.insert(
        "hour",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 3600.0,
            display: "hour",
        },
    );
    m.insert(
        "hours",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 3600.0,
            display: "hours",
        },
    );
    m.insert(
        "hr",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 3600.0,
            display: "hr",
        },
    );
    m.insert(
        "h",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 3600.0,
            display: "h",
        },
    );
    m.insert(
        "min",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 60.0,
            display: "min",
        },
    );
    m.insert(
        "mins",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 60.0,
            display: "mins",
        },
    );
    m.insert(
        "minute",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 60.0,
            display: "minute",
        },
    );
    m.insert(
        "minutes",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 60.0,
            display: "minutes",
        },
    );
    m.insert(
        "sec",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 1.0,
            display: "sec",
        },
    );
    m.insert(
        "secs",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 1.0,
            display: "secs",
        },
    );
    m.insert(
        "second",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 1.0,
            display: "second",
        },
    );
    m.insert(
        "seconds",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 1.0,
            display: "seconds",
        },
    );
    m.insert(
        "s",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 1.0,
            display: "s",
        },
    );
    m.insert(
        "ms",
        UnitInfo {
            category: UnitCategory::Time,
            to_base: 0.001,
            display: "ms",
        },
    );

    // ==================== 数据单位 ====================
    // 基准单位：字节 (B)
    m.insert(
        "tb",
        UnitInfo {
            category: UnitCategory::Data,
            to_base: 1_099_511_627_776.0,
            display: "TB",
        },
    );
    m.insert(
        "gb",
        UnitInfo {
            category: UnitCategory::Data,
            to_base: 1_073_741_824.0,
            display: "GB",
        },
    );
    m.insert(
        "mb",
        UnitInfo {
            category: UnitCategory::Data,
            to_base: 1_048_576.0,
            display: "MB",
        },
    );
    m.insert(
        "kb",
        UnitInfo {
            category: UnitCategory::Data,
            to_base: 1024.0,
            display: "KB",
        },
    );
    m.insert(
        "b",
        UnitInfo {
            category: UnitCategory::Data,
            to_base: 1.0,
            display: "B",
        },
    );

    // ==================== 容量单位 ====================
    // 基准单位：毫升 (ml)
    m.insert(
        "l",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 1000.0,
            display: "L",
        },
    );
    m.insert(
        "liter",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 1000.0,
            display: "L",
        },
    );
    m.insert(
        "liters",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 1000.0,
            display: "L",
        },
    );
    m.insert(
        "ml",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 1.0,
            display: "ml",
        },
    );
    m.insert(
        "tsp",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 4.929,
            display: "tsp",
        },
    );
    m.insert(
        "teaspoon",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 4.929,
            display: "tsp",
        },
    );
    m.insert(
        "tbsp",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 14.787,
            display: "tbsp",
        },
    );
    m.insert(
        "tablespoon",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 14.787,
            display: "tbsp",
        },
    );
    m.insert(
        "cup",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 236.588,
            display: "cup",
        },
    );
    m.insert(
        "cups",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 236.588,
            display: "cups",
        },
    );
    m.insert(
        "floz",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 29.574,
            display: "fl oz",
        },
    );
    m.insert(
        "pint",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 473.176,
            display: "pint",
        },
    );
    m.insert(
        "gallon",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 3785.41,
            display: "gallon",
        },
    );
    m.insert(
        "gal",
        UnitInfo {
            category: UnitCategory::Volume,
            to_base: 3785.41,
            display: "gal",
        },
    );

    m
});

/// 匹配单位转换的正则表达式
static UNIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(\d+\.?\d*)\s*([a-z]+)\s+(?:to\s+|in\s+)?([a-z]+)$").unwrap()
});

// ============================================================================
// 公共接口
// ============================================================================

/// 根据源单位类别解析目标单位
fn resolve_unit<'a>(
    unit_str: &'a str,
    context_category: Option<UnitCategory>,
) -> Option<(UnitCategory, f64, &'static str)> {
    // 首先检查是否是歧义单位
    if let Some(options) = AMBIGUOUS_UNITS.get(unit_str) {
        // 如果有上下文类别，优先匹配该类别
        if let Some(cat) = context_category {
            for (c, to_base, display) in options {
                if *c == cat {
                    return Some((*c, *to_base, display));
                }
            }
        }
        // 没有上下文或没匹配到，返回第一个（默认）
        let (c, to_base, display) = &options[0];
        return Some((*c, *to_base, display));
    }

    // 非歧义单位，直接查找
    UNITS
        .get(unit_str)
        .map(|info| (info.category, info.to_base, info.display))
}

/// 检查输入是否是单位转换表达式
pub fn matches(input: &str) -> bool {
    let trimmed = input.trim().to_lowercase();
    if let Some(caps) = UNIT_PATTERN.captures(&trimmed) {
        let from_unit = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let to_unit = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        // 解析源单位（无上下文）
        if let Some((from_category, _, _)) = resolve_unit(from_unit, None) {
            // 解析目标单位（带上下文）
            if let Some((to_category, _, _)) = resolve_unit(to_unit, Some(from_category)) {
                return from_category == to_category;
            }
        }
    }
    false
}

/// 执行单位转换
pub fn convert(input: &str) -> ExtensionResult {
    let trimmed = input.trim().to_lowercase();

    if let Some(caps) = UNIT_PATTERN.captures(&trimmed) {
        let value: f64 = caps
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);
        let from_unit = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let to_unit = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        // 解析源单位
        if let Some((from_category, from_base, _)) = resolve_unit(from_unit, None) {
            // 解析目标单位（使用源单位类别作为上下文）
            if let Some((to_category, to_base, to_display)) =
                resolve_unit(to_unit, Some(from_category))
            {
                if from_category != to_category {
                    return ExtensionResult::error("单位类别不匹配".to_string());
                }

                let result = value * from_base / to_base;
                let formatted = format_result(result, to_display);
                return ExtensionResult::conversion(formatted);
            }
        }
    }

    ExtensionResult::error("无法解析单位转换".to_string())
}

/// 格式化结果
fn format_result(value: f64, unit: &str) -> String {
    if value.fract() == 0.0 && value.abs() < 1e12 {
        return format!("{} {}", value as i64, unit);
    }

    let rounded = value.round();
    if (value - rounded).abs() < 1e-9 && rounded.abs() < 1e12 {
        return format!("{} {}", rounded as i64, unit);
    }

    let formatted = if value.abs() >= 1.0 {
        format!("{:.4}", value)
    } else {
        format!("{:.6}", value)
    };

    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{} {}", trimmed, unit)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 匹配测试 ====================

    #[test]
    fn test_matches_simple_syntax() {
        // 简化语法（空格分隔）
        assert!(matches("10km m"));
        assert!(matches("5 kg lb"));
        assert!(matches("1 GB MB"));
        assert!(matches("100cm inch"));
    }

    #[test]
    fn test_matches_traditional_syntax() {
        // 传统语法（带 to/in）
        assert!(matches("10 km to m"));
        assert!(matches("5 kg in lb"));
        assert!(matches("1 hour to min"));
    }

    #[test]
    fn test_matches_contextual() {
        // 上下文感知
        assert!(matches("1h m")); // h是时间，m是分钟
        assert!(matches("1 hour m")); // hour是时间，m是分钟
        assert!(matches("1km m")); // km是长度，m是米
    }

    #[test]
    fn test_matches_invalid() {
        assert!(!matches("10 km kg")); // 不同类别
        assert!(!matches("10 hour kg")); // 不同类别
        assert!(!matches("hello"));
        assert!(!matches("1+1"));
        assert!(!matches(""));
    }

    // ==================== 上下文消歧 ====================

    #[test]
    fn test_contextual_m_as_minute() {
        let result = convert("1h m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "60 min");

        let result = convert("2 hour m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "120 min");
    }

    #[test]
    fn test_contextual_m_as_meter() {
        let result = convert("1km m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 m");

        let result = convert("100cm m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1 m");
    }

    // ==================== 长度转换 ====================

    #[test]
    fn test_length_km_m() {
        let result = convert("10 km m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "10000 m");
    }

    #[test]
    fn test_length_m_cm() {
        let result = convert("1 m cm");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "100 cm");
    }

    #[test]
    fn test_length_cm_mm() {
        let result = convert("1 cm mm");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "10 mm");
    }

    #[test]
    fn test_length_mile_km() {
        let result = convert("1 mile km");
        assert!(result.success);
        // 1 mile ≈ 1.609 km
        assert!(result.value.as_ref().unwrap().contains("1.609"));
    }

    #[test]
    fn test_length_ft_inch() {
        let result = convert("1 ft inch");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "12 inch");
    }

    #[test]
    fn test_length_yard_ft() {
        let result = convert("1 yard ft");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "3 ft");
    }

    #[test]
    fn test_length_inch_cm() {
        let result = convert("1 inch cm");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "2.54 cm");
    }

    #[test]
    fn test_length_inch_pt() {
        let result = convert("1 inch pt");
        assert!(result.success);
        // 1 inch = 72 pt
        assert_eq!(result.value.as_ref().unwrap(), "72 pt");
    }

    #[test]
    fn test_length_cm_pt() {
        let result = convert("1 cm pt");
        assert!(result.success);
        // 1 cm ≈ 28.35 pt
        assert!(result.value.as_ref().unwrap().contains("28"));
    }

    #[test]
    fn test_length_inch_px() {
        let result = convert("1 inch px");
        assert!(result.success);
        // 1 inch = 96 px (CSS standard)
        assert_eq!(result.value.as_ref().unwrap(), "96 px");
    }

    // ==================== 重量转换 ====================

    #[test]
    fn test_weight_kg_g() {
        let result = convert("1 kg g");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 g");
    }

    #[test]
    fn test_weight_g_mg() {
        let result = convert("1 g mg");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 mg");
    }

    #[test]
    fn test_weight_kg_lb() {
        let result = convert("1 kg lb");
        assert!(result.success);
        // 1 kg ≈ 2.205 lb
        assert!(result.value.as_ref().unwrap().contains("2.20"));
    }

    #[test]
    fn test_weight_lb_oz() {
        let result = convert("1 lb oz");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "16 oz");
    }

    #[test]
    fn test_weight_ton_kg() {
        let result = convert("1 t kg");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 kg");

        let result = convert("1 ton kg");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 kg");
    }

    // ==================== 时间转换 ====================

    #[test]
    fn test_time_hour_min() {
        let result = convert("1 hour min");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "60 min");

        let result = convert("1 h min");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "60 min");
    }

    #[test]
    fn test_time_min_sec() {
        let result = convert("1 min sec");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "60 sec");
    }

    #[test]
    fn test_time_day_hour() {
        let result = convert("1 day hour");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "24 hour");

        let result = convert("1 d h");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "24 h");
    }

    #[test]
    fn test_time_week_day() {
        let result = convert("1 week day");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "7 day");

        let result = convert("2 w d");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "14 d");
    }

    #[test]
    fn test_time_month_day() {
        let result = convert("1 month day");
        assert!(result.success);
        // 1 month ≈ 30.44 days
        assert!(result.value.as_ref().unwrap().contains("30"));
    }

    #[test]
    fn test_time_year_day() {
        let result = convert("1 year day");
        assert!(result.success);
        // 1 year ≈ 365.25 days
        assert!(result.value.as_ref().unwrap().contains("365"));
    }

    #[test]
    fn test_time_sec_ms() {
        let result = convert("1 sec ms");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 ms");

        let result = convert("1 s ms");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 ms");
    }

    // ==================== 数据转换 ====================

    #[test]
    fn test_data_gb_mb() {
        let result = convert("1 GB MB");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1024 MB");

        let result = convert("1 gb mb");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1024 MB");
    }

    #[test]
    fn test_data_mb_kb() {
        let result = convert("1 MB KB");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1024 KB");
    }

    #[test]
    fn test_data_kb_b() {
        let result = convert("1 KB B");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1024 B");
    }

    #[test]
    fn test_data_tb_gb() {
        let result = convert("1 TB GB");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1024 GB");
    }

    // ==================== 容量转换 ====================

    #[test]
    fn test_volume_l_ml() {
        let result = convert("1 l ml");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 ml");

        let result = convert("1 liter ml");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000 ml");
    }

    #[test]
    fn test_volume_tsp_ml() {
        let result = convert("1 tsp ml");
        assert!(result.success);
        // 1 tsp ≈ 4.929 ml
        assert!(result.value.as_ref().unwrap().contains("4.929"));

        let result = convert("3 teaspoon ml");
        assert!(result.success);
    }

    #[test]
    fn test_volume_tbsp_ml() {
        let result = convert("1 tbsp ml");
        assert!(result.success);
        // 1 tbsp ≈ 14.787 ml
        assert!(result.value.as_ref().unwrap().contains("14.787"));
    }

    #[test]
    fn test_volume_cup_ml() {
        let result = convert("1 cup ml");
        assert!(result.success);
        // 1 cup ≈ 236.588 ml
        assert!(result.value.as_ref().unwrap().contains("236"));
    }

    #[test]
    fn test_volume_gallon_l() {
        let result = convert("1 gallon l");
        assert!(result.success);
        // 1 gallon ≈ 3.785 L
        assert!(result.value.as_ref().unwrap().contains("3.785"));

        let result = convert("1 gal l");
        assert!(result.success);
    }

    // ==================== 边界情况 ====================

    #[test]
    fn test_decimal_values() {
        let result = convert("1.5 km m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1500 m");

        let result = convert("0.5 hour min");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "30 min");
    }

    #[test]
    fn test_large_values() {
        let result = convert("1000 km m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "1000000 m");
    }

    #[test]
    fn test_small_values() {
        let result = convert("1 mm m");
        assert!(result.success);
        assert_eq!(result.value.as_ref().unwrap(), "0.001 m");
    }

    #[test]
    fn test_case_insensitive() {
        let result = convert("1 KM M");
        assert!(result.success);

        let result = convert("1 Kg G");
        assert!(result.success);
    }

    #[test]
    fn test_spaces_handling() {
        let result = convert("  1 km   m  ");
        assert!(result.success);

        let result = convert("1  hour  min");
        assert!(result.success);
    }

    // ==================== 错误处理 ====================

    #[test]
    fn test_different_categories() {
        let result = convert("1 km kg");
        assert!(!result.success);
    }

    #[test]
    fn test_unknown_unit() {
        let result = convert("1 xyz abc");
        assert!(!result.success);
    }
}
