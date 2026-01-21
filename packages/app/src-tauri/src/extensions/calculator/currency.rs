//! # 货币转换引擎
//!
//! 支持的语法：
//! - 符号格式：`$100 ¥`, `100$ ¥`, `€50 $`
//! - 代码格式：`100 usd cny`, `50 eur usd`
//!
//! 使用 frankfurter.app API（免费，无需 Key）

use crate::extension::types::ExtensionResult;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};

// ============================================================================
// 货币符号与代码映射
// ============================================================================

/// 符号到货币代码的映射
static SYMBOL_TO_CODE: LazyLock<HashMap<char, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('$', "USD");
    m.insert('¥', "CNY"); // 默认人民币
    m.insert('€', "EUR");
    m.insert('£', "GBP");
    m.insert('₩', "KRW");
    m
});

/// 货币代码到符号的映射
static CODE_TO_SYMBOL: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("USD", "$");
    m.insert("CNY", "¥");
    m.insert("JPY", "¥");
    m.insert("EUR", "€");
    m.insert("GBP", "£");
    m.insert("KRW", "₩");
    m.insert("HKD", "HK$");
    m.insert("TWD", "NT$");
    m.insert("SGD", "S$");
    m.insert("AUD", "A$");
    m.insert("CAD", "C$");
    m.insert("CHF", "CHF");
    m
});

/// 支持的货币代码列表
static SUPPORTED_CODES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "USD", "CNY", "EUR", "GBP", "JPY", "KRW", "HKD", "TWD", "SGD", "AUD", "CAD", "CHF", "RMB",
    ]
});

// ============================================================================
// 汇率缓存
// ============================================================================

/// 缓存的汇率数据
struct RateCache {
    rates: HashMap<String, f64>, // 相对于 EUR 的汇率
    last_update: Instant,
    update_date: String, // API 返回的日期
}

/// 全局汇率缓存
static RATE_CACHE: LazyLock<RwLock<Option<RateCache>>> = LazyLock::new(|| RwLock::new(None));

/// 缓存有效期（1小时）
const CACHE_DURATION: Duration = Duration::from_secs(3600);

// ============================================================================
// 正则表达式
// ============================================================================

/// 匹配符号在前：$100 ¥, $100 cny
static SYMBOL_FIRST_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([$¥€£₩])(\d+\.?\d*)\s+(?:([$¥€£₩])|([a-zA-Z]{3}))$").unwrap());

/// 匹配符号在后：100$ ¥, 100$ cny
static SYMBOL_LAST_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+\.?\d*)([$¥€£₩])\s+(?:([$¥€£₩])|([a-zA-Z]{3}))$").unwrap());

/// 匹配纯代码格式：100 usd cny
static CODE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(\d+\.?\d*)\s+([a-zA-Z]{3})\s+([a-zA-Z]{3})$").unwrap());

// ============================================================================
// API 响应结构
// ============================================================================

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    date: String,
    rates: HashMap<String, f64>,
}

// ============================================================================
// 公共接口
// ============================================================================

/// 检查输入是否是货币转换表达式
pub fn matches(input: &str) -> bool {
    let trimmed = input.trim();

    if SYMBOL_FIRST_PATTERN.is_match(trimmed) {
        return true;
    }
    if SYMBOL_LAST_PATTERN.is_match(trimmed) {
        return true;
    }
    if let Some(caps) = CODE_PATTERN.captures(trimmed) {
        let from_code = caps
            .get(2)
            .map(|m| m.as_str().to_uppercase())
            .unwrap_or_default();
        let to_code = caps
            .get(3)
            .map(|m| m.as_str().to_uppercase())
            .unwrap_or_default();
        return is_valid_currency(&from_code) && is_valid_currency(&to_code);
    }

    false
}

/// 检查缓存是否有效（用于决定是否显示加载状态）
pub fn is_cache_warm() -> bool {
    if let Ok(cache) = RATE_CACHE.read() {
        if let Some(ref c) = *cache {
            return c.last_update.elapsed() < CACHE_DURATION;
        }
    }
    false
}

