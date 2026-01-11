//! 命令生成器模块
//!
//! 按类型生成各种命令

mod app;
mod file;
mod plugin;
mod system;

pub use app::get_initial_app_commands;
pub use file::get_initial_file_commands;
pub use plugin::{
    get_initial_dynamic_commands, get_initial_plugin_commands, get_plugin_commands,
    get_plugin_id_name_mapping,
};
pub use system::get_initial_system_commands;

/// 清理命令名称部分
///
/// 替换可能导致问题的字符
pub fn sanitize_command_name_part(s: &str) -> String {
    s.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_")
}
