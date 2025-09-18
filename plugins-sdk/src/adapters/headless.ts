/**
 * @module adapters/headless
 * @description Headless 环境（Deno）的 API 适配器。
 */

/**
 * 调用一个由 Rust 后端暴露的 Tauri 命令。
 * 这是 headless 插件与核心后端通信的主要方式。
 * @param method - 要调用的方法名称。
 * @param arg - 传递给方法的参数，必须是可序列化的。
 * @returns 一个 Promise，解析为命令执行的结果。
 * @throws 如果后端返回错误，则会抛出异常。
 */
export async function invoke<T>(method: string, arg: any): Promise<T> {
  try {
    // @ts-ignore: Deno.core is injected by the Deno runtime in Rust.
    const result = await Deno.core.ops.op_invoke(method, arg);

    // The Rust backend returns an enum InvokeResult { Ok(T), Err { error: String } }
    // We need to handle this structure.
    if (result && typeof result === 'object') {
      if ('error' in result) {
        // This is the Err variant
        throw new Error(result.error || 'Unknown error from op_invoke');
      } else {
        // This is the Ok variant, which directly contains the data
        return result as T;
      }
    } else if (result === null || result === undefined) {
        // Handle cases where the Ok variant contains `()` in Rust, resulting in `null`.
        return result as T;
    } else {
        throw new Error('Invalid response format from op_invoke');
    }
  } catch (e) {
    // Re-throw the error to be caught by the caller
    throw e;
  }
}


/**
 * 在 Headless 环境中显示通知。
 * 
 * @param options 通知选项。
 * @param options.title 通知的标题。
 * @param options.body 通知的正文。
 * @returns {Promise<void>} 调用完成时解析的 Promise。
 */
export async function showNotification(options: { title: string; body: string }): Promise<void> {
  await invoke<void>("show_notification", options);
}

/**
 * 在 headless 插件中注册一个指令处理器。
 * 当后端需要执行该插件的指令时，这个注册的函数将被调用。
 * @param handler - 一个接收指令代码和参数的函数。它应该返回一个可序列化的结果，
 *                  或者一个解析为可序列化结果的 Promise。
 */
export function registerCommandHandler(
  handler: (command: string, args: any) => any | Promise<any>
): void {
  /**
   * 将用户的处理器挂载到一个全局的、可预测的变量上。
   * Rust 后端将通过执行一段 JS 代码来调用这个函数。
   * @see src-tauri/src/plugin_manager.rs
   */
  (globalThis as any).__BAIZE_COMMAND_HANDLER__ = handler;
}