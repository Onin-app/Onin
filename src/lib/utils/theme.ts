import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { Theme } from '$lib/type';

/**
 * 将主题类应用于 document 元素
 * @param {Theme} theme
 */
const applyTheme = (theme: Theme) => {
  if (browser) {
    document.documentElement.classList.remove(Theme.DARK, Theme.LIGHT, Theme.SYSTEM);
    document.documentElement.classList.add(theme);
  }
};

/**
 * 获取初始主题
 * 优先从 localStorage 获取，其次是系统偏好
 * @returns {Theme}
 */
const getInitialTheme = (): Theme => {
  if (!browser) return Theme.LIGHT; // SSR 期间的默认值

  const storedTheme = localStorage.getItem('theme') as Theme | null;
  if (storedTheme) {
    return storedTheme;
  }

  const userPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  return userPrefersDark ? Theme.DARK : Theme.LIGHT;
};

// 创建 Svelte store
const initialTheme = getInitialTheme();
export const theme = writable<Theme>(initialTheme);

// 订阅 theme store 的变化，以更新 DOM 和 localStorage
theme.subscribe((newTheme) => {
  if (browser) {
    localStorage.setItem('theme', newTheme);
  }
});

// 暴露一个用于切换主题的函数
export const toggleTheme = (currentTheme: Theme) => {
  applyTheme(getTheme(currentTheme));
  theme.update(() => currentTheme);
};

export const getTheme = (currentTheme: Theme): Theme.DARK | Theme.LIGHT => {
  let theme = Theme.DARK
  const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches
  if (currentTheme === Theme.SYSTEM) {
    theme = isDark ? Theme.DARK : Theme.LIGHT
  } else {
    theme = currentTheme
  }
  return theme
}
