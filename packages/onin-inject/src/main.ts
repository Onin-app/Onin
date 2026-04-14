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
  // 1. 基础环境初始化
  resolvePluginId();
  initRuntime();

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

  // 2. 暴露全局 Toast API (兼容旧版)
  const showToast = function (payload: ToastPayload) {
    ensureMounted();
    if (toastInstance) {
      (toastInstance as unknown as { show: (p: ToastPayload) => void }).show(
        payload,
      );
    }
  };

  window.__ONIN_SHOW_TOAST__ = showToast;

  // 3. 事件注册中心
  const listeners: Record<string, Array<() => void>> = {
    show: [],
    hide: [],
    focus: [],
    blur: [],
  };

  window.addEventListener("message", (event) => {
    const data = event.data;
    if (data?.type === "plugin-lifecycle-event" && data.event) {
      const callbacks = listeners[data.event];
      if (callbacks) {
        callbacks.forEach((cb) => {
          try {
            cb();
          } catch (e) {
            console.error(`[Onin SDK] Error in ${data.event} listener:`, e);
          }
        });
      }
    }
  });

  // 4. 暴露现代 Bridge API
  window.__ONIN_BRIDGE__ = {
    version: "0.1.0",
    showToast,
    postMessage: (message: any) => {
      window.postMessage(message, "*");
    },
    onShow: (cb) => listeners.show.push(cb),
    onHide: (cb) => listeners.hide.push(cb),
    onFocus: (cb) => listeners.focus.push(cb),
    onBlur: (cb) => listeners.blur.push(cb),
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => ensureMounted());
  } else {
    ensureMounted();
  }
}

/**
 * 解析插件 ID
 * 优先级: 1. URL 参数 2. Rust 预设的全局变量 3. 默认值
 */
function resolvePluginId() {
  const urlParams = new URLSearchParams(window.location.search);
  const pluginIdFromUrl = urlParams.get("plugin_id");

  if (pluginIdFromUrl) {
    window.__PLUGIN_ID__ = pluginIdFromUrl;
  }

  if (!window.__PLUGIN_ID__) {
    // 兜底值，不应出现
    window.__PLUGIN_ID__ = "unknown";
  }

  // 确保 globalThis 也有映射
  (globalThis as any).__PLUGIN_ID__ = window.__PLUGIN_ID__;
}

/**
 * 初始化运行时元数据
 */
function initRuntime() {
  if (!window.__ONIN_RUNTIME__) {
    const urlParams = new URLSearchParams(window.location.search);
    window.__ONIN_RUNTIME__ = {
      mode: (urlParams.get("mode") as any) || "inline",
      pluginId: window.__PLUGIN_ID__ || "unknown",
      version: "dev-fallback",
      mainWindowLabel: "main",
    };
  }
}
