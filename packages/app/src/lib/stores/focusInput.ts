import { writable } from "svelte/store";

export const focusInputTrigger = writable<number>(0);

export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
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
 * 2. 初次启动盲点：Tauri 应用首次启动时，主窗口默认是 visible 状态，因此不会触发后端的 `window_visibility` 事件钩子，
 *    如果不在此通过 onMount 主动轮询争取一次，应用首次冷启动后大概率将处于游离失焦状态。
 *
 * 此处使用高频短时间的重定向探测（例如 10次 x 50ms = 500ms），结合 visibilitychange 黄金时序事件，
 * 以"无论如何也要保证光标死死咬紧窗口"的策略弥补平台底层的时序间隙。
 *
 * 当重试耗尽但窗口已可见且仍未获得系统焦点时（hasFocus=false），通过 invoke("force_focus")
 * 再次从 Rust 层强抢前台，然后重新启动一轮 retry（最多一次 fallback）。
 */
export function requestInputFocusWithRetry(maxRetries = 10, intervalMs = 50) {
  // 启动前先清除上一次未完成的轮询，防止多任务重叠
  cancelInputFocusRetry();

  let retries = 0;

  const attemptFocus = () => {
    // 核心安全防御：如果窗口已被隐藏，在未达到最大重试次数前，可能只是因为窗口刚被唤起，
    // 可见性状态还没在浏览器中同步。因此我们绑定 visibilitychange 监听器以在可见时立即重试，并继续定时器探测。
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

    // 尝试直接聚焦 input DOM 元素（绕过 Svelte 异步响应时序）
    // 先调用 window.focus() 确保 WebView2 从父窗口请求键盘焦点，
    // 这在 SetForegroundWindow 已将顶层窗口置为前台但 WebView2
    // 尚未分到键盘焦点时尤为关键（体现为 hasFocus=false）。
    // 然后 el.focus() + el.click()：用户反馈鼠标点一下输入框就能
    // 恢复键盘焦点，说明 click 事件链路可以唤醒 WebView2 的键盘输入。
    if (typeof document !== "undefined") {
      window.focus();
      const el = document.getElementById(
        "main-search-input",
      ) as HTMLInputElement | null;
      if (el) {
        el.focus();
        el.click();
      }
    }

    const hasFocus = typeof document !== "undefined" && document.hasFocus();
    const activeEl =
      typeof document !== "undefined" ? document.activeElement : null;
    const isInputFocused =
      activeEl &&
      (activeEl.tagName === "INPUT" || activeEl.tagName === "TEXTAREA");

    // 成功：输入框在 DOM 中已获焦（活跃元素），并且窗口有系统焦点时立即停止。
    // 若 isInputFocused 成立但 hasFocus 仍为 false，再多重试几次，因为
    // WebView2 的 hasFocus 可能落后于实际焦点状态。
    if (isInputFocused && (hasFocus || retries > maxRetries / 2)) {
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
          invoke("force_focus").catch(() => {});
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
