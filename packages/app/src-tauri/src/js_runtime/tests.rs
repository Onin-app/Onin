use deno_core::{JsRuntime, PollEventLoopOptions, RuntimeOptions};

const REGISTER_UNLOAD_SCRIPT: &str = r#"
  globalThis.__ONIN_EXECUTE_UNLOAD_CALLBACKS__ = async () => {
    globalThis.__unloadCount = (globalThis.__unloadCount ?? 0) + 1;
  };
"#;

const EXECUTE_UNLOAD_SCRIPT: &str = r#"
  (async () => {
    if (typeof globalThis.__ONIN_EXECUTE_UNLOAD_CALLBACKS__ === 'function') {
      await globalThis.__ONIN_EXECUTE_UNLOAD_CALLBACKS__();
    }
  })();
"#;

#[tokio::test]
async fn unload_executor_runs_registered_callback() {
    let mut runtime = JsRuntime::new(RuntimeOptions::default());

    runtime
        .execute_script("<register_unload>", REGISTER_UNLOAD_SCRIPT.to_string())
        .expect("register unload hook");
    runtime
        .run_event_loop(PollEventLoopOptions::default())
        .await
        .expect("flush register script");

    runtime
        .execute_script("<execute_unload>", EXECUTE_UNLOAD_SCRIPT.to_string())
        .expect("execute unload hook");
    runtime
        .run_event_loop(PollEventLoopOptions::default())
        .await
        .expect("flush unload script");

    let value = runtime
        .execute_script(
            "<read_unload_count>",
            "globalThis.__unloadCount ?? 0".to_string(),
        )
        .expect("read unload count");

    let scope = &mut runtime.handle_scope();
    let local = value.open(scope);
    let unload_count = local
        .integer_value(scope)
        .expect("unload count should be an integer");

    assert_eq!(unload_count, 1);
}
