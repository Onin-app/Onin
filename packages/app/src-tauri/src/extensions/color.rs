//! Color Extension
//!
//! Parses common CSS color formats and previews normalized conversions.

use crate::extension::registry::Extension;
use crate::extension::types::{
    ExtensionCommand, ExtensionManifest, ExtensionPreview, ExtensionResult, PreviewViewType,
};
use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

pub static COLOR_MANIFEST: ExtensionManifest = ExtensionManifest {
    id: "color",
    name: "颜色转换",
    description: "解析 Hex、RGB、HSL 颜色并转换格式",
    icon: "palette",
    commands: &[
        ExtensionCommand {
            code: "convert",
            name: "颜色转换",
            description: Some("解析 Hex、RGB、HSL 颜色并转换格式"),
            icon: Some("palette"),
            keywords: &[
                "color", "colour", "颜色", "色值", "hex", "rgb", "rgba", "hsl", "hsla",
            ],
            matches: None,
        },
        ExtensionCommand {
            code: "pick",
            name: "取色",
            description: Some("从屏幕任意位置拾取颜色"),
            icon: Some("eyedropper"),
            keywords: &[
                "取色",
                "吸管",
                "拾色",
                "屏幕取色",
                "picker",
                "pick color",
                "eyedropper",
            ],
            matches: None,
        },
    ],
};

pub struct ColorExtension;

pub static COLOR_EXTENSION: ColorExtension = ColorExtension;

#[derive(Debug, Clone, Serialize)]
pub struct ColorConversion {
    pub hex: String,
    pub rgb: String,
    pub hsl: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorParseMode {
    Launcher,
    Full,
}

impl From<Color> for ColorConversion {
    fn from(color: Color) -> Self {
        Self {
            hex: color.hex(),
            rgb: color.rgb(),
            hsl: color.hsl(),
            red: color.r,
            green: color.g,
            blue: color.b,
            alpha: color.alpha,
        }
    }
}

impl Color {
    fn hex(self) -> String {
        if self.alpha < 0.999 {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                self.r,
                self.g,
                self.b,
                (self.alpha.clamp(0.0, 1.0) * 255.0).round() as u8
            )
        } else {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        }
    }

    fn rgb(self) -> String {
        if self.alpha < 0.999 {
            format!(
                "rgba({}, {}, {}, {:.2})",
                self.r, self.g, self.b, self.alpha
            )
        } else {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        }
    }

    fn hsl(self) -> String {
        let (h, s, l) = rgb_to_hsl(self.r, self.g, self.b);
        if self.alpha < 0.999 {
            format!("hsla({}, {}%, {}%, {:.2})", h, s, l, self.alpha)
        } else {
            format!("hsl({}, {}%, {}%)", h, s, l)
        }
    }
}

static HEX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^#?([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$").unwrap()
});

static RGB_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^rgba?\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?)\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?)\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?)
        (?:\s*(?:,|/)\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

static HSL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^hsla?\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))(deg|rad|turn|grad)?\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%
        (?:\s*(?:,|/)\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

static HSV_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^hsva?\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))(deg|rad|turn|grad)?\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%
        (?:\s*(?:,|/)\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

static HWB_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^hwb\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))(deg|rad|turn|grad)?\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%\s*(?:,|\s)\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))%
        (?:\s*(?:,|/)\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

static OKLCH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^oklch\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?)\s+
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))\s+
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))(deg|rad|turn|grad)?
        (?:\s*/\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

static OKLAB_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^oklab\(\s*
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?)\s+
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))\s+
        ([+-]?(?:\d+(?:\.\d+)?|\.\d+))
        (?:\s*/\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)%?))?
        \s*\)$",
    )
    .unwrap()
});

fn strip_color_prefix(input: &str) -> &str {
    let trimmed = input.trim();
    for prefix in ["color ", "colour ", "颜色 ", "色值 "] {
        if trimmed.to_lowercase().starts_with(prefix) {
            return trimmed[prefix.len()..].trim();
        }
    }
    trimmed
}

