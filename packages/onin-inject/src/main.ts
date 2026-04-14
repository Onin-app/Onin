/**
 * Onin Plugin Runtime Injection Layer
 *
 * 编译为 IIFE 后通过 Rust include_str! 注入到插件 Webview 中。
 * 使用 svelte-sonner 提供与主程序一致的 Toast 通知。
 */

import { mount } from "svelte";
import Toast from "./components/Toast.svelte";
import type { ToastPayload } from "./types";

// 避免重复注入
if (!window.__ONIN_TOAST_INJECTED__) {
  window.__ONIN_TOAST_INJECTED__ = true;
  init();
}

function init() {
  let toastInstance: ReturnType<typeof mountToast> | null = null;

  function mountToast() {
    const host = document.createElement("div");
    host.id = "onin-inject-root";
    document.body.appendChild(host);
    return mount(Toast, { target: host });
  }

  function ensureMounted() {
    if (!toastInstance && document.body) {
      toastInstance = mountToast();
    }
  }

  // 暴露全局 API，供 Rust eval() 调用
  window.__ONIN_SHOW_TOAST__ = function (payload: ToastPayload) {
    ensureMounted();
    if (toastInstance) {
      (toastInstance as unknown as { show: (p: ToastPayload) => void }).show(payload);
    }
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => ensureMounted());
  } else {
    ensureMounted();
  }
}
