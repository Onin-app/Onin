use crate::shared_types::PluginCommand;

#[tauri::command]
pub async fn register_plugin_command(command: PluginCommand) {
    println!("Registering command: {:?}", command);
}