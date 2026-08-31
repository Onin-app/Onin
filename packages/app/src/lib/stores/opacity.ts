import { writable } from "svelte/store";
import { browser } from "$app/environment";

const STORAGE_KEY = "window_opacity";
const DEFAULT_OPACITY = 100;

/**
 * 将透明度应用到 documentElement 的 CSS 变量中
 * @param opacity 不透明度数值 (30 - 100)
 */
export const applyWindowOpacity = (opacity: number) => {
  if (browser) {
    const clamped = Math.max(30, Math.min(100, opacity));
    document.documentElement.style.setProperty(
      "--window-opacity",
      (clamped / 100).toString(),
    );
  }
};

/**
 * 获取初始透明度
 */
const getInitialOpacity = (): number => {
  if (!browser) return DEFAULT_OPACITY;

  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored) {
    const parsed = parseInt(stored, 10);
    if (!isNaN(parsed) && parsed >= 30 && parsed <= 100) {
      return parsed;
    }
  }
  return DEFAULT_OPACITY;
};

const initialOpacity = getInitialOpacity();
if (browser) {
  applyWindowOpacity(initialOpacity);
}

export const windowOpacity = writable<number>(initialOpacity);

windowOpacity.subscribe((value) => {
  if (browser) {
    localStorage.setItem(STORAGE_KEY, value.toString());
    applyWindowOpacity(value);
  }
});

export const setWindowOpacity = (value: number) => {
  const clamped = Math.max(30, Math.min(100, value));
  windowOpacity.set(clamped);
};