fn parse_color(input: &str, mode: ColorParseMode) -> Option<Color> {
    let value = strip_color_prefix(input);
    let basic = parse_hex(value)
        .or_else(|| parse_rgb(value))
        .or_else(|| parse_hsl(value));

    if basic.is_some() || mode == ColorParseMode::Launcher {
        return basic;
    }

    parse_hsv(value)
        .or_else(|| parse_hwb(value))
        .or_else(|| parse_oklch(value))
        .or_else(|| parse_oklab(value))
        .or_else(|| parse_color_function(value))
}

pub fn convert_color_value(input: &str) -> Option<ColorConversion> {
    convert_color_value_with_mode(input, ColorParseMode::Launcher)
}

pub fn convert_color_value_with_mode(input: &str, mode: ColorParseMode) -> Option<ColorConversion> {
    parse_color(input, mode).map(ColorConversion::from)
}

fn parse_hex(input: &str) -> Option<Color> {
    let caps = HEX_PATTERN.captures(input.trim())?;
    let raw = caps.get(1)?.as_str();
    let expanded = match raw.len() {
        3 | 4 => raw
            .chars()
            .flat_map(|c| [c, c])
            .collect::<String>()
            .to_uppercase(),
        6 | 8 => raw.to_uppercase(),
        _ => return None,
    };

    let r = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let g = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let b = u8::from_str_radix(&expanded[4..6], 16).ok()?;
    let alpha = if expanded.len() == 8 {
        u8::from_str_radix(&expanded[6..8], 16).ok()? as f32 / 255.0
    } else {
        1.0
    };

    Some(Color { r, g, b, alpha })
}

fn parse_rgb(input: &str) -> Option<Color> {
    let caps = RGB_PATTERN.captures(input.trim())?;
    let r = parse_channel(caps.get(1)?.as_str())?;
    let g = parse_channel(caps.get(2)?.as_str())?;
    let b = parse_channel(caps.get(3)?.as_str())?;
    let alpha = caps
        .get(4)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);

    Some(Color { r, g, b, alpha })
}

fn parse_hsl(input: &str) -> Option<Color> {
    let caps = HSL_PATTERN.captures(input.trim())?;
    let h = parse_hue(caps.get(1)?.as_str(), caps.get(2).map(|m| m.as_str()))?;
    let s = caps.get(3)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let l = caps.get(4)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let alpha = caps
        .get(5)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Some(Color { r, g, b, alpha })
}

fn parse_channel(value: &str) -> Option<u8> {
    let trimmed = value.trim();
    let parsed = if let Some(percent) = trimmed.strip_suffix('%') {
        percent.parse::<f32>().ok()? * 255.0 / 100.0
    } else {
        trimmed.parse::<f32>().ok()?
    };
    Some(parsed.round().clamp(0.0, 255.0) as u8)
}

fn parse_alpha(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return Some((percent.parse::<f32>().ok()? / 100.0).clamp(0.0, 1.0));
    }
    Some(trimmed.parse::<f32>().ok()?.clamp(0.0, 1.0))
}

fn parse_hue(value: &str, unit: Option<&str>) -> Option<f32> {
    let parsed = value.parse::<f32>().ok()?;
    let degrees = match unit.unwrap_or("deg").to_ascii_lowercase().as_str() {
        "deg" => parsed,
        "rad" => parsed.to_degrees(),
        "turn" => parsed * 360.0,
        "grad" => parsed * 0.9,
        _ => return None,
    };
    Some(degrees)
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0, 0, (l * 100.0).round() as u8);
    }

    let s = delta / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (
        h.round() as u16,
        (s * 100.0).round() as u8,
        (l * 100.0).round() as u8,
    )
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn parse_hsv(input: &str) -> Option<Color> {
    let caps = HSV_PATTERN.captures(input.trim())?;
    let h = parse_hue(caps.get(1)?.as_str(), caps.get(2).map(|m| m.as_str()))?;
    let s = caps.get(3)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let v = caps.get(4)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let alpha = caps
        .get(5)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);
    let (r, g, b) = hsv_to_rgb(h, s, v);

    Some(Color { r, g, b, alpha })
}

fn parse_hwb(input: &str) -> Option<Color> {
    let caps = HWB_PATTERN.captures(input.trim())?;
    let h = parse_hue(caps.get(1)?.as_str(), caps.get(2).map(|m| m.as_str()))?;
    let w = caps.get(3)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0) / 100.0;
    let black = caps.get(4)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0) / 100.0;
    let alpha = caps
        .get(5)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);
    let (r, g, b) = hwb_to_rgb(h, w, black);

    Some(Color { r, g, b, alpha })
}

