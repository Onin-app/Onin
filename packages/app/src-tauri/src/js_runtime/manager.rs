//! 插件运行时管理器模块
//!
//! 管理多个插件的 JS 运行时实例，支持初始化、执行命令和清除

use deno_core::{JsRuntime, PollEventLoopOptions};
use std::collections::HashMap;
use tauri::AppHandle;
use tokio::sync::{mpsc, oneshot};

use super::runtime::create_runtime_with_plugin_id;
use super::types::PluginTask;

/// 插件运行时管理器
///
/// 使用消息传递机制在专用线程中管理所有插件的 JS 运行时
pub struct PluginRuntimeManager {
    pub(crate) sender: mpsc::UnboundedSender<PluginTask>,
}

impl PluginRuntimeManager {
    const EXECUTE_UNLOAD_SCRIPT: &'static str = r#"
        (async () => {
            if (typeof globalThis.__ONIN_EXECUTE_UNLOAD_CALLBACKS__ === 'function') {
                await globalThis.__ONIN_EXECUTE_UNLOAD_CALLBACKS__();
            }
        })().catch((error) => {
            console.error('Plugin unload execution error:', error);
            throw error;
        });
    "#;

    /// 创建新的运行时管理器
    ///
    /// 在专用线程中启动事件循环来处理所有插件任务
    pub fn new(app_handle: AppHandle) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<PluginTask>();

        // 在专门的线程中运行 JS 运行时
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
                        PluginTask::ClearPlugin {
                            plugin_id,
                            response,
                        } => {
                            let result = Self::handle_clear_plugin(&mut runtimes, &plugin_id).await;
                            let _ = response.send(result);
                        }
                        PluginTask::ClearAllPlugins { response } => {
                            let result = Self::handle_clear_all_plugins(&mut runtimes).await;
                            let _ = response.send(result);
                        }
                    }
                }
            });
        });

        Self { sender }
    }

    // ========================================================================
    // 内部处理方法
    // ========================================================================

    /// 处理插件初始化
    async fn handle_init_plugin(
        runtimes: &mut HashMap<String, JsRuntime>,
        app_handle: &AppHandle,
        plugin_id: &str,
        js_code: &str,
    ) -> Result<(), String> {
        // 如果插件已经存在，先清除它（支持热重载）
        if let Some(existing_runtime) = runtimes.remove(plugin_id) {
            Self::execute_unload(existing_runtime).await?;
        }

        let mut runtime = create_runtime_with_plugin_id(app_handle, plugin_id)?;

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

        Ok(())
    }

    /// 处理清除单个插件
    async fn handle_clear_plugin(
        runtimes: &mut HashMap<String, JsRuntime>,
        plugin_id: &str,
    ) -> Result<(), String> {
        if let Some(runtime) = runtimes.remove(plugin_id) {
            Self::execute_unload(runtime).await?;
        }
        Ok(())
    }

    /// 处理清除所有插件
    async fn handle_clear_all_plugins(
        runtimes: &mut HashMap<String, JsRuntime>,
    ) -> Result<(), String> {
        let existing_runtimes = std::mem::take(runtimes);
        for (_plugin_id, runtime) in existing_runtimes {
            Self::execute_unload(runtime).await?;
        }
        Ok(())
    }

    async fn execute_unload(mut runtime: JsRuntime) -> Result<(), String> {
        runtime
            .execute_script("<plugin_unload>", Self::EXECUTE_UNLOAD_SCRIPT.to_string())
            .map_err(|e| e.to_string())?;

        runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string())
    }

    /// 处理命令执行
    async fn handle_execute_command(
        runtimes: &mut HashMap<String, JsRuntime>,
        plugin_id: &str,
        command: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let runtime = runtimes
            .get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin runtime not found: {}", plugin_id))?;

        // 将 args 序列化为 JSON 字符串
        let args_json = serde_json::to_string(&args).map_err(|e| e.to_string())?;

        // 构造调用代码
        // 注意：第二个参数直接传递 JSON 对象（不用引号包裹）
        let call_code = format!(
            r#"
            (async () => {{
                try {{
                    if (typeof globalThis.__ONIN_COMMAND_HANDLER__ === 'function') {{
                        const result = await globalThis.__ONIN_COMMAND_HANDLER__('{}', {});                        return result;
                    }} else {{
                        throw new Error('No command handler registered');
                    }}
                }} catch (error) {{
                    console.error('Command execution error:', error);
                    throw error;
                }}
            }})();
            "#,
            command, args_json
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

        // 这里应该返回实际的结果，但由于 deno_core 的限制，我们先返回 null
        Ok(serde_json::Value::Null)
    }

    // ========================================================================
    // 公共 API
    // ========================================================================

    /// 初始化插件
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

    /// 执行命令
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

    /// 清除单个插件
    pub async fn clear_plugin(&self, plugin_id: String) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();
        self.sender
            .send(PluginTask::ClearPlugin {
                plugin_id,
                response: response_tx,
            })
            .map_err(|_| "Failed to send clear plugin task")?;

        response_rx
            .await
            .map_err(|_| "Failed to receive clear plugin response")?
    }

    /// 清除所有插件
    pub async fn clear_all_plugins(&self) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();
        self.sender
            .send(PluginTask::ClearAllPlugins {
                response: response_tx,
            })
            .map_err(|_| "Failed to send clear all plugins task")?;

        response_rx
            .await
            .map_err(|_| "Failed to receive clear all plugins response")?
    }
}
