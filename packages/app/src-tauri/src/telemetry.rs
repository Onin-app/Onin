use std::sync::Arc;

/// 过滤 tauri-plugin-updater 上报的"预期失败"噪音。
///
/// 该插件在更新端点返回非 2xx（如仓库暂无可用 release、网络被代理拦截等）时，
/// 会固定以 error 级别打一条日志（updater.rs:443 的
/// "update endpoint did not respond with a successful status code"）。
/// 这类失败是可预期的运行环境问题，并非应用 bug，每次检查更新都会触发，
/// 若不过滤会持续刷爆 GlitchTip/Sentry。这里只丢弃 updater 模块中
/// 该特定消息的错误事件，其余 updater 错误（如下载失败）仍会上报。
fn filter_updater_noise(
    event: sentry::protocol::Event<'static>,
) -> Option<sentry::protocol::Event<'static>> {
    let is_updater_log = event
        .logger
        .as_deref()
        .map(|logger| logger.starts_with("tauri_plugin_updater"))
        .unwrap_or(false);
    let message = event
        .message
        .as_deref()
        .or_else(|| event.logentry.as_ref().map(|entry| entry.message.as_str()))
        .unwrap_or_default();
    let is_endpoint_status_noise = message.contains("update endpoint did not respond");

    if is_updater_log && is_endpoint_status_noise {
        return None;
    }
    Some(event)
}

/// 过滤 tao（Tauri 的窗口事件循环库）在 Windows 上偶发的重入断言崩溃。
///
/// tao 的 `thread_event_target_callback` 在收到线程事件目标窗口的
/// `WM_PAINT` 时会执行 `assert!(flush_paint_messages(None, ...))`；
/// 当事件循环恰好处于"正在处理重绘事件"（HandlingRedrawEvents）阶段时，
/// 该断言会失败并 panic。这是 tao 内部的时序竞态，常见于窗口快速创建/销毁、
/// 嵌套消息循环（如系统菜单/对话框）或退出流程，应用侧无法规避。
/// 该 panic 会被 tao 捕获后在主循环重新抛出导致进程退出，
/// 这里将其从 GlitchTip 上报中剔除，避免噪音；
/// `lib.rs::run` 中的 `catch_unwind` 会将其兜底为优雅退出。
fn filter_tao_event_loop_race(
    event: sentry::protocol::Event<'static>,
) -> Option<sentry::protocol::Event<'static>> {
    let is_tao_race_panic = event.exception.values.iter().any(|exc| {
        exc.ty == "panic"
            && exc
                .value
                .as_deref()
                .map(|value| value.contains("flush_paint_messages"))
                .unwrap_or(false)
    });
    if is_tao_race_panic {
        return None;
    }
    Some(event)
}

pub fn init_glitchtip() -> Option<sentry::ClientInitGuard> {
    let dsn = option_env!("GLITCHTIP_DSN_NATIVE")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| option_env!("VITE_GLITCHTIP_DSN").filter(|value| !value.trim().is_empty()))?;

    Some(sentry::init((
        dsn,
        sentry::ClientOptions {
            release: Some(format!("onin@{}", env!("CARGO_PKG_VERSION")).into()),
            environment: option_env!("GLITCHTIP_ENVIRONMENT")
                .or(option_env!("VITE_GLITCHTIP_ENVIRONMENT"))
                .filter(|value| !value.trim().is_empty())
                .map(Into::into),
            attach_stacktrace: true,
            send_default_pii: false,
            before_send: Some(Arc::new(|event| {
                filter_updater_noise(event).and_then(filter_tao_event_loop_race)
            })),
            ..Default::default()
        },
    )))
    .map(|guard| {
        sentry::configure_scope(|scope| {
            scope.set_tag("layer", "rust");
        });
        guard
    })
}