fn parse_oklch(input: &str) -> Option<Color> {
    let caps = OKLCH_PATTERN.captures(input.trim())?;
    let l = parse_lightness(caps.get(1)?.as_str())?;
    let c = caps.get(2)?.as_str().parse::<f32>().ok()?.max(0.0);
    let h = parse_hue(caps.get(3)?.as_str(), caps.get(4).map(|m| m.as_str()))?.to_radians();
    let alpha = caps
        .get(5)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);

    let a = c * h.cos();
    let b = c * h.sin();
    let (r, g, b) = oklab_to_rgb(l, a, b);

    Some(Color { r, g, b, alpha })
}

fn parse_oklab(input: &str) -> Option<Color> {
    let caps = OKLAB_PATTERN.captures(input.trim())?;
    let l = parse_lightness(caps.get(1)?.as_str())?;
    let a = caps.get(2)?.as_str().parse::<f32>().ok()?;
    let b = caps.get(3)?.as_str().parse::<f32>().ok()?;
    let alpha = caps
        .get(4)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);
    let (r, g, b) = oklab_to_rgb(l, a, b);

    Some(Color { r, g, b, alpha })
}

fn parse_color_function(input: &str) -> Option<Color> {
    let trimmed = input.trim();
    let body = trimmed
        .strip_prefix("color(")
        .or_else(|| trimmed.strip_prefix("COLOR("))?
        .strip_suffix(')')?
        .trim();
    let body = body
        .strip_prefix("srgb")
        .or_else(|| body.strip_prefix("SRGB"))?
        .trim();
    let (channels, alpha) = match body.split_once('/') {
        Some((channels, alpha)) => (channels.trim(), parse_alpha(alpha.trim()).unwrap_or(1.0)),
        None => (body, 1.0),
    };
    let values = channels.split_whitespace().collect::<Vec<_>>();
    if values.len() != 3 {
        return None;
    }

    Some(Color {
        r: parse_srgb_component(values[0])?,
        g: parse_srgb_component(values[1])?,
        b: parse_srgb_component(values[2])?,
        alpha,
    })
}

fn parse_lightness(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return Some((percent.parse::<f32>().ok()? / 100.0).clamp(0.0, 1.0));
    }
    Some(trimmed.parse::<f32>().ok()?.clamp(0.0, 1.0))
}

