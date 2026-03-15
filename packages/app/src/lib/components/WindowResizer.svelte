<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let isWindows = $state(false);

  onMount(() => {
    // Synchronously check if the OS is Windows via user agent.
    isWindows = navigator.userAgent.toLowerCase().includes("win");
  });

  const handleResize = (direction: string) => {
    if ((window as any).__TAURI_INTERNALS__) {
      getCurrentWindow().startResizeDragging(direction as any);
    }
  };
</script>

{#if isWindows}
  <!-- The 4 Main Edges -->
  <!-- Top Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed top-0 right-2 left-2 z-[9999] h-1 cursor-n-resize"
    onmousedown={() => handleResize("North")}
  ></div>

  <!-- Bottom Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed right-2 bottom-0 left-2 z-[9999] h-1 cursor-s-resize"
    onmousedown={() => handleResize("South")}
  ></div>

  <!-- Left Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed top-2 bottom-2 left-0 z-[9999] w-1 cursor-w-resize"
    onmousedown={() => handleResize("West")}
  ></div>

  <!-- Right Edge -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed top-2 right-0 bottom-2 z-[9999] w-1 cursor-e-resize"
    onmousedown={() => handleResize("East")}
  ></div>

  <!-- The overlapping 4 Corners to catch diagonal drags! -->
  <!-- The trick to making them stick to the rounded corner perfectly is to make them the exact same border size but hollow on the inside! But for drag area we just make them slightly smaller transparent blocks over the curve. -->

  <!-- Top Left Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed top-0 left-0 z-[9999] h-2 w-2 cursor-nw-resize rounded-tl-xl"
    onmousedown={() => handleResize("NorthWest")}
  ></div>

  <!-- Top Right Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed top-0 right-0 z-[9999] h-2 w-2 cursor-ne-resize rounded-tr-xl"
    onmousedown={() => handleResize("NorthEast")}
  ></div>

  <!-- Bottom Left Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed bottom-0 left-0 z-[9999] h-2 w-2 cursor-sw-resize rounded-bl-xl"
    onmousedown={() => handleResize("SouthWest")}
  ></div>

  <!-- Bottom Right Corner -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pointer-events-auto fixed right-0 bottom-0 z-[9999] h-2 w-2 cursor-se-resize rounded-br-xl"
    onmousedown={() => handleResize("SouthEast")}
  ></div>
{/if}
