use deno_core::op2;
use deno_core::{JsRuntime, OpState, PollEventLoopOptions, RuntimeOptions};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot};

use crate::plugin_api;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum InvokeResult {
    #[serde(rename = "ok")]
    Ok { value: serde_json::Value },
    #[serde(rename = "error")]
    Err { error: String },
}

// 同步 op 用于console输出
#[op2(fast)]
fn op_console_log(state: Rc<RefCell<OpState>>, #[string] message: String) {
    println!("[Plugin Console] {}", message);

    // 尝试发送到前端
    if let Ok(app_handle) = state.try_borrow().map(|s| s.borrow::<AppHandle>().clone()) {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit(
                "plugin_console_log",
                serde_json::json!({
                    "message": message,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }),
            );
        }
    }
}

// 异步 op
#[op2(async)]
#[serde]
async fn op_invoke(
    state: Rc<RefCell<OpState>>,
    #[string] method: String,
    #[serde] arg: serde_json::Value,
) -> InvokeResult {
    println!("插件异步调用 invoke: method={}, arg={}", method, arg);

    let app_handle = state.borrow().borrow::<AppHandle>().clone();

    match method.as_str() {
        "show_notification" => {
            // 首先，尝试直接从 Value 反序列化
            let options_result = serde_json::from_value::<
                plugin_api::notification::NotificationOptions,
            >(arg.clone());

            // 如果失败，并且 arg 是一个字符串，则尝试将该字符串作为 JSON 解析
            let final_options = match options_result {
                Ok(options) => Ok(options),
                Err(_) => {
                    if let Some(s) = arg.as_str() {
                        serde_json::from_str(s)
                    } else {
                        // Provide a type annotation for the compiler
                        let err_result: Result<plugin_api::notification::NotificationOptions, _> =
                            serde_json::from_str("");
                        Err(err_result.unwrap_err())
                    }
                }
            };

            match final_options {
                Ok(options) => {
                    match plugin_api::notification::show_notification(app_handle, options) {
                        Ok(_) => InvokeResult::Ok { value: serde_json::Value::Null },
                        Err(e) => InvokeResult::Err { error: e },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!(
                        "Invalid argument for show_notification: {}. Original arg: {}",
                        e, arg
                    ),
                },
            }
        }

        "plugin_request" => {
            // 处理两种格式：直接的 RequestOptions 或 {"options": RequestOptions}
            let request_options = if let Some(options) = arg.get("options") {
                options.clone()
            } else {
                arg.clone()
            };

            let options_result = serde_json::from_value::<
                plugin_api::request::RequestOptions,
            >(request_options.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::request::make_request(app_handle, options).await {
                        Ok(response) => InvokeResult::Ok { value: response },
                        Err(e) => InvokeResult::Err { error: e },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_request: {}", e),
                },
            }
        }

        _ => InvokeResult::Err {
            error: "unknown method".to_string(),
        },
    }
}

// 定义扩展
deno_core::extension!(
    baize_plugin_api,
    ops = [op_invoke, op_console_log],
    options = {
        app_handle: AppHandle,
    },
    state = |state, options| {
        state.put(options.app_handle);
    },
);

pub fn create_runtime(app_handle: &AppHandle) -> Result<JsRuntime, String> {
    let ext = baize_plugin_api::init(app_handle.clone());

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    // 设置全局对象和函数
    let global_setup = r#"
        // 重写console.log以使用我们的op
        globalThis.console = {
            log: (...args) => {
                const message = args.map(arg => 
                    typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                ).join(' ');
                Deno.core.ops.op_console_log(message);
            },
            error: (...args) => {
                const message = '[ERROR] ' + args.map(arg => 
                    typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                ).join(' ');
                Deno.core.ops.op_console_log(message);
            },
            warn: (...args) => {
                const message = '[WARN] ' + args.map(arg => 
                    typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
                ).join(' ');
                Deno.core.ops.op_console_log(message);
            }
        };


    "#;

    // 设置全局对象
    runtime
        .execute_script("<global_setup>", global_setup.to_string())
        .map_err(|e| e.to_string())?;

    Ok(runtime)
}

// 插件任务类型
#[derive(Debug)]
pub enum PluginTask {
    InitPlugin {
        plugin_id: String,
        js_code: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    ExecuteCommand {
        plugin_id: String,
        command: String,
        args: serde_json::Value,
        response: oneshot::Sender<Result<serde_json::Value, String>>,
    },
}

// 全局插件运行时管理器
pub struct PluginRuntimeManager {
    sender: mpsc::UnboundedSender<PluginTask>,
}

impl PluginRuntimeManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<PluginTask>();

        // 在专门的线程中运行JS运行时
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let mut runtimes: HashMap<String, JsRuntime> = HashMap::new();

                while let Some(task) = receiver.recv().await {
                    match task {
                        PluginTask::InitPlugin {
                            plugin_id,
                            js_code,
                            response,
                        } => {
                            let result = Self::handle_init_plugin(
                                &mut runtimes,
                                &app_handle,
                                &plugin_id,
                                &js_code,
                            )
                            .await;
                            let _ = response.send(result);
                        }
                        PluginTask::ExecuteCommand {
                            plugin_id,
                            command,
                            args,
                            response,
                        } => {
                            let result = Self::handle_execute_command(
                                &mut runtimes,
                                &plugin_id,
                                &command,
                                args,
                            )
                            .await;
                            let _ = response.send(result);
                        }
                    }
                }
            });
        });

        Self { sender }
    }

    async fn handle_init_plugin(
        runtimes: &mut HashMap<String, JsRuntime>,
        app_handle: &AppHandle,
        plugin_id: &str,
        js_code: &str,
    ) -> Result<(), String> {
        if runtimes.contains_key(plugin_id) {
            return Ok(()); // 已经初始化过了
        }

        let mut runtime = create_runtime(app_handle)?;

        // 执行插件初始化代码
        runtime
            .execute_script("<plugin_init>", js_code.to_string())
            .map_err(|e| e.to_string())?;

        // 运行事件循环完成初始化
        runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string())?;

        runtimes.insert(plugin_id.to_string(), runtime);
        println!("[Plugin] Initialized plugin runtime: {}", plugin_id);

        Ok(())
    }

    async fn handle_execute_command(
        runtimes: &mut HashMap<String, JsRuntime>,
        plugin_id: &str,
        command: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let runtime = runtimes
            .get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin runtime not found: {}", plugin_id))?;

        // 构造调用代码
        let call_code = format!(
            r#"
            (async () => {{
                try {{
                    if (typeof globalThis.__BAIZE_COMMAND_HANDLER__ === 'function') {{
                        const result = await globalThis.__BAIZE_COMMAND_HANDLER__('{}', {});
                        console.log('Command execution result:', JSON.stringify(result));
                        return result;
                    }} else {{
                        throw new Error('No command handler registered');
                    }}
                }} catch (error) {{
                    console.error('Command execution error:', error);
                    throw error;
                }}
            }})();
            "#,
            command, args
        );

        // 执行调用
        runtime
            .execute_script("<command_call>", call_code)
            .map_err(|e| e.to_string())?;

        // 运行事件循环
        runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string())?;

        // 这里应该返回实际的结果，但由于deno_core的限制，我们先返回null
        Ok(serde_json::Value::Null)
    }

    pub async fn init_plugin(&self, plugin_id: String, js_code: String) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(PluginTask::InitPlugin {
                plugin_id,
                js_code,
                response: response_tx,
            })
            .map_err(|_| "Failed to send init plugin task")?;

        response_rx
            .await
            .map_err(|_| "Failed to receive init plugin response")?
    }

    pub async fn execute_command(
        &self,
        plugin_id: String,
        command: String,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send(PluginTask::ExecuteCommand {
                plugin_id,
                command,
                args,
                response: response_tx,
            })
            .map_err(|_| "Failed to send execute command task")?;

        response_rx
            .await
            .map_err(|_| "Failed to receive execute command response")?
    }
}

// 全局插件运行时管理器实例
static PLUGIN_RUNTIME_MANAGER: Lazy<Arc<tokio::sync::Mutex<Option<PluginRuntimeManager>>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

pub async fn init_plugin_runtime_manager(app_handle: AppHandle) {
    let mut manager = PLUGIN_RUNTIME_MANAGER.lock().await;
    if manager.is_none() {
        *manager = Some(PluginRuntimeManager::new(app_handle));
    }
}

pub async fn get_plugin_runtime_manager() -> Result<PluginRuntimeManager, String> {
    let manager = PLUGIN_RUNTIME_MANAGER.lock().await;
    manager
        .as_ref()
        .ok_or_else(|| "Plugin runtime manager not initialized".to_string())
        .map(|m| PluginRuntimeManager {
            sender: m.sender.clone(),
        })
}

pub async fn execute_js(app_handle: &AppHandle, js_code: &str) -> Result<(), String> {
    let mut runtime = create_runtime(app_handle)?;

    // 执行插件代码
    let js_code_owned = js_code.to_string();
    let result = runtime.execute_script("<plugin>", js_code_owned);
    match result {
        Ok(_) => runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
