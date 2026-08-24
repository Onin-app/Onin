import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

/**
 * Create a custom store for detach window shortcut
 * Automatically loads on first subscription
 */
function createDetachWindowShortcutStore() {
  const { subscribe, set } = writable<string>("", (set) => {
    // This function runs when the first subscriber subscribes
    // Automatically load the shortcut from backend
    invoke<string>("get_detach_window_shortcut")
      .then((shortcut) => set(shortcut))
      .catch((e) => console.error("Failed to load detach window shortcut:", e));

    // Return cleanup function (optional)
    return () => {};
  });

  return {
    subscribe,
    set, // 暴露 set 方法以支持 bind:value
    /**
     * Set the shortcut both in backend and store
     */
    setShortcut: async (shortcut: string) => {
      try {
        await invoke("set_detach_window_shortcut", { shortcutStr: shortcut });
        set(shortcut);
      } catch (e) {
        console.error("Failed to set detach window shortcut:", e);
        throw e;
      }
    },
  };
}

export const detachWindowShortcut = createDetachWindowShortcutStore();

/**
 * Create a custom store for the toggle window shortcut (显示/隐藏窗口)
 */
function createToggleWindowShortcutStore() {
  const store = writable<string>("alt+Space");

  // 模块加载时立即拉取后端配置，不依赖 writable 的惰性 start 订阅时机（勿改回惰性加载）：
  // 加载完成前/失败时保持应用默认 "alt+Space"，确保 Alt+Space 拦截器的 gate 恒可用，
  // 避免"按住 Alt 再按 Space 只打空格、窗口不隐藏"的竞态。
  invoke<string>("get_toggle_shortcut")
    .then((shortcut) => {
      if (shortcut) store.set(shortcut);
    })
    .catch((e) => console.error("Failed to load toggle window shortcut:", e));

  return {
    subscribe: store.subscribe,
    set: store.set, // 暴露 set 方法以支持保存后同步更新
    /**
     * Set the shortcut both in backend and store
     */
    setShortcut: async (shortcut: string) => {
      try {
        await invoke("set_toggle_shortcut", { shortcutStr: shortcut });
        store.set(shortcut);
      } catch (e) {
        console.error("Failed to set toggle window shortcut:", e);
        throw e;
      }
    },
  };
}

export const toggleWindowShortcut = createToggleWindowShortcutStore();
