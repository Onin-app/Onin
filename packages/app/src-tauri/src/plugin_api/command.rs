use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

// 存储webview插件指令执行请求的状态
pub struct CommandExecutionStore(pub Mutex<HashMap<String, tokio::sync::oneshot::Sender<CommandExecutionResult>>>);

// 存储已加载的插件状态
pub struct PluginLoadedState(pub Mutex<HashMap<String, bool>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandExecutionResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn execute_plugin_command(
    app: AppHandle,
    plugin_store: State<'_, crate::plugin_manager::PluginStore>,
    execution_store: State<'_, CommandExecutionStore>,
    plugin_id: String,
    command_name: String,
    args: Option<serde_json::Value>,
) -> Result<CommandExecutionResult, String> {
    println!("Executing plugin command: {} for plugin: {}", command_name, plugin_id);

    // 获取插件信息
    let plugin = {
        let plugins = plugin_store.0.lock().unwrap();
        plugins.get(&plugin_id).cloned()
    }.ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

    // 根据插件类型执行指令
    println!("[Plugin] Plugin type: {:?}, Entry: {}", plugin.manifest.plugin_type, plugin.manifest.entry);
    
    // 根据entry文件扩展名判断插件类型
    let is_webview_plugin = plugin.manifest.entry.ends_with(".html") || 
                           plugin.manifest.plugin_type.as_deref() == Some("webview");
    
    if is_webview_plugin {
        // Webview插件：通过IPC通信执行
        execute_webview_command(&app, &plugin, &command_name, args, execution_store).await
    } else {
        // Headless插件：通过JS运行时执行
        execute_headless_command(&app, &plugin, &command_name, args).await
    }
}

async fn execute_headless_command(
    app: &AppHandle,
    plugin: &crate::plugin_manager::LoadedPlugin,
    command_name: &str,
    args: Option<serde_json::Value>,
) -> Result<CommandExecutionResult, String> {
    let plugin_id = &plugin.manifest.id;
    let args_json = args.unwrap_or(serde_json::Value::Null);
    
    // 获取插件运行时管理器
    let manager = crate::js_runtime::get_plugin_runtime_manager().await?;
    
    // 检查插件是否需要初始化
    let loaded_state = app.state::<PluginLoadedState>();
    let is_initialized = {
        let state = loaded_state.0.lock().unwrap();
        state.get(plugin_id).copied().unwrap_or(false)
    };
    
    if !is_initialized {
        // 初始化插件
        let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let plugin_dir = data_dir.join("plugins").join(&plugin.dir_name);
        let entry_path = plugin_dir.join(&plugin.manifest.entry);

        if !entry_path.is_file() {
            return Err(format!("Plugin entry file not found: {:?}", entry_path));
        }

        let js_code = std::fs::read_to_string(entry_path).map_err(|e| e.to_string())?;
        
        println!("[Plugin] Initializing plugin: {}", plugin_id);
        manager.init_plugin(plugin_id.clone(), js_code).await?;
        
        // 标记为已初始化
        {
            let mut state = loaded_state.0.lock().unwrap();
            state.insert(plugin_id.clone(), true);
        }
    }
    
    // 执行指令
    println!("[Plugin] Executing command '{}' on plugin: {}", command_name, plugin_id);
    let result = manager.execute_command(
        plugin_id.clone(),
        command_name.to_string(),
        args_json,
    ).await?;
    
    Ok(CommandExecutionResult {
        success: true,
        result: Some(result),
        error: None,
    })
}

async fn execute_webview_command(
    app: &AppHandle,
    plugin: &crate::plugin_manager::LoadedPlugin,
    command_name: &str,
    args: Option<serde_json::Value>,
    execution_store: State<'_, CommandExecutionStore>,
) -> Result<CommandExecutionResult, String> {
    let window_label = format!("plugin_{}", plugin.manifest.id.replace('.', "_"));
    
    // 检查插件窗口是否存在
    let window = app.get_webview_window(&window_label)
        .ok_or_else(|| "Plugin window not found. Please open the plugin first.".to_string())?;

    // 生成请求ID
    let request_id = Uuid::new_v4().to_string();
    
    // 创建响应通道
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut executions = execution_store.0.lock().unwrap();
        executions.insert(request_id.clone(), tx);
    }

    // 发送指令执行事件到插件窗口
    window.emit("plugin_command_execute", serde_json::json!({
        "command": command_name,
        "args": args.unwrap_or(serde_json::Value::Null),
        "requestId": request_id
    })).map_err(|e| e.to_string())?;

    // 等待响应（设置超时）
    match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(_)) => Err("Command execution was cancelled".to_string()),
        Err(_) => {
            // 清理超时的请求
            let mut executions = execution_store.0.lock().unwrap();
            executions.remove(&request_id);
            Err("Command execution timeout".to_string())
        }
    }
}

#[tauri::command]
pub async fn plugin_command_result(
    execution_store: State<'_, CommandExecutionStore>,
    request_id: String,
    success: bool,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<(), String> {
    let sender = {
        let mut executions = execution_store.0.lock().unwrap();
        executions.remove(&request_id)
    };

    if let Some(sender) = sender {
        let execution_result = CommandExecutionResult {
            success,
            result,
            error,
        };
        
        let _ = sender.send(execution_result);
    }

    Ok(())
}