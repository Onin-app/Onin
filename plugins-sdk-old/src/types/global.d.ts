// Deno 全局类型声明
declare global {
  namespace globalThis {
    var Deno: {
      core: {
        ops: {
          op_invoke: (method: string, args: unknown) => Promise<unknown>;
        };
      };
    } | undefined;
  }

  interface Window {
    __TAURI__: unknown;
  }
}

export {};