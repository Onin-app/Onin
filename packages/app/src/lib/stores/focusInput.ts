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
let forceFocusFallbackUsed = false;

/**
 * 掐断当前的聚焦重试轮询
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
  forceFocusFallbackUsed = false;
}

/**
 * 【Windows WebView2 焦点抢夺补偿机制】
 *
 * 为什么需要轮询重试？
 * 1. 异步窗口渲染：当后端 Tauri 刚完成强制夺权 (AttachThreadInput) 并显示窗口时，
 *    内部的 WebView2 渲染层级可能还有几十到几百毫秒的呈现延迟。如果刚弹出就只执行一次 `window.focus()`，
 *    极容易落空（document.hasFocus() 此时返回 false）。
 *
 * 2. 一次性预聚焦与探测机制：在 500ms 重试周期中，我们仅在第 0 次（第一下）对 DOM input 进行一次 focus() 和 click()（Pre-focus），
 *    随后不再对其重复写入，以避免重绘光标动画；剩余的重试探测仅负责 window.focus() 系统激活与等待 document.hasFocus() 同步。
 */
export function requestInputFocusWithRetry(maxRetries = 10, intervalMs = 50) {
  cancelInputFocusRetry();

  let retries = 0;
  let preFocusAttempted = false; // 记录在 hasFocus === false 期间是否已尝试过一次预聚焦

  const attemptFocus = () => {
    // 如果窗口已被隐藏，在未达到最大重试次数前，可能只是因为窗口刚被唤起，
    // 可见性状态还没在浏览器中同步。因此我们绑定 visibilitychange 监听器以在可见时立即重试。
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

    const hasFocus = typeof document !== "undefined" && document.hasFocus();

    // 尝试直接聚焦 input DOM 元素（绕过 Svelte 异步响应时序）
    if (typeof document !== "undefined") {
      let el = document.getElementById(
        "main-search-input",
      ) as HTMLInputElement | null;
      if (!el) {
        el = document.getElementById(
          "extension-search-input",
        ) as HTMLInputElement | null;
      }

      if (el) {
        const activeEl = document.activeElement;
        const isCurrentlyActive = activeEl === el;

        if (!isCurrentlyActive) {
          if (hasFocus) {
            // 系统已获焦，但输入框尚未激活：执行强力聚焦兜底，确保百分百可用
            window.focus();
            el.focus();
            el.click();
          } else if (!preFocusAttempted) {
            // 系统未获焦且未做过预聚焦：执行唯一一次温和的预置聚焦，并在后续探测中跳过
            preFocusAttempted = true;
            window.focus();
            el.focus();
            el.click();
          }
        }
      }

      if (!hasFocus) {
        window.focus();
      }
    }

    const activeEl =
      typeof document !== "undefined" ? document.activeElement : null;
    let targetEl: HTMLElement | null = null;
    if (typeof document !== "undefined") {
      targetEl =
        document.getElementById("main-search-input") ||
        document.getElementById("extension-search-input");
    }
    const isInputFocused = activeEl && targetEl && activeEl === targetEl;

    // 成功条件：输入框已激活（活跃元素），并且窗口有系统焦点。
    if (isInputFocused && hasFocus) {
      activeFocusTimeout = null;
      if (visibilityCleanup) {
        visibilityCleanup();
        visibilityCleanup = null;
      }
      return;
    }

    // 重试次数用尽
    if (retries >= maxRetries) {
      // 窗口可见但无系统焦点：做一次 Rust 层的 force_focus 抢救
      if (!forceFocusFallbackUsed && !document.hidden) {
        forceFocusFallbackUsed = true;
        activeFocusTimeout = null;
        if (visibilityCleanup) {
          visibilityCleanup();
          visibilityCleanup = null;
        }

        import("@tauri-apps/api/core").then(({ invoke }) => {
          invoke("force_focus").catch((err) =>
            console.error("[FocusEngine] force_focus failed:", err),
          );
        });

        setTimeout(() => {
          forceFocusFallbackUsed = false;
          requestInputFocusWithRetry(maxRetries, intervalMs);
        }, 80);

        return;
      }

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