/// 执行货币转换
pub fn convert(input: &str) -> ExtensionResult {
    let trimmed = input.trim();

    // 解析输入
    let parsed = parse_input(trimmed);
    if parsed.is_none() {
        return ExtensionResult::error("无法解析货币转换表达式".to_string());
    }

    let (amount, from_code, to_code) = parsed.unwrap();

    // 相同货币
    if from_code == to_code {
        let symbol = CODE_TO_SYMBOL.get(to_code.as_str()).unwrap_or(&"");
        return ExtensionResult::currency(format!("{}{:.2}", symbol, amount), None);
    }

    // 获取汇率并转换
    match get_exchange_rate(&from_code, &to_code) {
        Ok((rate, date)) => {
            let result = amount * rate;
            let symbol = CODE_TO_SYMBOL.get(to_code.as_str()).unwrap_or(&"");
            let formatted = format_currency_result(result, symbol);
            ExtensionResult::currency(formatted, Some(format!("汇率更新于 {}", date)))
        }
        Err(e) => ExtensionResult::error(format!("获取汇率失败: {}", e)),
    }
}

// ============================================================================
// 内部实现
// ============================================================================

/// 检查是否是有效的货币代码
fn is_valid_currency(code: &str) -> bool {
    let upper = code.to_uppercase();
    SUPPORTED_CODES.contains(&upper.as_str()) || upper == "RMB"
}

/// 规范化货币代码
fn normalize_code(code: &str) -> String {
    let upper = code.to_uppercase();
    if upper == "RMB" {
        "CNY".to_string()
    } else {
        upper
    }
}

/// 从符号获取货币代码
fn symbol_to_code(symbol: char) -> Option<&'static str> {
    SYMBOL_TO_CODE.get(&symbol).copied()
}

/// 解析输入，返回 (金额, 源货币代码, 目标货币代码)
fn parse_input(input: &str) -> Option<(f64, String, String)> {
    // 尝试符号在前格式：$100 ¥
    if let Some(caps) = SYMBOL_FIRST_PATTERN.captures(input) {
        let from_symbol = caps.get(1)?.as_str().chars().next()?;
        let amount: f64 = caps.get(2)?.as_str().parse().ok()?;
        let to_code = if let Some(to_sym) = caps.get(3) {
            symbol_to_code(to_sym.as_str().chars().next()?)?.to_string()
        } else {
            normalize_code(caps.get(4)?.as_str())
        };
        let from_code = symbol_to_code(from_symbol)?.to_string();
        return Some((amount, from_code, to_code));
    }

    // 尝试符号在后格式：100$ ¥
    if let Some(caps) = SYMBOL_LAST_PATTERN.captures(input) {
        let amount: f64 = caps.get(1)?.as_str().parse().ok()?;
        let from_symbol = caps.get(2)?.as_str().chars().next()?;
        let to_code = if let Some(to_sym) = caps.get(3) {
            symbol_to_code(to_sym.as_str().chars().next()?)?.to_string()
        } else {
            normalize_code(caps.get(4)?.as_str())
        };
        let from_code = symbol_to_code(from_symbol)?.to_string();
        return Some((amount, from_code, to_code));
    }

    // 尝试纯代码格式：100 usd cny
    if let Some(caps) = CODE_PATTERN.captures(input) {
        let amount: f64 = caps.get(1)?.as_str().parse().ok()?;
        let from_code = normalize_code(caps.get(2)?.as_str());
        let to_code = normalize_code(caps.get(3)?.as_str());
        if is_valid_currency(&from_code) && is_valid_currency(&to_code) {
            return Some((amount, from_code, to_code));
        }
    }

    None
}

