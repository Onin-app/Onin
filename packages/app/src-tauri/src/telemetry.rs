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
