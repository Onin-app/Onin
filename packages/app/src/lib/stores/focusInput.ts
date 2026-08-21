import { writable } from "svelte/store";

export const focusInputTrigger = writable<number>(0);
export const focusExtensionInputTrigger = writable<number>(0);

export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
}

export function requestExtensionInputFocus() {
  focusExtensionInputTrigger.update((n) => n + 1);
}

// 持有当前的 timer 引用以支持随时取消
let activeFocusTimeout: any = null;
let visibilityCleanup: (() => void) | null = null;

/**
 * 掐断当前的聚焦重试
 */
export function cancelInputFocusRetry() {
  if (activeFocusTimeout) {
    clearTimeout(activeFocusTimeout);
    activeFocusTimeout = null;
  }
  if (visibilityCleanup) {
    visibilityCleanup();
    visibilityCleanup = null;
  }
}

/**
 * 聚焦输入框（轻量重试版）。
 *
 * 焦点模型（事件驱动）：
 * - OS 前台夺取完全由 Rust 层负责（`request_show` → 确定性激活配方
 *   `SendInput(ALT) + AttachThreadInput + SetForegroundWindow` → 后台定时验证重试），
 *   前端不再轮询 `document.hasFocus()`，也不再调用 `force_focus` 抢救。
 * - 前端只做 DOM 层面聚焦：输入框可能在事件到达时尚未挂载（WebView2 渲染/
 *   Svelte 异步时序），因此做少量重试；窗口隐藏期间挂起，待可见时立即重试。
 */
export function requestInputFocusWithRetry(maxRetries = 5, intervalMs = 60) {
  cancelInputFocusRetry();

  let retries = 0;
  let clickAttempted = false; // click() 只做一次，避免反复触发输入框点击动画

  const attemptFocus = () => {
    // 窗口隐藏：挂起重试，等 visibilitychange 后立即恢复
    if (typeof document !== "undefined" && document.hidden) {
      if (!visibilityCleanup) {
        const handleVisibilityChange = () => {
          if (!document.hidden) {
            if (visibilityCleanup) {
              visibilityCleanup();
              visibilityCleanup = null;
            }
            requestInputFocusWithRetry(maxRetries, intervalMs);
          }
        };
        document.addEventListener("visibilitychange", handleVisibilityChange);
        visibilityCleanup = () => {
          document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange,
          );
        };
      }

      if (retries >= maxRetries) {
        activeFocusTimeout = null;
        return;
      }
      retries++;
      activeFocusTimeout = setTimeout(attemptFocus, intervalMs);
      return;
    }

    let el =
      typeof document !== "undefined"
        ? (document.getElementById(
            "main-search-input",
          ) as HTMLInputElement | null)
        : null;
    if (!el) {
      el =
        typeof document !== "undefined"
          ? (document.getElementById(
              "extension-search-input",
            ) as HTMLInputElement | null)
          : null;
    }

    if (el) {
      window.focus();
      el.focus();
      if (!clickAttempted) {
        clickAttempted = true;
        el.click();
      }
    }

    const activeEl =
      typeof document !== "undefined" ? document.activeElement : null;
    const isInputFocused = el !== null && activeEl === el;

    // 输入框已获得 DOM 焦点即成功（系统焦点由 Rust 层保证）
    if (isInputFocused || retries >= maxRetries) {
      activeFocusTimeout = null;
      if (visibilityCleanup) {
        visibilityCleanup();
        visibilityCleanup = null;
      }
      return;
    }

    retries++;
    activeFocusTimeout = setTimeout(attemptFocus, intervalMs);
  };

  attemptFocus();
}
