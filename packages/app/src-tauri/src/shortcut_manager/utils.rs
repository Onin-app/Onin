//! 工具函数模块

/// 标准化快捷键字符串以便比较
pub fn normalize_shortcut_string(shortcut_str: &str) -> String {
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for part in parts.iter() {
        let trimmed_part = part.trim();
        if trimmed_part.is_empty() {
            continue;
        }

        let lower_part = trimmed_part.to_lowercase();
        match lower_part.as_str() {
            "ctrl" | "control" => modifiers.push("ctrl"),
            "alt" => modifiers.push("alt"),
            "shift" => modifiers.push("shift"),
            "cmd" | "command" | "meta" | "super" => modifiers.push("cmd"),
            "commandorcontrol" => {
                #[cfg(target_os = "macos")]
                modifiers.push("cmd");
                #[cfg(not(target_os = "macos"))]
                modifiers.push("ctrl");
            }
            _ => {
                // 处理 "KeyN" -> "N" 或 "B" -> "B"
                let mut key_part = trimmed_part;
                if key_part.starts_with("Key") && key_part.len() > 3 {
                    key_part = &key_part[3..];
                }
                key = key_part.to_uppercase();
            }
        }
    }

    modifiers.sort();

    if key.is_empty() {
        modifiers.join("+")
    } else if modifiers.is_empty() {
        key
    } else {
        format!("{}+{}", modifiers.join("+"), key)
    }
}

/// 检查 macOS 辅助功能权限
#[cfg(target_os = "macos")]
pub fn check_accessibility_permissions() -> bool {
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get name of every process")
        .output();

    matches!(output, Ok(_))
}

/// 非 macOS 平台的占位实现
#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permissions() -> bool {
    true
}
