use deno_core::op2;
use deno_core::{JsRuntime, PollEventLoopOptions, RuntimeOptions};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
enum InvokeResult {
    Ok { id: String, name: String },
    Err { error: String },
}

// 定义扩展
deno_core::extension!(plugin_api, ops = [op_invoke],);

// 异步 op
#[op2(async)]
#[serde] // 整个返回类型用 serde
async fn op_invoke(#[string] method: String, #[string] arg: String) -> InvokeResult {
    println!("插件异步调用 invoke: method={}, arg={}", method, arg);

    // 模拟异步操作，比如调用数据库或网络
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    match method.as_str() {
        "get_user" => InvokeResult::Ok {
            id: arg,
            name: "Alice".to_string(),
        },
        _ => InvokeResult::Err {
            error: "unknown method".to_string(),
        },
    }
}

pub fn execute_js(js_code: &str) -> Result<(), String> {
    let ext = plugin_api::init();

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    let tokio_runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let js_code_owned = js_code.to_string();

    tokio_runtime.block_on(async {
        let result = runtime.execute_script("<plugin>", js_code_owned);
        match result {
            Ok(_) => runtime
                .run_event_loop(PollEventLoopOptions::default())
                .await
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    })
}
