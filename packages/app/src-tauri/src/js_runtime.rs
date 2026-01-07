use deno_core::op2;
use deno_core::{JsRuntime, ModuleSpecifier, OpState, PollEventLoopOptions, RuntimeOptions};
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
    let plugin_id = state.borrow().borrow::<String>().clone();

    // 设置当前插件ID到线程本地存储
    crate::plugin_api::storage::set_current_plugin_id(plugin_id.clone());

    let result = match method.as_str() {
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
                        Ok(_) => InvokeResult::Ok {
                            value: serde_json::Value::Null,
                        },
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

            let options_result = serde_json::from_value::<plugin_api::request::RequestOptions>(
                request_options.clone(),
            );

            match options_result {
                Ok(options) => match plugin_api::request::make_request(app_handle, options).await {
                    Ok(response) => InvokeResult::Ok { value: response },
                    Err(e) => InvokeResult::Err { error: e },
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_request: {}", e),
                },
            }
        }

        "plugin_storage_set" => {
            let key = match arg.get("key").and_then(|v| v.as_str()) {
                Some(k) => k.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "key is required".to_string(),
                    }
                }
            };
            let value = match arg.get("value") {
                Some(v) => v.clone(),
                None => {
                    return InvokeResult::Err {
                        error: "value is required".to_string(),
                    }
                }
            };

            match plugin_api::storage::plugin_storage_set(app_handle, key, value).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        "plugin_storage_get" => {
            let key = match arg.get("key").and_then(|v| v.as_str()) {
                Some(k) => k.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "key is required".to_string(),
                    }
                }
            };

            match plugin_api::storage::plugin_storage_get(app_handle, key).await {
                Ok(value) => InvokeResult::Ok {
                    value: value.unwrap_or(serde_json::Value::Null),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        "plugin_storage_remove" => {
            let key = match arg.get("key").and_then(|v| v.as_str()) {
                Some(k) => k.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "key is required".to_string(),
                    }
                }
            };

            match plugin_api::storage::plugin_storage_remove(app_handle, key).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        "plugin_storage_clear" => {
            match plugin_api::storage::plugin_storage_clear(app_handle).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        "plugin_storage_keys" => match plugin_api::storage::plugin_storage_keys(app_handle).await {
            Ok(keys) => InvokeResult::Ok {
                value: serde_json::json!(keys),
            },
            Err(e) => InvokeResult::Err {
                error: format!("Storage error: {:?}", e),
            },
        },

        "plugin_storage_set_items" => {
            let items = match arg.get("items") {
                Some(items_value) => match items_value.as_object() {
                    Some(obj) => obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    None => {
                        return InvokeResult::Err {
                            error: "items must be an object".to_string(),
                        }
                    }
                },
                None => {
                    return InvokeResult::Err {
                        error: "items is required".to_string(),
                    }
                }
            };

            match plugin_api::storage::plugin_storage_set_items(app_handle, items).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        "plugin_storage_get_items" => {
            let keys = match arg.get("keys") {
                Some(keys_value) => match keys_value.as_array() {
                    Some(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                    None => {
                        return InvokeResult::Err {
                            error: "keys must be an array".to_string(),
                        }
                    }
                },
                None => {
                    return InvokeResult::Err {
                        error: "keys is required".to_string(),
                    }
                }
            };

            match plugin_api::storage::plugin_storage_get_items(app_handle, keys).await {
                Ok(items) => InvokeResult::Ok {
                    value: serde_json::json!(items),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Storage error: {:?}", e),
                },
            }
        }

        // 文件系统 API
        "plugin_fs_read_file" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_read_file(app_handle, path).await {
                Ok(content) => InvokeResult::Ok {
                    value: serde_json::json!(content),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_write_file" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };
            let content = match arg.get("content").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "content is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_write_file(app_handle, path, content).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_exists" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_exists(app_handle, path).await {
                Ok(exists) => InvokeResult::Ok {
                    value: serde_json::json!(exists),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_create_dir" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };
            let recursive = arg
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            match plugin_api::fs::plugin_fs_create_dir(app_handle, path, recursive).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_list_dir" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_list_dir(app_handle, path).await {
                Ok(files) => InvokeResult::Ok {
                    value: serde_json::json!(files),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_delete_file" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_delete_file(app_handle, path).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_delete_dir" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };
            let recursive = arg
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            match plugin_api::fs::plugin_fs_delete_dir(app_handle, path, recursive).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_get_file_info" => {
            let path = match arg.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "path is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_get_file_info(app_handle, path).await {
                Ok(info) => InvokeResult::Ok {
                    value: serde_json::json!(info),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_copy_file" => {
            let source_path = match arg.get("sourcePath").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "sourcePath is required".to_string(),
                    }
                }
            };
            let dest_path = match arg.get("destPath").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "destPath is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_copy_file(app_handle, source_path, dest_path).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        "plugin_fs_move_file" => {
            let source_path = match arg.get("sourcePath").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "sourcePath is required".to_string(),
                    }
                }
            };
            let dest_path = match arg.get("destPath").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "destPath is required".to_string(),
                    }
                }
            };

            match plugin_api::fs::plugin_fs_move_file(app_handle, source_path, dest_path).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("File system error: {:?}", e),
                },
            }
        }

        // Dialog API
        "plugin_dialog_message" => {
            let options_result =
                serde_json::from_value::<plugin_api::dialog::MessageDialogOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::dialog::plugin_dialog_message(app_handle, options).await {
                        Ok(_) => InvokeResult::Ok {
                            value: serde_json::Value::Null,
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Dialog error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_dialog_message: {}", e),
                },
            }
        }

        "plugin_dialog_confirm" => {
            let options_result =
                serde_json::from_value::<plugin_api::dialog::ConfirmDialogOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::dialog::plugin_dialog_confirm(app_handle, options).await {
                        Ok(result) => InvokeResult::Ok {
                            value: serde_json::json!(result),
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Dialog error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_dialog_confirm: {}", e),
                },
            }
        }

        "plugin_dialog_open" => {
            let options_result =
                serde_json::from_value::<plugin_api::dialog::OpenDialogOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::dialog::plugin_dialog_open(app_handle, options).await {
                        Ok(result) => InvokeResult::Ok {
                            value: result.unwrap_or(serde_json::Value::Null),
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Dialog error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_dialog_open: {}", e),
                },
            }
        }

        "plugin_dialog_save" => {
            let options_result =
                serde_json::from_value::<plugin_api::dialog::SaveDialogOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::dialog::plugin_dialog_save(app_handle, options).await {
                        Ok(result) => InvokeResult::Ok {
                            value: result
                                .map(serde_json::Value::String)
                                .unwrap_or(serde_json::Value::Null),
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Dialog error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_dialog_save: {}", e),
                },
            }
        }

        // Clipboard API
        "plugin_clipboard_read_text" => {
            match plugin_api::clipboard::plugin_clipboard_read_text(app_handle).await {
                Ok(text) => InvokeResult::Ok {
                    value: text
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Clipboard error: {:?}", e),
                },
            }
        }

        "plugin_clipboard_write_text" => {
            let options_result =
                serde_json::from_value::<plugin_api::clipboard::WriteTextOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::clipboard::plugin_clipboard_write_text(app_handle, options)
                        .await
                    {
                        Ok(_) => InvokeResult::Ok {
                            value: serde_json::Value::Null,
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Clipboard error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_clipboard_write_text: {}", e),
                },
            }
        }

        "plugin_clipboard_read_image" => {
            match plugin_api::clipboard::plugin_clipboard_read_image(app_handle).await {
                Ok(image) => InvokeResult::Ok {
                    value: image
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Clipboard error: {:?}", e),
                },
            }
        }

        "plugin_clipboard_write_image" => {
            let options_result =
                serde_json::from_value::<plugin_api::clipboard::WriteImageOptions>(arg.clone());

            match options_result {
                Ok(options) => {
                    match plugin_api::clipboard::plugin_clipboard_write_image(app_handle, options)
                        .await
                    {
                        Ok(_) => InvokeResult::Ok {
                            value: serde_json::Value::Null,
                        },
                        Err(e) => InvokeResult::Err {
                            error: format!("Clipboard error: {:?}", e),
                        },
                    }
                }
                Err(e) => InvokeResult::Err {
                    error: format!("Invalid argument for plugin_clipboard_write_image: {}", e),
                },
            }
        }

        "plugin_clipboard_clear" => {
            match plugin_api::clipboard::plugin_clipboard_clear(app_handle).await {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Clipboard error: {:?}", e),
                },
            }
        }

        "plugin_clipboard_get_metadata" => {
            match plugin_api::clipboard::plugin_clipboard_get_metadata(app_handle).await {
                Ok(metadata) => InvokeResult::Ok {
                    value: serde_json::to_value(metadata).unwrap_or(serde_json::Value::Null),
                },
                Err(e) => InvokeResult::Err {
                    error: format!("Clipboard error: {:?}", e),
                },
            }
        }

        "register_plugin_settings_schema" => {
            println!("[js_runtime] Handling register_plugin_settings_schema");

            // 从 arg 中提取 pluginId 和 schema
            let plugin_id_from_arg = match arg.get("pluginId").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => {
                    return InvokeResult::Err {
                        error: "Missing pluginId in register_plugin_settings_schema".to_string(),
                    };
                }
            };

            // 🔒 安全检查：验证插件只能注册自己的设置
            if plugin_id_from_arg != plugin_id {
                eprintln!(
                    "[js_runtime] Security violation: Plugin '{}' attempted to register settings for plugin '{}'",
                    plugin_id, plugin_id_from_arg
                );
                return InvokeResult::Err {
                    error: format!(
                        "Permission denied: Plugin can only register its own settings (expected: {}, got: {})",
                        plugin_id, plugin_id_from_arg
                    ),
                };
            }

            let schema_value = match arg.get("schema") {
                Some(s) => s.clone(),
                None => {
                    return InvokeResult::Err {
                        error: "Missing schema in register_plugin_settings_schema".to_string(),
                    };
                }
            };

            // 反序列化 schema
            let schema: crate::plugin::PluginSettingsSchema =
                match serde_json::from_value(schema_value) {
                    Ok(s) => s,
                    Err(e) => {
                        return InvokeResult::Err {
                            error: format!("Failed to deserialize schema: {}", e),
                        };
                    }
                };

            // 调用 plugin_manager 的函数
            let store = app_handle.state::<crate::plugin::PluginStore>();
            match crate::plugin::register_plugin_settings_schema(
                app_handle.clone(),
                store,
                plugin_id_from_arg,
                schema,
            ) {
                Ok(_) => InvokeResult::Ok {
                    value: serde_json::Value::Null,
                },
                Err(e) => InvokeResult::Err { error: e },
            }
        }

        _ => InvokeResult::Err {
            error: "unknown method".to_string(),
        },
    };

    // 清除当前插件ID
    crate::plugin_api::storage::clear_current_plugin_id();

    result
}

