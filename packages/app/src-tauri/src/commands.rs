//! 集中管理所有 Tauri 命令注册
//!
//! 这个模块将所有分散在各个模块中的 Tauri 命令汇总到一个地方，
//! 使 `lib.rs` 更加简洁，同时方便统一管理和查找命令。

use crate::{
    app_config, command_manager, file_command_manager, plugin, plugin_api, shortcut_manager,
    system_commands, tray_manager, unified_launch_manager, usage_tracker, window_manager,
};
use tauri::Manager;

/// 生成包含所有 Tauri 命令的 invoke handler
pub fn get_invoke_handler(
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // Basic
        greet,
        // Unified launch manager
        unified_launch_manager::get_all_launchable_items,
        // Window manager
        window_manager::acquire_window_close_lock,
        window_manager::release_window_close_lock,
        window_manager::close_main_window,
        // Tray manager
        tray_manager::set_tray_visibility,
        tray_manager::is_tray_visible,
        // Shortcut manager
        shortcut_manager::commands::get_shortcuts,
        shortcut_manager::commands::add_shortcut,
        shortcut_manager::commands::remove_shortcut,
        shortcut_manager::commands::set_toggle_shortcut,
        shortcut_manager::commands::get_toggle_shortcut,
        shortcut_manager::commands::set_detach_window_shortcut,
        shortcut_manager::commands::get_detach_window_shortcut,
        // File command manager
        file_command_manager::get_file_commands,
        file_command_manager::add_file_commands,
        file_command_manager::remove_file_command,
        // System commands
        system_commands::execute_command,
        system_commands::get_basic_commands,
        // Plugin loader
        plugin::loader::load_plugins,
        plugin::loader::get_loaded_plugins,
        plugin::loader::refresh_plugins,
        // Plugin window
        plugin::window::open_plugin_in_window,
        plugin::window::plugin_close_window,
        plugin::window::plugin_minimize_window,
        plugin::window::plugin_maximize_window,
        plugin::window::plugin_unmaximize_window,
        plugin::window::plugin_is_maximized,
        plugin::window::plugin_show_window,
        plugin::window::plugin_set_focus,
        plugin::window::plugin_start_dragging,
        // Plugin executor
        plugin::executor::execute_plugin_entry,
        // Plugin settings
        plugin::settings::toggle_plugin,
        plugin::settings::toggle_plugin_auto_detach,
        plugin::settings::register_plugin_settings_schema,
        plugin::settings::get_plugin_settings,
        plugin::settings::save_plugin_settings,
        plugin::settings::get_plugin_with_schema,
        plugin::settings::get_plugin_detail,
        plugin::settings::get_plugin_server_port,
        // Plugin installer
        plugin::installer::import_plugin,
        plugin::installer::uninstall_plugin,
        plugin::installer::download_and_install_plugin,
        // Plugin API: notification
        plugin_api::notification::show_notification,
        // Plugin API: command
        plugin_api::command::execute_plugin_command,
        plugin_api::command::plugin_command_result,
        // Plugin API: request
        plugin_api::request::plugin_request,
        // Plugin API: storage
        plugin_api::storage::plugin_storage_set,
        plugin_api::storage::plugin_storage_get,
        plugin_api::storage::plugin_storage_remove,
        plugin_api::storage::plugin_storage_clear,
        plugin_api::storage::plugin_storage_keys,
        plugin_api::storage::plugin_storage_set_items,
        plugin_api::storage::plugin_storage_get_items,
        // Plugin API: filesystem
        plugin_api::fs::plugin_fs_read_file,
        plugin_api::fs::plugin_fs_write_file,
        plugin_api::fs::plugin_fs_exists,
        plugin_api::fs::plugin_fs_create_dir,
        plugin_api::fs::plugin_fs_list_dir,
        plugin_api::fs::plugin_fs_delete_file,
        plugin_api::fs::plugin_fs_delete_dir,
        plugin_api::fs::plugin_fs_get_file_info,
        plugin_api::fs::plugin_fs_copy_file,
        plugin_api::fs::plugin_fs_move_file,
        // Plugin API: dialog
        plugin_api::dialog::plugin_dialog_message,
        plugin_api::dialog::plugin_dialog_confirm,
        plugin_api::dialog::plugin_dialog_open,
        plugin_api::dialog::plugin_dialog_save,
        // Plugin API: clipboard
        plugin_api::clipboard::commands::plugin_clipboard_read_text,
        plugin_api::clipboard::commands::plugin_clipboard_write_text,
        plugin_api::clipboard::commands::plugin_clipboard_read_image,
        plugin_api::clipboard::commands::plugin_clipboard_write_image,
        plugin_api::clipboard::commands::plugin_clipboard_clear,
        plugin_api::clipboard::commands::plugin_clipboard_get_metadata,
        plugin_api::clipboard::commands::get_clipboard_content,
        // Plugin API: scheduler
        plugin_api::scheduler::schedule_task,
        plugin_api::scheduler::cancel_task,
        plugin_api::scheduler::list_tasks,
        // Command manager
        command_manager::commands::get_commands,
        command_manager::commands::update_command,
        command_manager::commands::refresh_commands,
        command_manager::commands::get_plugin_commands_list,
        command_manager::commands::get_plugin_id_mapping,
        // App config
        app_config::get_app_config,
        app_config::update_app_config,
        // Usage tracker
        usage_tracker::record_command_usage,
        usage_tracker::get_usage_stats,
        usage_tracker::clear_usage_stats,
    ]
}

/// 简单的问候命令，用于测试
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
