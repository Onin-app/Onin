/**
 * @module adapters/headless
 * @description Headless 环境（Deno）的 API 适配器。
 */

/**
 * 在 Headless 环境中显示通知。
 * 
 * @param options 通知选项。
 * @param options.title 通知的标题。
 * @param options.body 通知的正文。
 * @returns {Promise<any>} 调用 op_invoke 的结果。
 */
export async function showNotification(options: { title: string; body: string }): Promise<any> {
  try {
    // @ts-ignore
    const result = await Deno.core.ops.op_invoke(
      "show_notification",
      options
    );

    // 处理 InvokeResult 枚举格式
    if (result && typeof result === 'object') {
      // 检查是否是 Ok 变体 (直接包含数据)
      if (!('error' in result)) {
        return result;
      } else {
        // 检查是否是 Err 变体 (包含 error 字段)
        throw new Error(result.error || 'Unknown error from op_invoke');
      }
    } else {
      throw new Error('op_invoke returned null or undefined');
    }
  } catch (e) {
    throw e;
  }
}