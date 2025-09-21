import { getEnvironment, RuntimeEnvironment } from './environment';

// 用来缓存导入的 invoke 函数
let invokeFn: ((cmd: string, args?: any) => Promise<any>) | null = null;

// 异步加载并缓存 invoke 函数
async function loadInvoke() {
  if (invokeFn) {
    return invokeFn;
  }

  const environment = getEnvironment();
  if (environment === RuntimeEnvironment.Webview) {
    const { invoke } = await import('@tauri-apps/api/core');
    invokeFn = invoke;
  } else if (environment === RuntimeEnvironment.Headless) {
    const { invoke } = await import('../adapters/headless');
    invokeFn = invoke;
  } else {
    throw new Error('Unsupported runtime environment');
  }
  return invokeFn;
}

// 导出的 invoke 函数
export async function invoke<T>(method: string, arg: any): Promise<T> {
  const fn = await loadInvoke();
  if (!fn) {
    throw new Error('Invoke function not loaded');
  }
  return fn(method, arg);
}