fn parse_srgb_component(value: &str) -> Option<u8> {
    let trimmed = value.trim();
    let parsed = if let Some(percent) = trimmed.strip_suffix('%') {
        percent.parse::<f32>().ok()? / 100.0
    } else {
        trimmed.parse::<f32>().ok()?
    };
    Some((parsed.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let s = (s / 100.0).clamp(0.0, 1.0);
    let v = (v / 100.0).clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn hwb_to_rgb(h: f32, w: f32, black: f32) -> (u8, u8, u8) {
    if w + black >= 1.0 {
        let gray = w / (w + black);
        let channel = (gray * 255.0).round().clamp(0.0, 255.0) as u8;
        return (channel, channel, channel);
    }

    let (r, g, b) = hsv_to_rgb(h, 100.0, 100.0);
    let factor = 1.0 - w - black;
    let convert = |channel: u8| ((channel as f32 / 255.0 * factor + w) * 255.0).round() as u8;
    (convert(r), convert(g), convert(b))
}

fn oklab_to_rgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (
        linear_srgb_to_channel(r),
        linear_srgb_to_channel(g),
        linear_srgb_to_channel(b),
    )
}

fn linear_srgb_to_channel(value: f32) -> u8 {
    let encoded = if value <= 0.0031308 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Extension for ColorExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &COLOR_MANIFEST
    }

    fn custom_matches(&self, input: &str) -> Option<bool> {
        Some(parse_color(input, ColorParseMode::Launcher).is_some())
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        match parse_color(input, ColorParseMode::Launcher) {
            Some(color) => ExtensionResult::conversion(color.hex()),
            None => ExtensionResult::error("无法识别的颜色格式".to_string()),
        }
    }

    fn execute_command(&self, command_code: &str, input: &str) -> ExtensionResult {
        match command_code {
            "convert" => self.execute(input),
            "pick" => ExtensionResult::error("取色命令需要由前端交互流程处理".to_string()),
            _ => ExtensionResult::error(format!("未知命令: {}", command_code)),
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let color = parse_color(input, ColorParseMode::Launcher)?;
        let hex = color.hex();
        let rgb = color.rgb();
        let hsl = color.hsl();

        Some(ExtensionPreview {
            extension_id: "color".to_string(),
            command_code: "convert".to_string(),
            title: hex.clone(),
            description: format!("{} · {} · 回车复制 Hex", rgb, hsl),
            icon: "palette".to_string(),
            copyable: hex,
            view_type: PreviewViewType::Single,
            grid_data: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_short_and_long() {
        assert_eq!(
            parse_color("#f50", ColorParseMode::Launcher).unwrap().hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("336699", ColorParseMode::Launcher)
                .unwrap()
                .rgb(),
            "rgb(51, 102, 153)"
        );
    }

    #[test]
    fn parses_rgb_and_hsl() {
        assert_eq!(
            parse_color("rgb(255, 85, 0)", ColorParseMode::Launcher)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("hsl(20, 100%, 50%)", ColorParseMode::Launcher)
                .unwrap()
                .hex(),
            "#FF5500"
        );
    }

    #[test]
    fn supports_prefixed_input() {
        assert_eq!(
            parse_color("color #ff5500", ColorParseMode::Launcher)
                .unwrap()
                .hsl(),
            "hsl(20, 100%, 50%)"
        );
    }

    #[test]
    fn dispatches_declared_commands_explicitly() {
        let convert = COLOR_EXTENSION.execute_command("convert", "#ff5500");
        assert!(convert.success);
        assert_eq!(convert.copyable.as_deref(), Some("#FF5500"));

        let pick = COLOR_EXTENSION.execute_command("pick", "");
        assert!(!pick.success);
        assert_eq!(
            pick.error.as_deref(),
            Some("取色命令需要由前端交互流程处理")
        );

        let unknown = COLOR_EXTENSION.execute_command("unknown", "");
        assert!(!unknown.success);
        assert_eq!(unknown.error.as_deref(), Some("未知命令: unknown"));
    }

    #[test]
    fn launcher_mode_rejects_full_only_formats() {
        assert!(parse_color("hsv(20, 100%, 100%)", ColorParseMode::Launcher).is_none());
        assert!(parse_color("oklch(0.7 0.2 40)", ColorParseMode::Launcher).is_none());
        assert!(parse_color("color(srgb 1 0.3333 0)", ColorParseMode::Launcher).is_none());
    }

    #[test]
    fn full_mode_parses_extended_formats() {
        assert_eq!(
            parse_color("hsv(20, 100%, 100%)", ColorParseMode::Full)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("hwb(20 0% 0%)", ColorParseMode::Full)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("color(srgb 1 0.3333 0 / 50%)", ColorParseMode::Full)
                .unwrap()
                .hex(),
            "#FF550080"
        );
        assert!(parse_color("oklch(0.7 0.2 40)", ColorParseMode::Full).is_some());
        assert!(parse_color("oklab(0.7 0.1 0.1)", ColorParseMode::Full).is_some());
    }

    #[test]
    fn supports_modern_rgb_and_hsl_syntax_in_launcher_mode() {
        assert_eq!(
            parse_color("rgb(255 85 0 / 50%)", ColorParseMode::Launcher)
                .unwrap()
                .hex(),
            "#FF550080"
        );
        assert_eq!(
            parse_color("hsl(20deg 100% 50% / .5)", ColorParseMode::Launcher)
                .unwrap()
                .hex(),
            "#FF550080"
        );
    }

    #[test]
    fn supports_hue_units() {
        assert_eq!(
            parse_color("hsl(0.0555556turn 100% 50%)", ColorParseMode::Launcher)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("hsv(0.349066rad 100% 100%)", ColorParseMode::Full)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert_eq!(
            parse_color("hwb(22.2222grad 0% 0%)", ColorParseMode::Full)
                .unwrap()
                .hex(),
            "#FF5500"
        );
        assert!(parse_color("oklch(0.7 0.2 0.111turn)", ColorParseMode::Full).is_some());
    }
}
