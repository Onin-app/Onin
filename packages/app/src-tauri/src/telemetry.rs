use std::sync::Arc;

/// 过滤 tauri-plugin-updater 上报的"预期失败"噪音。
///
/// 该插件在更新端点返回非 2xx（如仓库暂无可用 release、网络被代理拦截等）或
/// 网络请求失败（如网络离线、DNS 解析失败、连接超时或境内直连 GitHub 失败等）时，
/// 会以 error 级别记录日志（如 "update endpoint did not respond..."
/// 或 "failed to check for updates: ..."）。
/// 这类失败是可预期的运行环境与网络问题，并非应用代码 bug，
/// 若不过滤会持续刷爆 GlitchTip/Sentry。这里丢弃 updater 检查更新阶段的
/// 网络与端点噪音，其余 updater 错误（如安装失败、验签失败等）仍会上报。
///
/// 注：tauri-plugin-updater 内部使用 `log::error!`，当由 `tracing-log`
/// 桥接至 `sentry-tracing` 时，`event.logger` 会被标记为 `"log"`，真实的模块名
/// 记录在 `Rust Tracing Fields` 上下文的 `log.target` / `log.module_path` 中。
/// 因此需要同时匹配直接 logger、tracing 上下文字段及通用 log 来源。
fn is_updater_source(event: &sentry::protocol::Event<'static>) -> bool {
    let direct_logger = event
        .logger
        .as_deref()
        .map(|logger| logger.starts_with("tauri_plugin_updater") || logger == "log")
        .unwrap_or(false);
    if direct_logger {
        return true;
    }

    if let Some(sentry::protocol::Context::Other(map)) = event.contexts.get("Rust Tracing Fields") {
        let is_updater_target = map
            .get("log.target")
            .or_else(|| map.get("log.module_path"))
            .and_then(|val| val.as_str())
            .map(|target| target.starts_with("tauri_plugin_updater"))
            .unwrap_or(false);
        if is_updater_target {
            return true;
        }
    }

    event.logger.is_none()
}

fn filter_updater_noise(
    event: sentry::protocol::Event<'static>,
) -> Option<sentry::protocol::Event<'static>> {
    let message = event
        .message
        .as_deref()
        .or_else(|| event.logentry.as_ref().map(|entry| entry.message.as_str()))
        .unwrap_or_default();
    let is_updater_check_noise = message.contains("update endpoint did not respond")
        || message.contains("failed to check for updates");

    if is_updater_check_noise && is_updater_source(&event) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_filter_updater_noise_direct_logger() {
        let event = sentry::protocol::Event {
            logger: Some("tauri_plugin_updater::updater".into()),
            message: Some("update endpoint did not respond with a successful status code".into()),
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_none());
    }

    #[test]
    fn test_filter_updater_noise_tracing_log_target() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "log.target".to_string(),
            "tauri_plugin_updater::updater".into(),
        );
        fields.insert(
            "log.module_path".to_string(),
            "tauri_plugin_updater::updater".into(),
        );

        let mut contexts = BTreeMap::new();
        contexts.insert(
            "Rust Tracing Fields".to_string(),
            sentry::protocol::Context::Other(fields),
        );

        let event = sentry::protocol::Event {
            logger: Some("log".into()),
            message: Some("update endpoint did not respond with a successful status code".into()),
            contexts,
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_none());
    }

    #[test]
    fn test_filter_updater_noise_without_logger() {
        let event = sentry::protocol::Event {
            logger: None,
            message: Some("update endpoint did not respond with a successful status code".into()),
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_none());
    }

    #[test]
    fn test_filter_updater_network_check_noise() {
        let event = sentry::protocol::Event {
            logger: Some("tauri_plugin_updater::updater".into()),
            message: Some(
                "failed to check for updates: error sending request for url (https://github.com/Onin-app/Onin/releases/latest/download/latest.json)".into(),
            ),
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_none());
    }

    #[test]
    fn test_keep_other_updater_errors() {
        let event = sentry::protocol::Event {
            logger: Some("tauri_plugin_updater::updater".into()),
            message: Some("failed to install update: signature verification failed".into()),
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_some());
    }

    #[test]
    fn test_keep_unrelated_errors() {
        let event = sentry::protocol::Event {
            logger: Some("app::module".into()),
            message: Some("network error".into()),
            ..Default::default()
        };
        assert!(filter_updater_noise(event).is_some());
    }

    #[test]
    fn test_filter_tao_event_loop_race() {
        let event = sentry::protocol::Event {
            exception: vec![sentry::protocol::Exception {
                ty: "panic".into(),
                value: Some("assertion failed: flush_paint_messages(None, ...)".into()),
                ..Default::default()
            }]
            .into(),
            ..Default::default()
        };
        assert!(filter_tao_event_loop_race(event).is_none());
    }
}
