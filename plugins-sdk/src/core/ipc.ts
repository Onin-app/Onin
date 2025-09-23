import { getEnvironment, RuntimeEnvironment } from './environment';
import { type EventName, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event';

// --- invoke ---

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
    // Headless 环境的 invoke 实现直接在这里定义
    invokeFn = async <T>(method: string, arg: any): Promise<T> => {
      // @ts-ignore: Deno.core is injected by the Deno runtime in Rust.
      const result = await Deno.core.ops.op_invoke(method, arg);

      // 处理 InvokeResult 枚举
      if (result && typeof result === 'object') {
        if (result.type === 'error') {
          throw new Error(result.error);
        } else if (result.type === 'ok') {
          return result.value as T;
        }
      }

      // 兼容旧格式
      if (result && typeof result === 'object' && 'error' in result) {
        throw new Error(result.error);
      }

      return result as T;
    };
  } else {
    throw new Error('Unsupported runtime environment for invoke');
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


// --- listen ---

// 用来缓存导入的 listen 函数
let listenFn: ((event: EventName, handler: EventCallback<any>) => Promise<UnlistenFn>) | null = null;

// 异步加载并缓存 listen 函数
async function loadListen() {
  if (listenFn) {
    return listenFn;
  }

  const environment = getEnvironment();
  if (environment === RuntimeEnvironment.Webview) {
    const { listen } = await import('@tauri-apps/api/event');
    listenFn = listen;
  } else if (environment === RuntimeEnvironment.Headless) {
    // Headless 环境通过挂载全局变量来模拟对特定事件的“监听”
    listenFn = (event: EventName, handler: EventCallback<any>): Promise<UnlistenFn> => {
      if (event === 'plugin_command_execute') {
        // 这是为 registerCommandHandler 特别处理的逻辑
        (globalThis as any).__BAIZE_COMMAND_HANDLER__ = handler;
        // Headless 模式下没有 unlisten 的概念，返回一个空函数
        return Promise.resolve(() => {});
      }
      
      console.warn(`Event listening for '${event.toString()}' is not supported in headless mode.`);
      return Promise.resolve(() => {}); // 返回一个空的 unlisten 函数
    };
  } else {
    throw new Error('Unsupported runtime environment for listen');
  }
  return listenFn;
}

// 导出的 listen 函数
export async function listen<T>(event: EventName, handler: EventCallback<T>): Promise<UnlistenFn> {
  const fn = await loadListen();
  if (!fn) {
    throw new Error('Listen function not loaded');
  }
  return fn(event, handler);
}