/// 获取汇率（带缓存）
fn get_exchange_rate(from: &str, to: &str) -> Result<(f64, String), String> {
    // 检查缓存
    {
        let cache = RATE_CACHE.read().map_err(|e| e.to_string())?;
        if let Some(ref c) = *cache {
            if c.last_update.elapsed() < CACHE_DURATION {
                if let (Some(&from_rate), Some(&to_rate)) = (c.rates.get(from), c.rates.get(to)) {
                    let rate = to_rate / from_rate;
                    return Ok((rate, c.update_date.clone()));
                }
            }
        }
    }

    // 缓存过期或不存在，重新获取
    fetch_and_cache_rates()?;

    // 再次从缓存读取
    let cache = RATE_CACHE.read().map_err(|e| e.to_string())?;
    if let Some(ref c) = *cache {
        if let (Some(&from_rate), Some(&to_rate)) = (c.rates.get(from), c.rates.get(to)) {
            let rate = to_rate / from_rate;
            return Ok((rate, c.update_date.clone()));
        }
    }

    Err("无法获取汇率".to_string())
}

/// 从 API 获取汇率并缓存
fn fetch_and_cache_rates() -> Result<(), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    // 获取所有支持货币相对于 EUR 的汇率
    let currencies = "USD,CNY,JPY,GBP,KRW,HKD,TWD,SGD,AUD,CAD,CHF";
    let url = format!("https://api.frankfurter.app/latest?to={}", currencies);

    let response: FrankfurterResponse = client
        .get(&url)
        .send()
        .map_err(|e| format!("网络请求失败: {}", e))?
        .json()
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 构建汇率表（包含 EUR 自身）
    let mut rates = response.rates;
    rates.insert("EUR".to_string(), 1.0);

    // 更新缓存
    let mut cache = RATE_CACHE.write().map_err(|e| e.to_string())?;
    *cache = Some(RateCache {
        rates,
        last_update: Instant::now(),
        update_date: response.date,
    });

    Ok(())
}

/// 格式化货币结果（不包含日期）
fn format_currency_result(amount: f64, symbol: &str) -> String {
    if amount >= 1.0 {
        format!("{}{:.2}", symbol, amount)
    } else {
        format!("{}{:.4}", symbol, amount)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_symbol_first() {
        assert!(matches("$100 ¥"));
        assert!(matches("€50 $"));
        assert!(matches("£30 cny"));
        assert!(matches("¥1000 usd"));
    }

    #[test]
    fn test_matches_symbol_last() {
        assert!(matches("100$ ¥"));
        assert!(matches("50€ $"));
        assert!(matches("30£ cny"));
    }

    #[test]
    fn test_matches_code_format() {
        assert!(matches("100 usd cny"));
        assert!(matches("50 EUR USD"));
        assert!(matches("1000 jpy cny"));
        assert!(matches("100 rmb usd")); // RMB 别名
    }

    #[test]
    fn test_matches_invalid() {
        assert!(!matches("100"));
        assert!(!matches("$100"));
        assert!(!matches("hello world"));
        assert!(!matches("100 xyz abc")); // 无效货币代码
    }

    #[test]
    fn test_parse_symbol_first() {
        let result = parse_input("$100 ¥");
        assert!(result.is_some());
        let (amount, from, to) = result.unwrap();
        assert_eq!(amount, 100.0);
        assert_eq!(from, "USD");
        assert_eq!(to, "CNY");
    }

    #[test]
    fn test_parse_symbol_last() {
        let result = parse_input("100$ ¥");
        assert!(result.is_some());
        let (amount, from, to) = result.unwrap();
        assert_eq!(amount, 100.0);
        assert_eq!(from, "USD");
        assert_eq!(to, "CNY");
    }

    #[test]
    fn test_parse_code_format() {
        let result = parse_input("100 usd cny");
        assert!(result.is_some());
        let (amount, from, to) = result.unwrap();
        assert_eq!(amount, 100.0);
        assert_eq!(from, "USD");
        assert_eq!(to, "CNY");
    }

    #[test]
    fn test_normalize_rmb() {
        assert_eq!(normalize_code("rmb"), "CNY");
        assert_eq!(normalize_code("RMB"), "CNY");
        assert_eq!(normalize_code("cny"), "CNY");
    }

    #[test]
    fn test_is_valid_currency() {
        assert!(is_valid_currency("USD"));
        assert!(is_valid_currency("cny"));
        assert!(is_valid_currency("RMB"));
        assert!(!is_valid_currency("XYZ"));
    }
}
