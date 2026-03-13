<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { requestInputFocus } from "$lib/stores/focusInput";
  import { detachWindowShortcut } from "$lib/stores/shortcuts";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { page } from "$app/state";
  import { setupPluginConsoleListener } from "$lib/plugin-console";
  import { Toaster } from "svelte-sonner";
  import WindowResizer from "$lib/components/WindowResizer.svelte";

  // Setup plugin console listener to forward plugin console output to webview devtools
  setupPluginConsoleListener();

  // Subscribe to shortcuts store to trigger auto-loading
  // The subscription itself triggers the load in the store's start function
  $detachWindowShortcut;

  // Focus input when navigating to main page
  $effect(() => {
    if (page.route.id === "/") {
      requestInputFocus();
    }
  });

  // This onMount block sets up a single, persistent listener for the 'esc_key_pressed' event.
  // It will live for the entire duration of the app, avoiding setup/teardown during page navigation.
  onMount(() => {
    const listenersPromise = (async () => {
      // Restore Listener:
      // Listen to "escape_pressed" (standardized event)
      // BUT only handle it if we are NOT on the main page.
      // The main page (+page.svelte) has its own listener.
      const unlisten = await listen("escape_pressed", () => {
        // Check if we are on the main page
        if (page.route.id === "/") {
          // Do NOTHING. Let +page.svelte handle it.
        } else {
          // If not on main page, check if there is a registered handler
          const handler = get(escapeHandler);
          if (handler && typeof handler === "function") {
            handler();
          } else {
            // Fallback
            window.history.back();
          }
        }
      });

      const unlistenVisibility = await listen<boolean>(
        "window_visibility",
        (event) => {
          // When window becomes visible, check if we are on the main page.
          if (event.payload && page.route.id === "/") {
            requestInputFocus();
          }
        },
      );

      const unlistenCommand = await listen<string>(
        "execute_command_by_name",
        (event) => {
          invoke("execute_command", { name: event.payload });
        },
      );

      return { unlisten, unlistenVisibility, unlistenCommand };
    })();

    // The returned cleanup function will only run if the entire layout is destroyed.
    return () => {
      listenersPromise.then(
        ({ unlisten, unlistenVisibility, unlistenCommand }) => {
          unlisten();
          unlistenVisibility();
          unlistenCommand();
        },
      );
    };
  });

  let { children } = $props();
</script>

{@render children()}

<WindowResizer />
<Toaster richColors position="top-center" />
