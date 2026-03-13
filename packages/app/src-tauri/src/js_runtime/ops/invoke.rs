//! Invoke 操作模块
//!
//! 处理插件对 Tauri API 的调用请求

use deno_core::op2;
use deno_core::OpState;
use std::cell::RefCell;
use std::rc::Rc;
use tauri::AppHandle;

use crate::js_runtime::handlers;
use crate::js_runtime::types::InvokeResult;

/// 异步 op：处理插件 API 调用
///
/// 这是插件调用所有 Tauri API 的统一入口点。
/// 根据 method 参数将请求分发到对应的处理器。
///
/// 注意：异步 op 需要使用 Rc<RefCell<OpState>> 来避免生命周期问题
#[op2(async)]
#[serde]
pub async fn op_invoke(
    state: Rc<RefCell<OpState>>,
    #[string] method: String,
    #[serde] arg: serde_json::Value,
) -> InvokeResult {

    // 在进入 async 之前提取所需数据，避免生命周期问题
    let (app_handle, plugin_id) = {
        let borrowed = state.borrow();
        (
            borrowed.borrow::<AppHandle>().clone(),
            borrowed.borrow::<String>().clone(),
        )
    };

    // 设置当前插件ID到线程本地存储 (使用 RAII Guard 确保即使 panic 也能清理)
    let _guard = crate::plugin::context::PluginContextGuard::new(plugin_id.clone());

    // 调用处理器分发
    let result = handlers::dispatch(&method, app_handle, plugin_id, arg).await;

    result
}

