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
    commands: &[ExtensionCommand {
        code: "convert",
        name: "颜色转换",
        description: Some("解析 Hex、RGB、HSL 颜色并转换格式"),
        icon: Some("palette"),
        keywords: &[
            "color", "colour", "颜色", "色值", "hex", "rgb", "rgba", "hsl", "hsla", "取色",
        ],
        matches: None,
    }],
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
        ([+-]?\d+(?:\.\d+)?)\s*(?:,|\s)\s*
        ([+-]?\d+(?:\.\d+)?)\s*(?:,|\s)\s*
        ([+-]?\d+(?:\.\d+)?)
        (?:\s*(?:,|/)\s*([+-]?\d+(?:\.\d+)?%?))?
        \s*\)$",
    )
    .unwrap()
});

static HSL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)^hsla?\(\s*
        ([+-]?\d+(?:\.\d+)?)(?:deg)?\s*(?:,|\s)\s*
        ([+-]?\d+(?:\.\d+)?)%\s*(?:,|\s)\s*
        ([+-]?\d+(?:\.\d+)?)%
        (?:\s*(?:,|/)\s*([+-]?\d+(?:\.\d+)?%?))?
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

fn parse_color(input: &str) -> Option<Color> {
    let value = strip_color_prefix(input);
    parse_hex(value)
        .or_else(|| parse_rgb(value))
        .or_else(|| parse_hsl(value))
}

pub fn convert_color_value(input: &str) -> Option<ColorConversion> {
    parse_color(input).map(ColorConversion::from)
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
    let h = caps.get(1)?.as_str().parse::<f32>().ok()?;
    let s = caps.get(2)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let l = caps.get(3)?.as_str().parse::<f32>().ok()?.clamp(0.0, 100.0);
    let alpha = caps
        .get(4)
        .and_then(|m| parse_alpha(m.as_str()))
        .unwrap_or(1.0);
    let (r, g, b) = hsl_to_rgb(h, s, l);

    Some(Color { r, g, b, alpha })
}

fn parse_channel(value: &str) -> Option<u8> {
    let parsed = value.parse::<f32>().ok()?;
    Some(parsed.round().clamp(0.0, 255.0) as u8)
}

fn parse_alpha(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return Some((percent.parse::<f32>().ok()? / 100.0).clamp(0.0, 1.0));
    }
    Some(trimmed.parse::<f32>().ok()?.clamp(0.0, 1.0))
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

impl Extension for ColorExtension {
    fn manifest(&self) -> &'static ExtensionManifest {
        &COLOR_MANIFEST
    }

    fn custom_matches(&self, input: &str) -> Option<bool> {
        Some(parse_color(input).is_some())
    }

    fn execute(&self, input: &str) -> ExtensionResult {
        match parse_color(input) {
            Some(color) => ExtensionResult::conversion(color.hex()),
            None => ExtensionResult::error("无法识别的颜色格式".to_string()),
        }
    }

    fn preview(&self, input: &str) -> Option<ExtensionPreview> {
        let color = parse_color(input)?;
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
        assert_eq!(parse_color("#f50").unwrap().hex(), "#FF5500");
        assert_eq!(parse_color("336699").unwrap().rgb(), "rgb(51, 102, 153)");
    }

    #[test]
    fn parses_rgb_and_hsl() {
        assert_eq!(parse_color("rgb(255, 85, 0)").unwrap().hex(), "#FF5500");
        assert_eq!(parse_color("hsl(20, 100%, 50%)").unwrap().hex(), "#FF5500");
    }

    #[test]
    fn supports_prefixed_input() {
        assert_eq!(
            parse_color("color #ff5500").unwrap().hsl(),
            "hsl(20, 100%, 50%)"
        );
    }
}
