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
      const unlisten = await listen("esc_key_pressed", () => {
        console.log("Layout: esc_key_pressed received. Delegating to logic.");

        // Check if we are on the main page
        if (page.route.id === "/") {
          // Get the current handler function from the store and execute it.
          const handler = get(escapeHandler);
          handler();
        } else {
          // If not on main page, default behavior is to go back
          window.history.back();
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

<Toaster richColors position="top-center" />
