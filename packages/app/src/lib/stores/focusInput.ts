import { writable } from "svelte/store";

export const focusInputTrigger = writable<number>(0);

export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
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
export function requestInputFocusWithRetry(maxRetries = 15, intervalMs = 50) {
  let retries = 0;

  const attemptFocus = () => {
    // 强制触发 Svelte 绑定去执行 searchInputRef.focus()
    // 不管 document.hasFocus() 是不是 false，先狂点 focus() 再说

    // Attempt DOM Window focus first
    if (typeof window !== "undefined") {
      window.focus();
    }

    requestInputFocus();

    // 如果焦点已经进入 Webview，或者超时，则停止轮询
    if (document.hasFocus() || retries >= maxRetries) {
      return;
    }

    retries++;
    setTimeout(attemptFocus, intervalMs);
  };

  attemptFocus();
}