// 定义扩展
deno_core::extension!(
    onin_plugin_api,
    ops = [op_invoke, op_console_log],
    options = {
        app_handle: AppHandle,
        plugin_id: String,
    },
    state = |state, options| {
        state.put(options.app_handle);
        state.put(options.plugin_id);
    },
);

pub fn create_runtime(app_handle: &AppHandle) -> Result<JsRuntime, String> {
    create_runtime_with_plugin_id(app_handle, "")
}

pub fn create_runtime_with_plugin_id(
    app_handle: &AppHandle,
    plugin_id: &str,
) -> Result<JsRuntime, String> {
    let ext = onin_plugin_api::init(app_handle.clone(), plugin_id.to_string());

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
    ClearPlugin {
        plugin_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    ClearAllPlugins {
        response: oneshot::Sender<Result<(), String>>,
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
                        PluginTask::ClearPlugin {
                            plugin_id,
                            response,
                        } => {
                            let result = Self::handle_clear_plugin(&mut runtimes, &plugin_id);
                            let _ = response.send(result);
                        }
                        PluginTask::ClearAllPlugins { response } => {
                            let result = Self::handle_clear_all_plugins(&mut runtimes);
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
        // 如果插件已经存在，先清除它（支持热重载）
        if runtimes.contains_key(plugin_id) {
            println!(
                "[Plugin] Clearing existing runtime for plugin: {}",
                plugin_id
            );
            runtimes.remove(plugin_id);
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
        println!("[Plugin] Initialized plugin runtime: {}", plugin_id);

        Ok(())
    }

    fn handle_clear_plugin(
        runtimes: &mut HashMap<String, JsRuntime>,
        plugin_id: &str,
    ) -> Result<(), String> {
        if runtimes.remove(plugin_id).is_some() {
            println!("[Plugin] Cleared runtime for plugin: {}", plugin_id);
        } else {
            println!("[Plugin] No runtime found for plugin: {}", plugin_id);
        }
        Ok(())
    }

    fn handle_clear_all_plugins(runtimes: &mut HashMap<String, JsRuntime>) -> Result<(), String> {
        let count = runtimes.len();
        runtimes.clear();
        println!("[Plugin] Cleared all {} plugin runtimes", count);
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

        // 将 args 序列化为 JSON 字符串
        let args_json = serde_json::to_string(&args).map_err(|e| e.to_string())?;

        // 构造调用代码
        // 注意：第二个参数直接传递 JSON 对象（不用引号包裹）
        let call_code = format!(
            r#"
            (async () => {{
                try {{
                    if (typeof globalThis.__ONIN_COMMAND_HANDLER__ === 'function') {{
                        const result = await globalThis.__ONIN_COMMAND_HANDLER__('{}', {});
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

pub async fn clear_all_plugin_runtimes() -> Result<(), String> {
    let manager = get_plugin_runtime_manager().await?;
    manager.clear_all_plugins().await
}

pub async fn clear_plugin_runtime(plugin_id: &str) -> Result<(), String> {
    let manager = get_plugin_runtime_manager().await?;
    manager.clear_plugin(plugin_id.to_string()).await
}

pub async fn execute_js(
    app_handle: &AppHandle,
    js_code: &str,
    plugin_id: Option<&str>,
) -> Result<(), String> {
    // 使用 plugin_id 创建运行时，这样 OpState 中就有正确的 plugin_id
    let plugin_id_str = plugin_id.unwrap_or("");
    let mut runtime = create_runtime_with_plugin_id(app_handle, plugin_id_str)?;

    // 检测代码是否已经是 IIFE 格式
    let is_iife =
        js_code.trim_start().starts_with("(function") || js_code.trim_start().starts_with("(()");

    // 检测代码是否包含 import 语句
    let has_import =
        js_code.contains("import ") && (js_code.contains("from \"") || js_code.contains("from '"));

    let wrapped_code = if has_import {
        // ES 模块：转换为动态 import
        // 将 ES 模块代码转换为 data URL
        let code_with_id = if let Some(id) = plugin_id {
            format!("globalThis.__PLUGIN_ID__ = '{}';\n{}", id, js_code)
        } else {
            js_code.to_string()
        };

        // 使用 data URL 和动态 import
        let data_url = format!(
            "data:text/javascript;base64,{}",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                code_with_id.as_bytes()
            )
        );

        format!(
            "(async () => {{\n  await import('{}');\n}})().catch(err => console.error('Plugin module error:', err));",
            data_url
        )
    } else if is_iife {
        // 如果已经是 IIFE，直接使用，但注入 plugin_id
        if let Some(id) = plugin_id {
            format!("globalThis.__PLUGIN_ID__ = '{}';\n{}", id, js_code)
        } else {
            js_code.to_string()
        }
    } else {
        // 否则包装为异步 IIFE，支持顶层 await
        if let Some(id) = plugin_id {
            format!(
                "(async () => {{\n  globalThis.__PLUGIN_ID__ = '{}';\n{}\n}})().catch(err => console.error('Plugin error:', err));",
                id, js_code
            )
        } else {
            format!(
                "(async () => {{\n{}\n}})().catch(err => console.error('Plugin error:', err));",
                js_code
            )
        }
    };

    // 执行插件代码
    let result = runtime.execute_script("<plugin>", wrapped_code);
    match result {
        Ok(_) => runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
