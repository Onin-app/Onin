//! 快捷键状态类型定义

use crate::shared_types::Shortcut as AppShortcut;
use std::sync::Mutex;

/// 保存当前配置的快捷键
pub struct ShortcutState {
    pub shortcuts: Mutex<Vec<AppShortcut>>,
    pub last_executed: Mutex<std::collections::HashMap<String, std::time::Instant>>,
}
