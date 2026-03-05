<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  let isWindows = $state(false);

  onMount(() => {
    // Synchronously check if the OS is Windows via user agent.
    isWindows = navigator.userAgent.toLowerCase().includes("win");
    console.log("[WindowResizer] isWindows set to:", isWindows, navigator.userAgent);
  });

  const handleResize = (direction: string) => {
    if ((window as any).__TAURI_INTERNALS__) {
      getCurrentWebviewWindow().startResizeDragging(direction as any);
    }
  };
</script>

{#if isWindows}
  <!-- The 4 Main Edges -->
  <!-- Top Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-0 left-2 right-2 h-1 cursor-n-resize z-[9999] pointer-events-auto"
    onmousedown={() => handleResize('North')}
  ></div>

  <!-- Bottom Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 left-2 right-2 h-1 cursor-s-resize z-[9999] pointer-events-auto"
    onmousedown={() => handleResize('South')}
  ></div>

  <!-- Left Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-2 bottom-2 left-0 w-1 cursor-w-resize z-[9999] pointer-events-auto"
    onmousedown={() => handleResize('West')}
  ></div>

  <!-- Right Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-2 bottom-2 right-0 w-1 cursor-e-resize z-[9999] pointer-events-auto"
    onmousedown={() => handleResize('East')}
  ></div>

  <!-- The overlapping 4 Corners to catch diagonal drags! -->
  <!-- The trick to making them stick to the rounded corner perfectly is to make them the exact same border size but hollow on the inside! But for drag area we just make them slightly smaller transparent blocks over the curve. -->
  
  <!-- Top Left Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-0 left-0 w-2 h-2 cursor-nw-resize z-[9999] pointer-events-auto rounded-tl-xl"
    onmousedown={() => handleResize('NorthWest')}
  ></div>

  <!-- Top Right Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed top-0 right-0 w-2 h-2 cursor-ne-resize z-[9999] pointer-events-auto rounded-tr-xl"
    onmousedown={() => handleResize('NorthEast')}
  ></div>

  <!-- Bottom Left Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 left-0 w-2 h-2 cursor-sw-resize z-[9999] pointer-events-auto rounded-bl-xl"
    onmousedown={() => handleResize('SouthWest')}
  ></div>

  <!-- Bottom Right Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 right-0 w-2 h-2 cursor-se-resize z-[9999] pointer-events-auto rounded-br-xl"
    onmousedown={() => handleResize('SouthEast')}
  ></div>
{/if}
