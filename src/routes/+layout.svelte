<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import { get } from "svelte/store"; // `get` is still needed for the custom escapeHandler store
  import { page } from "$app/state"; // Use the new reactive state module

  // This onMount block sets up a single, persistent listener for the 'esc_key_pressed' event.
  // It will live for the entire duration of the app, avoiding setup/teardown during page navigation.
  onMount(() => {
    const listenersPromise = (async () => {
      const unlisten = await listen("esc_key_pressed", () => {
        console.log(
          "Layout: esc_key_pressed received. Delegating to current handler.",
        );
        // Get the current handler function from the store and execute it.
        const handler = get(escapeHandler);
        handler();
      });

      const unlistenVisibility = await listen<boolean>(
        "window_visibility",
        (event) => {
          // When window becomes visible, check if we are on the main page.
          if (event.payload && page.route.id === "/") {
            // Use queueMicrotask to ensure the DOM is ready after a potential navigation.
            queueMicrotask(() => {
              const input = document.querySelector<HTMLInputElement>(
                'input[placeholder="Hi Baize!"]',
              );
              input?.focus();
            });
          }
        },
      );

      return { unlisten, unlistenVisibility };
    })();

    // The returned cleanup function will only run if the entire layout is destroyed.
    return () => {
      listenersPromise.then(({ unlisten, unlistenVisibility }) => {
        unlisten();
        unlistenVisibility();
      });
    };
  });
</script>

<slot />
