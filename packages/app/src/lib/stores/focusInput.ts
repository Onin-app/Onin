import { writable } from "svelte/store";

export const focusInputTrigger = writable<number>(0);
export const focusExtensionInputTrigger = writable<number>(0);

/**
 * 请求主搜索输入框聚焦
 */
export function requestInputFocus() {
  focusInputTrigger.update((n) => n + 1);
}

/**
 * 请求扩展搜索输入框聚焦
 */
export function requestExtensionInputFocus() {
  focusExtensionInputTrigger.update((n) => n + 1);
}

/**
 * 在当前微任务周期安全执行 DOM 元素聚焦
 */
export function focusInputElement(element?: HTMLElement | null) {
  if (!element || typeof document === "undefined") {
    return;
  }
  queueMicrotask(() => {
    element.focus();
  });
}
