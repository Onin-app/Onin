import { writable } from "svelte/store";

export const focusInputTrigger = writable<number>(0);

export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
}

// 持有当前的 timer 引用以支持随时取消
let activeFocusTimeout: any = null;

/**
 * 掐断当前的聚焦重试轮询
 */
export function cancelInputFocusRetry() {
  if (activeFocusTimeout) {
    clearTimeout(activeFocusTimeout);
    activeFocusTimeout = null;
  }
}

/**
 * 【Windows WebView2 焦点抢夺补偿机制】
 *
 * 为什么需要轮询重试？
 * 1. 异步窗口渲染：当后端 Tauri 刚完成强制夺权 (AttachThreadInput) 并显示窗口时，
 *    内部的 WebView2 渲染层级可能还有几十到几百毫秒的呈现延迟。如果刚弹出就只执行一次 `window.focus()`，
 *    极容易落空（document.hasFocus() 此时返回 false）。
 * 2. 初次启动盲点：Tauri 应用首次启动时，主窗口默认是 visible 状态，因此不会触发后端的 `window_visibility` 事件钩子，
 *    如果不在此通过 onMount 主动轮询争取一次，应用首次冷启动后大概率将处于游离失焦状态。
 *
 * 此处使用高频短时间的重定向探测（例如 15次 x 50ms=750ms），以“无论如何也要保证光标死死咬紧窗口”的策略弥补平台底层的时序间隙。
 */
export function requestInputFocusWithRetry(maxRetries = 3, intervalMs = 150) {
  // 启动前先清除上一次未完成的轮询，防止多任务重叠
  cancelInputFocusRetry();

  let retries = 0;

  const attemptFocus = () => {
    // 核心安全防御：如果窗口已被隐藏，立刻静默退出，切勿在无效 HWND 上调用 SetFocus 触发闪退
    if (typeof document !== "undefined" && document.hidden) {
      activeFocusTimeout = null;
      return;
    }

    // 尝试直接聚焦 input DOM 元素（绕过 Svelte 异步响应时序）
    if (typeof document !== "undefined") {
      const el = document.getElementById(
        "main-search-input",
      ) as HTMLInputElement | null;
      if (el) {
        el.focus();
      }
    }

    const activeEl =
      typeof document !== "undefined" ? document.activeElement : null;
    const isInputFocused =
      activeEl &&
      (activeEl.tagName === "INPUT" || activeEl.tagName === "TEXTAREA");

    // 如果焦点已经进入 Webview 内部的输入框，或者超时，则停止轮询
    if (isInputFocused || retries >= maxRetries) {
      activeFocusTimeout = null;
      return;
    }

    retries++;
    activeFocusTimeout = setTimeout(attemptFocus, intervalMs);
  };

  attemptFocus();
}
