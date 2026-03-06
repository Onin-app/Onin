import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { Theme } from "$lib/type";

/**
 * 暴露一个用于计算最终主题（处理 SYSTEM 模式）的函数
 * 放在顶部以避免在订阅时出现 ReferenceError
 */
export const getTheme = (currentTheme: Theme): Theme.DARK | Theme.LIGHT => {
  let theme = Theme.DARK;
  const isDark =
    browser && window.matchMedia("(prefers-color-scheme: dark)").matches;
  if (currentTheme === Theme.SYSTEM) {
    theme = isDark ? Theme.DARK : Theme.LIGHT;
  } else {
    theme = currentTheme;
  }
  return theme;
};

/**
 * 将主题类应用于 document 元素
 * @param {Theme} theme
 */
const applyTheme = (theme: Theme) => {
  if (browser) {
    document.documentElement.classList.remove(
      Theme.DARK,
      Theme.LIGHT,
      Theme.SYSTEM,
    );
    document.documentElement.classList.add(theme);
  }
};

/**
 * 获取初始主题
 * 优先从 localStorage 获取
 * @returns {Theme}
 */
const getInitialTheme = (): Theme => {
  if (!browser) return Theme.DARK; // SSR 期间的默认值

  const storedTheme = localStorage.getItem("theme") as Theme | null;
  if (storedTheme) {
    return storedTheme;
  }

  // 默认使用暗黑模式
  return Theme.DARK;
};

// 创建 Svelte store
const initialTheme = getInitialTheme();
export const theme = writable<Theme>(initialTheme);

// 订阅 theme store 的变化，以更新 DOM 和 localStorage
// 由于 getTheme 和 applyTheme 已在上方定义，这里调用是安全的
theme.subscribe((newTheme) => {
  if (browser) {
    localStorage.setItem("theme", newTheme);
    applyTheme(getTheme(newTheme));
  }
});

/**
 * 暴露一个用于切换主题的函数
 */
export const toggleTheme = (currentTheme: Theme) => {
  theme.update(() => currentTheme);
};
