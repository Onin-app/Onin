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
    return () => { };
  });

  return {
    subscribe,
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
