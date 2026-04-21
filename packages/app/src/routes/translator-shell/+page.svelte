<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { emit } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import { Tabs } from "bits-ui";
  import AppScrollArea from "$lib/components/AppScrollArea.svelte";
  import { escapeHandler } from "$lib/stores/escapeHandler";
  import "../../index.css";

  let engine = $state("sougou");

  const engines = [
    { id: "sougou", label: "搜狗" },
    { id: "google", label: "Google" },
    { id: "baidu", label: "百度" },
  ];

  const switchEngine = async (newEngine: string) => {
    if (engine === newEngine) return;
    engine = newEngine;
    await emit("translator_switch", { engine: newEngine });
  };

  const handleEsc = async () => {
    try {
      await WebviewWindow.getCurrent().close();
    } catch (e) {
      console.error("Failed to close translator window on Esc:", e);
    }
  };

  onMount(() => {
    escapeHandler.set(handleEsc);
  });

  onDestroy(() => {
    if (get(escapeHandler) === handleEsc) {
      escapeHandler.set(null);
    }
  });
</script>

<div
  class="flex h-full w-full items-stretch border-b border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-900"
>
  <Tabs.Root
    value={engine}
    onValueChange={(val) => switchEngine(val)}
    class="h-full w-full"
  >
    <AppScrollArea
      class="h-full w-full"
      orientation="horizontal"
      viewportClass="h-full w-full overflow-y-hidden"
    >
      <Tabs.List
        class="flex h-full w-max min-w-full gap-0 bg-gray-200 text-xs font-medium text-gray-500 dark:bg-gray-800 dark:text-gray-400"
      >
        {#each engines as e}
          <Tabs.Trigger
            value={e.id}
            class="flex h-full flex-shrink-0 items-center justify-center px-3 transition-all focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-0 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-white data-[state=active]:text-gray-900 dark:data-[state=active]:bg-gray-950 dark:data-[state=active]:text-gray-50"
          >
            {e.label}
          </Tabs.Trigger>
        {/each}
      </Tabs.List>
    </AppScrollArea>
  </Tabs.Root>
</div>

<style>
  :global(html) {
    height: 100%;
    background: rgb(249 250 251);
  }

  :global(body) {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: rgb(249 250 251);
  }
</style>
