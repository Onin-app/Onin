import { getEnvironment, RuntimeEnvironment } from './environment';

/**
 * 定义不同环境下的处理器函数签名。
 */
interface Handlers<T> {
  webview: () => T;
  headless: () => T;
}

/**
 * 根据当前运行环境，自动选择并执行相应的处理器函数。
 * 
 * @param handlers 一个包含 webview 和 headless 两个环境下实现的对象。
 * @returns 返回所选处理器函数的执行结果。
 * @throws 如果环境不是 webview 或 headless，则抛出错误。
 */
export function dispatch<T>(handlers: Handlers<T>): T {
  const environment = getEnvironment();

  if (environment === RuntimeEnvironment.Webview) {
    return handlers.webview();
  }

  if (environment === RuntimeEnvironment.Headless) {
    return handlers.headless();
  }

  throw new Error(`Unsupported environment: ${environment}`);
}