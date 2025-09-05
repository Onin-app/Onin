use deno_core::op2;
use deno_core::{JsRuntime, OpState, PollEventLoopOptions, RuntimeOptions};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use tauri::AppHandle;

use crate::plugin_api;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum InvokeResult {
    Ok(serde_json::Value),
    Err { error: String },
}

// 定义扩展
deno_core::extension!(
    baize_plugin_api,
    ops = [op_invoke],
    options = {
        app_handle: AppHandle,
    },
    state = |state, options| {
        state.put(options.app_handle);
    },
);

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
                        Ok(_) => InvokeResult::Ok(serde_json::Value::Null),
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
        _ => InvokeResult::Err {
            error: "unknown method".to_string(),
        },
    }
}

pub async fn execute_js(app_handle: &AppHandle, js_code: &str) -> Result<(), String> {
    let ext = baize_plugin_api::init(app_handle.clone());

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });
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
